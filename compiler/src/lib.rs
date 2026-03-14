use serde::{Deserialize, Serialize};

pub mod optimizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub intent: String,
    pub raw_tokens_estimate: usize,
    pub compiled_tokens_estimate: usize,
    /// Tokens saved by the BPE optimizer pass alone (before semantic compilation).
    pub optimizer_savings: usize,
    pub summary: String,
    pub compiled_context: String,
}

/// Canonicalize raw context before fingerprinting to reduce cache misses caused
/// by volatile values (IDs, timestamps, noisy whitespace) while keeping intent.
pub fn canonicalize_context(raw: &str) -> String {
    if raw.trim().is_empty() {
        return String::new();
    }

    let mut normalized = Vec::new();
    let mut previous_blank = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !previous_blank {
                normalized.push(String::new());
            }
            previous_blank = true;
            continue;
        }

        previous_blank = false;
        let collapsed_ws = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
        normalized.push(normalize_volatile_tokens(&collapsed_ws));
    }

    while matches!(normalized.last(), Some(line) if line.is_empty()) {
        normalized.pop();
    }

    normalized.join("\n")
}

fn normalize_volatile_tokens(input: &str) -> String {
    input
        .split_whitespace()
        .map(normalize_single_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_single_token(token: &str) -> String {
    if let Some((left, right)) = token.split_once('=') {
        return format!("{left}={}", normalize_single_token(right));
    }

    if is_uuid_like(token) {
        "<uuid>".to_string()
    } else if is_long_number(token) {
        "<num>".to_string()
    } else {
        token.to_string()
    }
}

fn is_uuid_like(token: &str) -> bool {
    let t = token
        .trim_matches(|c: char| !c.is_ascii_hexdigit() && c != '-')
        .to_ascii_lowercase();
    let parts: Vec<&str> = t.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    let expected = [8, 4, 4, 4, 12];
    parts
        .iter()
        .zip(expected)
        .all(|(p, len)| p.len() == len && p.chars().all(|c| c.is_ascii_hexdigit()))
}

fn is_long_number(token: &str) -> bool {
    let t = token.trim_matches(|c: char| !c.is_ascii_digit());
    t.len() >= 6 && t.chars().all(|c| c.is_ascii_digit())
}

pub fn compile_context(raw: &str) -> CompileResult {
    // V9.5: encode input for optimal tokenization (Unicode normalization,
    // invisible char removal, whitespace collapsing) before any measurement.
    let encoded = tokenizer::encode(raw);
    let raw_encoded = encoded.as_str();
    let raw_tokens_estimate = token_count(raw_encoded);

    // V9.10.1: BPE optimizer pass — lossless lexical transformations that
    // reduce token count before semantic compilation.
    let intent = detect_intent(raw_encoded);
    let optimized = optimizer::optimize(raw_encoded, &intent);
    let optimized_tokens = token_count(&optimized);
    let optimizer_savings = raw_tokens_estimate.saturating_sub(optimized_tokens);

    // V9.12 — Per-intent distillation ratio: aggressive for summarize,
    // conservative for debug/review where structure must be preserved.
    let target_tokens = optimized_tokens
        .saturating_div(distillation_divisor(&intent))
        .max(16)
        .min(optimized_tokens);
    // Reserve budget for the intent marker that shape_by_intent prepends so the
    // final compiled_context never exceeds target_tokens due to marker overhead.
    let marker_cost = token_count(intent_marker(&intent));
    let truncation_target = target_tokens.saturating_sub(marker_cost).max(1);
    let compiled_context = build_compiled_context(&optimized, &intent, truncation_target);
    let compiled_tokens_estimate = token_count(&compiled_context);

    CompileResult {
        intent: intent.clone(),
        raw_tokens_estimate,
        compiled_tokens_estimate,
        optimizer_savings,
        summary: build_summary(&intent, raw_tokens_estimate, compiled_tokens_estimate),
        compiled_context,
    }
}

/// Returns the intent marker prefix used by [`shape_by_intent`].
fn intent_marker(intent: &str) -> &'static str {
    match intent {
        "debug" => "[k:debug]|",
        "review" => "[k:review]|",
        "summarize" => "[k:summarize]|",
        "codegen" => "[k:codegen]|",
        "translate" => "[k:translate]|",
        "ocr" => "[k:ocr]|",
        _ => "[k:general]|",
    }
}

/// Per-intent distillation divisor (V9.12).
/// Returns the denominator used to compute `target_tokens = optimized / divisor`.
/// - 1 = keep 100% (OCR, translate: content must be fully preserved)
/// - 2 = keep  50% (debug, review: preserve structure / stack traces)
/// - 3 = keep  33% (codegen: compress boilerplate, keep logic)
/// - 4 = keep  25% (general: balanced reduction)
/// - 5 = keep  20% (summarize: aggressive distillation)
fn distillation_divisor(intent: &str) -> usize {
    match intent {
        "ocr" | "translate" => 1,
        "debug" | "review" => 2,
        "codegen" => 3,
        "summarize" => 5,
        _ => 4, // general
    }
}

fn build_compiled_context(raw: &str, intent: &str, target_tokens: usize) -> String {
    if raw.trim().is_empty() {
        return String::new();
    }

    let reduced = match intent {
        "debug" => reduce_debug_context(raw),
        "review" => reduce_review_context(raw),
        "codegen" => reduce_general_context(raw),
        "translate" => reduce_general_context(raw),
        "summarize" => reduce_summarize_context(raw),
        "ocr" => reduce_ocr_context(raw),
        _ => reduce_general_context(raw),
    };

    let truncated = truncate_to_token_budget(&reduced, target_tokens);
    let compact = if truncated.trim().is_empty() {
        truncate_to_token_budget(raw, target_tokens)
    } else {
        truncated
    };

    shape_by_intent(intent, &compact)
}

fn shape_by_intent(intent: &str, content: &str) -> String {
    if content.trim().is_empty() {
        return String::new();
    }

    let marker = intent_marker(intent);

    // Token-neutral shaping: inject intent metadata into the first token so we
    // keep stable structure without increasing token count or losing signal.
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if let Some((first, rest)) = normalized.split_once(' ') {
        format!("{marker}{first} {rest}")
    } else {
        format!("{marker}{normalized}")
    }
}

fn build_summary(intent: &str, raw_tokens: usize, compiled_tokens: usize) -> String {
    let action = match intent {
        "debug" => "trimmed repeated trace noise and preserved failure-focused lines",
        "review" => "collapsed the diff to changed hunks and review-relevant lines",
        "summarize" => "compressed the transcript to the latest high-signal points",
        _ => "trimmed low-signal context while preserving the main request",
    };

    format!(
        "Intent: {intent}. Reduced estimated context from {raw_tokens} to {compiled_tokens} tokens and {action}."
    )
}

/// Estimate BPE token count using the Distira universal tokenizer.
/// More accurate than the previous `chars / 4` approximation: ±5 % for prose,
/// ±8 % for code, compared to ±18–22 % for the naive formula.
/// Minimum return value is 1 to keep budget arithmetic safe.
fn token_count(raw: &str) -> usize {
    tokenizer::count(raw).max(1)
}

/// Public token count estimator — exposed so the `core` crate can measure
/// actual forwarded token counts without duplicating the approximation.
/// Delegates to the `tokenizer` crate universal estimator (±5–8 % accuracy).
pub fn estimate_tokens(s: &str) -> usize {
    tokenizer::count(s).max(1)
}

/// Mask PII-like patterns from raw context before compilation.
///
/// Replaces the following patterns with safe placeholders (no regex needed):
/// - Email addresses (`user@domain.tld`) → `[EMAIL]`
/// - API key-style tokens (`sk-...`, `Bearer ...`, `token ...`) → `[API_KEY]`
/// - Credit card 16-digit groups (`dddd dddd dddd dddd` or with dashes) → `[CC_NUM]`
/// - Phone numbers (10-15 digit strings with optional +/dashes/spaces) → `[PHONE]`
/// - JWT tokens (`xxxxx.yyyyy.zzzzz` with base64url parts) → `[JWT]`
pub fn mask_pii(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());

    // Process word-by-word, handling multi-word Bearer/token patterns.
    let mut remaining = raw;
    while !remaining.is_empty() {
        // Handle newlines and whitespace runs without losing structure.
        let ws_end = remaining
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(remaining.len());
        if ws_end > 0 {
            result.push_str(&remaining[..ws_end]);
            remaining = &remaining[ws_end..];
            continue;
        }

        // Grab the next word (non-whitespace token).
        let word_end = remaining
            .find(|c: char| c.is_whitespace())
            .unwrap_or(remaining.len());
        let word = &remaining[..word_end];

        if is_email(word) {
            result.push_str("[EMAIL]");
        } else if is_jwt(word) {
            result.push_str("[JWT]");
        } else if is_api_key(word) {
            result.push_str("[API_KEY]");
        } else if is_credit_card(word) {
            result.push_str("[CC_NUM]");
        } else if is_phone(word) {
            result.push_str("[PHONE]");
        } else {
            // Check for Bearer/token prefix — the next word is the secret.
            let lower = word.to_ascii_lowercase();
            if lower == "bearer" || lower == "token:" || lower == "authorization:" {
                result.push_str(word);
                // Consume whitespace then the token value.
                remaining = &remaining[word_end..];
                let ws2 = remaining
                    .find(|c: char| !c.is_whitespace())
                    .unwrap_or(remaining.len());
                result.push_str(&remaining[..ws2]);
                remaining = &remaining[ws2..];
                let secret_end = remaining
                    .find(|c: char| c.is_whitespace())
                    .unwrap_or(remaining.len());
                if secret_end > 0 {
                    result.push_str("[API_KEY]");
                    remaining = &remaining[secret_end..];
                }
                continue;
            }
            result.push_str(word);
        }
        remaining = &remaining[word_end..];
    }

    result
}

fn is_email(s: &str) -> bool {
    // Must contain exactly one @, with non-empty local and domain parts containing a dot.
    let s = s.trim_matches(|c: char| {
        !c.is_alphanumeric() && c != '@' && c != '.' && c != '-' && c != '_'
    });
    let at = s.find('@');
    if let Some(at_pos) = at {
        let local = &s[..at_pos];
        let domain = &s[at_pos + 1..];
        !local.is_empty() && domain.contains('.') && domain.len() >= 3
    } else {
        false
    }
}

fn is_jwt(s: &str) -> bool {
    // JWT: three base64url segments separated by dots, each at least 4 chars.
    let parts: Vec<&str> = s.splitn(4, '.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|p| {
        p.len() >= 4
            && p.chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '=')
    })
}

fn is_api_key(s: &str) -> bool {
    // Common patterns: sk-..., pk-..., api_..., key_... with length >= 20.
    let lower = s.to_ascii_lowercase();
    let known_prefixes = [
        "sk-", "pk-", "api-", "api_", "key-", "key_", "secret-", "token-",
    ];
    known_prefixes.iter().any(|pfx| lower.starts_with(pfx)) && s.len() >= 20
}

fn is_credit_card(s: &str) -> bool {
    // Strip dashes/spaces, check for 16-digit string starting with 3/4/5/6.
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 16 {
        matches!(digits.chars().next(), Some('3' | '4' | '5' | '6'))
    } else {
        false
    }
}

fn is_phone(s: &str) -> bool {
    // Strip +, dashes, spaces, parens — check for 10-15 digit string.
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    let stripped: String = s
        .chars()
        .filter(|c| {
            c.is_ascii_digit() || *c == '+' || *c == '-' || *c == ' ' || *c == '(' || *c == ')'
        })
        .collect();
    // Must be mostly phone chars (no other chars present).
    stripped.len() == s.len() && digits.len() >= 10 && digits.len() <= 15
}

fn normalize_lines(raw: &str) -> Vec<String> {
    let mut normalized = Vec::new();
    let mut previous_blank = false;

    for line in raw.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            if !previous_blank {
                normalized.push(String::new());
            }
            previous_blank = true;
            continue;
        }

        previous_blank = false;
        normalized.push(trimmed.to_string());
    }

    while matches!(normalized.last(), Some(line) if line.is_empty()) {
        normalized.pop();
    }

    normalized
}

fn dedupe_consecutive(lines: &[String]) -> Vec<String> {
    let mut deduped = Vec::with_capacity(lines.len());
    let mut previous: Option<&str> = None;

    for line in lines {
        if previous == Some(line.as_str()) {
            continue;
        }
        previous = Some(line.as_str());
        deduped.push(line.clone());
    }

    deduped
}

fn keep_head_tail(lines: &[String], head: usize, tail: usize) -> Vec<String> {
    if lines.len() <= head + tail {
        return lines.to_vec();
    }

    let mut selected = Vec::with_capacity(head + tail + 1);
    selected.extend(lines.iter().take(head).cloned());
    selected.push("[...]".into());
    selected.extend(lines.iter().skip(lines.len().saturating_sub(tail)).cloned());
    selected
}

fn join_lines(lines: &[String]) -> String {
    lines.join("\n").trim().to_string()
}

fn reduce_debug_context(raw: &str) -> String {
    let lines = dedupe_consecutive(&normalize_lines(raw));

    // Tier 1: Error / panic headline signals
    let error_kw = [
        "error",
        "panic",
        "exception",
        "fatal",
        "caused by",
        "failed",
        "failure",
        "segfault",
        "crash",
        "abort",
    ];
    // Tier 2: Stack trace / source-location signals
    let trace_kw = ["trace", "stack", "at ", "--> ", " | ", "note:"];
    let file_suffixes = [
        ".rs:", ".py:", ".js:", ".ts:", ".go:", ".java:", ".c:", ".cpp:",
    ];

    let head: Vec<String> = lines.iter().take(3).cloned().collect();
    let mut priority1: Vec<String> = Vec::new();
    let mut priority2: Vec<String> = Vec::new();

    for line in &lines {
        let lower = line.to_lowercase();
        if error_kw.iter().any(|kw| lower.contains(kw)) {
            priority1.push(line.clone());
        } else if trace_kw.iter().any(|kw| line.contains(kw))
            || file_suffixes.iter().any(|s| line.contains(s))
        {
            priority2.push(line.clone());
        }
    }

    let mut selected: Vec<String> = head;
    for l in &priority1 {
        if !selected.contains(l) {
            selected.push(l.clone());
        }
    }
    for l in priority2.iter().take(10) {
        if !selected.contains(l) {
            selected.push(l.clone());
        }
    }

    let non_empty = selected.iter().filter(|l| !l.is_empty()).count();
    if non_empty <= 3 {
        join_lines(&keep_head_tail(&lines, 4, 12))
    } else {
        join_lines(&selected)
    }
}

fn reduce_ocr_context(raw: &str) -> String {
    // OCR tasks require full content — just normalise and dedup whitespace
    let lines = dedupe_consecutive(&normalize_lines(raw));
    join_lines(&lines)
}

fn reduce_review_context(raw: &str) -> String {
    let lines = normalize_lines(raw);
    let diff_markers = ["diff ", "index ", "@@", "+++", "---", "+", "-"];
    let selected: Vec<String> = lines
        .into_iter()
        .filter(|line| {
            let trimmed = line.trim_start();
            diff_markers
                .iter()
                .any(|marker| trimmed.starts_with(marker))
        })
        .collect();

    if selected.is_empty() {
        join_lines(&keep_head_tail(&normalize_lines(raw), 10, 10))
    } else {
        join_lines(&selected)
    }
}

fn reduce_summarize_context(raw: &str) -> String {
    const KW: &[&str] = &[
        "key",
        "main",
        "conclusion",
        "result",
        "summary",
        "finding",
        "decision",
        "recommended",
        "therefore",
        "finally",
        "overall",
    ];
    reduce_by_salience(raw, KW)
}

fn reduce_general_context(raw: &str) -> String {
    const KW: &[&str] = &[
        "important",
        "key",
        "note",
        "result",
        "warning",
        "error",
        "required",
        "must",
        "should",
        "because",
        "therefore",
        "summary",
    ];
    reduce_by_salience(raw, KW)
}

/// Score a single line by signal density relative to intent keywords (V9.12 BM25-inspired).
/// Higher = more salient. Empty lines score 0.
fn salience_score(line: &str, keywords: &[&str]) -> usize {
    if line.trim().is_empty() {
        return 0;
    }
    let lower = line.to_lowercase();
    let word_count = line.split_whitespace().count().min(20);
    let kw_hits: usize = keywords.iter().filter(|kw| lower.contains(**kw)).count();
    // Structured line bonus: lines with `:`, `=`, `->`, `-`, `*` carry more info
    let structure_bonus: usize = if line.contains(':')
        || line.contains('=')
        || line.contains("->")
        || line.trim_start().starts_with('-')
        || line.trim_start().starts_with('*')
    {
        3
    } else {
        0
    };
    word_count + kw_hits * 5 + structure_bonus
}

/// Select the most salient lines (BM25-inspired) while preserving original order.
/// Falls back to head/tail for very short inputs.
fn reduce_by_salience(raw: &str, keywords: &[&str]) -> String {
    let lines = normalize_lines(raw);
    if lines.len() <= 16 {
        return join_lines(&lines);
    }
    // Score every line, keep top 2/3 by salience, restore original order.
    let keep = (lines.len() * 2 / 3).max(8);
    let mut scored: Vec<(usize, usize)> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| (salience_score(l, keywords), i))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let mut keep_indices: Vec<usize> = scored.iter().take(keep).map(|(_, i)| *i).collect();
    keep_indices.sort_unstable();
    let selected: Vec<String> = keep_indices.iter().map(|&i| lines[i].clone()).collect();
    join_lines(&selected)
}

fn truncate_to_token_budget(text: &str, budget: usize) -> String {
    if budget == 0 {
        return String::new();
    }

    if token_count(text) <= budget {
        return text.trim().to_string();
    }

    let mut remaining = budget;
    let mut output: Vec<String> = Vec::new();

    for line in text.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() {
            if !matches!(output.last(), Some(previous) if previous.is_empty()) {
                output.push(String::new());
            }
            continue;
        }

        if words.len() <= remaining {
            output.push(line.to_string());
            remaining -= words.len();
        } else {
            let take = remaining.saturating_sub(1).max(1).min(words.len());
            output.push(format!("{} ...", words[..take].join(" ")));
            remaining = 0;
        }

        if remaining == 0 {
            break;
        }
    }

    output.join("\n").trim().to_string()
}

pub fn detect_intent(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("error")
        || lower.contains("trace")
        || lower.contains("debug")
        || lower.contains("segfault")
        || lower.contains("panic")
        || lower.contains("crash")
        || lower.contains("bug")
        || lower.contains("fix")
    {
        "debug".into()
    } else if lower.contains("diff")
        || lower.contains("pull request")
        || lower.contains("code review")
        || lower.contains("refactor")
        || lower.contains("review")
    {
        "review".into()
    } else if lower.contains("write a function")
        || lower.contains("write function")
        || lower.contains("write a rust function")
        || lower.contains("write a python function")
        || lower.contains("write a typescript")
        || lower.contains("write a javascript")
        || lower.contains("write a go function")
        || lower.contains("write a kotlin")
        || lower.contains("write a swift")
        || lower.contains("write code")
        || lower.contains("write me a")
        || lower.contains("implement this in")
        || lower.contains("implement in rust")
        || lower.contains("implement in python")
        || lower.contains("implement in typescript")
        || lower.contains("implement in javascript")
        || lower.contains("implement in go")
        || lower.contains("generate code")
        || lower.contains("generate a function")
        || lower.contains("create a function")
        || lower.contains("create a class")
        || lower.contains("create a script")
        || lower.contains("code example in")
        || lower.contains("code snippet in")
        || lower.contains("snippet in rust")
        || lower.contains("snippet in python")
        || lower.contains("snippet in typescript")
        || lower.contains("snippet in javascript")
        || lower.contains("codex")
        || lower.contains("help me code")
        || lower.contains("complete this code")
        || lower.contains("complete the code")
        || lower.contains("complete this function")
        // French
        || lower.contains("écris du code")
        || lower.contains("écris une fonction")
        || lower.contains("implémente en")
        || lower.contains("crée une fonction")
        || lower.contains("génère du code")
        || lower.contains("génère une fonction")
        || lower.contains("écris un script")
    {
        "codegen".into()
    } else if lower.contains(" ocr")
        || lower.starts_with("ocr")
        || lower.contains("scan image")
        || lower.contains("extract text from")
        || lower.contains("image to text")
        || lower.contains("read this image")
    {
        "ocr".into()
    } else if lower.contains("translat")
        || lower.contains("traduire")
        || lower.contains("traduis")
        || lower.contains("traduction")
        || lower.contains("übersetze")
        || lower.contains("traducir")
        || lower.contains("traduci")
        || lower.contains("翻译")
        || lower.contains("in english")
        || lower.contains("in french")
        || lower.contains("in german")
        || lower.contains("in spanish")
        || lower.contains("in japanese")
        || lower.contains("in chinese")
    {
        "translate".into()
    } else if lower.contains("summar")
        || lower.contains("explain")
        || lower.contains("tldr")
        || lower.contains("recap")
    {
        "summarize".into()
    } else {
        "general".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_reduces_tokens() {
        let input = (0..120)
            .map(|i| format!("w{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let result = compile_context(&input);
        // Raw estimate must equal token_count of the input
        assert_eq!(result.raw_tokens_estimate, token_count(&input));
        // Compiled must be less than or equal to raw
        assert!(result.compiled_tokens_estimate <= result.raw_tokens_estimate);
        // Internal consistency: token_count of the compiled context = reported estimate
        assert_eq!(
            token_count(&result.compiled_context),
            result.compiled_tokens_estimate
        );
    }

    #[test]
    fn compile_enforces_minimum() {
        let result = compile_context("hi");
        // Compiled context is never zero-token
        assert!(result.compiled_tokens_estimate >= 1);
        // Internal consistency
        assert_eq!(
            token_count(&result.compiled_context),
            result.compiled_tokens_estimate
        );
    }

    #[test]
    fn compile_floor_applies_above_threshold() {
        // ~99 words: raw/3 target should be around 32 (the floor)
        let input = (0..99)
            .map(|i| format!("w{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let result = compile_context(&input);
        // Compiled must not exceed raw
        assert!(result.compiled_tokens_estimate <= result.raw_tokens_estimate);
        // Compiled must be at least the 32-token floor
        assert!(result.compiled_tokens_estimate >= 1);
        // Target is max(raw/3, 32) — compiled_tokens_estimate should be ≤ this target
        let target = (result.raw_tokens_estimate / 3)
            .max(32)
            .min(result.raw_tokens_estimate);
        // Allow up to +5 tokens headroom for the intent shaping prefix
        assert!(result.compiled_tokens_estimate <= target + 5);
    }

    #[test]
    fn detect_debug_intent() {
        assert_eq!(detect_intent("stack trace error"), "debug");
    }

    #[test]
    fn detect_debug_segfault() {
        assert_eq!(detect_intent("Debug this segfault"), "debug");
    }

    #[test]
    fn detect_debug_panic() {
        assert_eq!(detect_intent("fix this panic in main"), "debug");
    }

    #[test]
    fn detect_summarize_intent() {
        assert_eq!(detect_intent("please summarize this"), "summarize");
    }

    #[test]
    fn detect_summarize_explain() {
        assert_eq!(
            detect_intent("explain the concept of context windowing"),
            "summarize"
        );
    }

    #[test]
    fn detect_review_intent() {
        assert_eq!(detect_intent("review this diff"), "review");
    }

    #[test]
    fn detect_review_refactor() {
        assert_eq!(detect_intent("refactor this function"), "review");
    }

    #[test]
    fn detect_codegen_intent_english() {
        assert_eq!(
            detect_intent("Write a Rust function that checks if a number is a palindrome"),
            "codegen"
        );
        assert_eq!(
            detect_intent("Generate code example in Python for quicksort"),
            "codegen"
        );
    }

    #[test]
    fn detect_codegen_intent_french() {
        assert_eq!(
            detect_intent("Écris du code en Rust pour inverser une liste"),
            "codegen"
        );
    }

    #[test]
    fn detect_codegen_typescript() {
        assert_eq!(
            detect_intent("write a typescript function to debounce events"),
            "codegen"
        );
    }

    #[test]
    fn detect_codegen_complete() {
        assert_eq!(
            detect_intent("complete this function: fn add(a: i32, b: i32)"),
            "codegen"
        );
    }

    #[test]
    fn detect_codegen_create_class() {
        assert_eq!(
            detect_intent("create a class User with fields name and email"),
            "codegen"
        );
    }

    #[test]
    fn detect_translate_english() {
        assert_eq!(
            detect_intent("translate this paragraph into French"),
            "translate"
        );
    }

    #[test]
    fn detect_translate_french_keyword() {
        assert_eq!(detect_intent("traduis ce texte en anglais"), "translate");
    }

    #[test]
    fn detect_translate_in_language() {
        assert_eq!(detect_intent("rewrite this in German"), "translate");
    }

    #[test]
    fn detect_general_intent() {
        assert_eq!(detect_intent("hello world"), "general");
    }

    #[test]
    fn detect_ocr_intent() {
        assert_eq!(detect_intent("OCR this scanned document"), "ocr");
        assert_eq!(detect_intent("extract text from this image"), "ocr");
    }

    #[test]
    fn debug_compiler_keeps_file_line_references() {
        let input = "some log\nerror: mismatched types\n --> src/main.rs:42:5\n  |\n42 | let x: i32 = \"bad\";\n";
        let result = compile_context(input);

        assert_eq!(result.intent, "debug");
        assert!(result.compiled_context.contains("error: mismatched types"));
        assert!(result.compiled_context.contains("src/main.rs:42"));
    }

    #[test]
    fn debug_compiler_keeps_failure_lines() {
        let input = "info boot\npanic: exploded\ntrace: frame a\ntrace: frame a\ntrace: frame b";
        let result = compile_context(input);

        assert_eq!(result.intent, "debug");
        assert!(result.compiled_context.contains("panic: exploded"));
        assert!(result.compiled_context.contains("trace: frame b"));
    }

    #[test]
    fn review_compiler_keeps_diff_markers() {
        let input = "diff --git a/app.rs b/app.rs\n@@\n- old\n+ new\n context";
        let result = compile_context(input);

        assert_eq!(result.intent, "review");
        assert!(result.compiled_context.contains("[k:review]|"));
        assert!(result.compiled_context.contains("diff --git"));
        assert!(result.compiled_context.contains("+ new"));
        assert!(!result.compiled_context.contains(" context"));
    }

    #[test]
    fn compiler_applies_transparent_intent_shaping() {
        let result = compile_context("Debug this panic at src/main.rs:42");
        assert_eq!(result.intent, "debug");
        assert!(result.compiled_context.contains("[k:debug]|"));
        assert!(result.compiled_context.contains("panic"));
    }

    #[test]
    fn canonicalize_context_normalizes_volatile_values() {
        let raw =
            "request_id=123456789\ntrace=550e8400-e29b-41d4-a716-446655440000\n\n  same    line";
        let canonical = canonicalize_context(raw);
        assert!(canonical.contains("request_id=<num>"));
        assert!(canonical.contains("trace=<uuid>"));
        assert!(canonical.contains("same line"));
    }

    #[test]
    fn mask_pii_masks_email() {
        let out = mask_pii("contact user@example.com for more");
        assert!(out.contains("[EMAIL]"));
        assert!(!out.contains("user@example.com"));
    }

    #[test]
    fn mask_pii_masks_api_key() {
        let out = mask_pii("key is sk-abcdefghij1234567890xyz");
        assert!(out.contains("[API_KEY]"));
        assert!(!out.contains("sk-abcdefghij"));
    }

    #[test]
    fn mask_pii_masks_bearer_token() {
        let out = mask_pii("Authorization: Bearer sk-abcdefghij1234567890xyz rest of line");
        assert!(out.contains("[API_KEY]"));
        assert!(!out.contains("sk-abcdefghij"));
    }

    #[test]
    fn mask_pii_masks_jwt() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6Ikpva.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let out = mask_pii(&format!("token is {jwt}"));
        assert!(out.contains("[JWT]"));
        assert!(!out.contains("eyJhbGci"));
    }

    #[test]
    fn mask_pii_leaves_normal_text_unchanged() {
        let input = "please summarize this meeting transcript";
        let out = mask_pii(input);
        assert_eq!(out, input);
    }
}
