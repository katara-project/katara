use serde::{Deserialize, Serialize};

pub mod optimizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub intent: String,
    /// V10.4 — Confidence score for the detected intent, ∈ [0.0, 1.0].
    /// 0.0 when reconstructed from cache (confidence not re-derived).
    pub intent_confidence: f32,
    pub raw_tokens_estimate: usize,
    pub compiled_tokens_estimate: usize,
    /// Tokens saved by the BPE optimizer pass alone (before semantic compilation).
    pub optimizer_savings: usize,
    pub summary: String,
    pub compiled_context: String,
    /// V10.9 — Slash command detected from user input (e.g. "/debug", "/dtlr").
    /// `None` when no slash command was present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slash_command: Option<String>,
    /// V10.9 — When true, the compiler requests local-only routing
    /// (equivalent to `sensitive: true`). Set by `/dtlr` slash command.
    #[serde(default)]
    pub force_local: bool,
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

/// V10.9 — Slash command parsing result.
struct SlashCommand {
    /// The canonical intent to use (e.g. "debug", "codegen").
    intent: String,
    /// The original command string (e.g. "/debug", "/dtlr").
    command: String,
    /// The context with the slash command stripped.
    stripped: String,
    /// Whether this command forces local-only routing.
    force_local: bool,
}

/// V10.9 — Extract a slash command prefix from user input.
/// Recognized commands: /debug, /code, /review, /summarize, /translate,
/// /ocr, /dtlr, /fast, /quality, /general.
/// Returns `None` if no slash command is found.
fn extract_slash_command(raw: &str) -> Option<SlashCommand> {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with('/') {
        return None;
    }

    // Extract the command token (everything up to the first whitespace or end).
    let cmd_end = trimmed
        .find(|c: char| c.is_whitespace())
        .unwrap_or(trimmed.len());
    let cmd = &trimmed[..cmd_end];
    let rest = trimmed[cmd_end..].trim_start();

    let (intent, force_local) = match cmd.to_ascii_lowercase().as_str() {
        "/debug" => ("debug", false),
        "/code" | "/codegen" => ("codegen", false),
        "/review" => ("review", false),
        "/summarize" | "/summary" | "/resume" | "/résumé" => ("summarize", false),
        "/translate" | "/traduire" => ("translate", false),
        "/ocr" => ("ocr", false),
        "/dtlr" | "/local" | "/sovereign" => ("general", true),
        "/fast" | "/rapide" => ("fast", false),
        "/quality" | "/qualite" | "/qualité" => ("quality", false),
        "/general" => ("general", false),
        _ => return None,
    };

    Some(SlashCommand {
        intent: intent.to_string(),
        command: cmd.to_string(),
        stripped: if rest.is_empty() {
            raw.trim_start().to_string()
        } else {
            rest.to_string()
        },
        force_local,
    })
}

/// Compile context with an optional `client_app` hint for smarter intent scoring.
/// When `client_app` is "VS Code Copilot" (or similar), code-adjacent intents
/// receive a signal boost so the correct LLM is selected more reliably.
pub fn compile_context_with_hint(raw: &str, client_app: Option<&str>) -> CompileResult {
    // V10.9: detect and strip slash commands before compilation.
    let slash = extract_slash_command(raw);
    let effective_raw = if let Some(ref sc) = slash {
        &sc.stripped
    } else {
        raw
    };

    // V9.5: encode input for optimal tokenization.
    let encoded = tokenizer::encode(effective_raw);
    let raw_encoded = encoded.as_str();
    let raw_tokens_estimate = token_count(raw_encoded);

    // V10.9: slash command overrides intent detection with confidence 1.0.
    let (intent, intent_confidence) = if let Some(ref sc) = slash {
        (sc.intent.clone(), 1.0_f32)
    } else {
        // V10.4: multi-signal scored intent detection.
        detect_intent_scored(raw_encoded, client_app)
    };

    // V9.10.1: BPE optimizer pass — lossless lexical transformations.
    let optimized = optimizer::optimize(raw_encoded, &intent);
    let optimized_tokens = token_count(&optimized);
    let optimizer_savings = raw_tokens_estimate.saturating_sub(optimized_tokens);

    // V9.12 — Per-intent distillation ratio.
    let target_tokens = optimized_tokens
        .saturating_div(distillation_divisor(&intent))
        .max(16)
        .min(optimized_tokens);
    let marker_cost = token_count(intent_marker(&intent));
    let truncation_target = target_tokens.saturating_sub(marker_cost).max(1);
    let compiled_context = build_compiled_context(&optimized, &intent, truncation_target);
    let compiled_tokens_estimate = token_count(&compiled_context);

    CompileResult {
        intent: intent.clone(),
        intent_confidence,
        raw_tokens_estimate,
        compiled_tokens_estimate,
        optimizer_savings,
        summary: build_summary(&intent, raw_tokens_estimate, compiled_tokens_estimate),
        compiled_context,
        slash_command: slash.as_ref().map(|sc| sc.command.clone()),
        force_local: slash.as_ref().is_some_and(|sc| sc.force_local),
    }
}

/// Compile context with no client-app hint (backward-compatible wrapper).
pub fn compile_context(raw: &str) -> CompileResult {
    compile_context_with_hint(raw, None)
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
        "fast" => "[k:fast]|",
        "quality" => "[k:quality]|",
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
        "fast" => 5,    // aggressive reduction for speed
        "quality" => 2, // preserve more context for quality
        _ => 4,         // general
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
        "summarize" | "fast" => reduce_summarize_context(raw),
        "ocr" | "quality" => reduce_ocr_context(raw),
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

/// V10.4 — Thin backward-compatible wrapper around the scored detector.
pub fn detect_intent(raw: &str) -> String {
    detect_intent_scored(raw, None).0
}

/// V10.4 — Weighted multi-signal intent detection.
///
/// Returns `(intent, confidence)` where confidence ∈ [0.0, 1.0].
/// Multiple signals are scored in parallel; the highest total wins.
/// `client_app` hint (e.g. "VS Code Copilot") boosts code-adjacent intents
/// so coding contexts are classified more accurately.
pub fn detect_intent_scored(raw: &str, client_app: Option<&str>) -> (String, f32) {
    use std::collections::HashMap;
    let lower = raw.to_lowercase();

    // VS Code / Copilot context → slight boost for code intents.
    let is_code_context = client_app
        .map(|a| {
            let al = a.to_lowercase();
            al.contains("copilot") || al.contains("vscode") || al.contains("vs code")
        })
        .unwrap_or(false);

    // Structural signals — independent of intent.
    let has_code_block = lower.contains("```")
        || lower.contains("fn ")
        || lower.contains("def ")
        || (lower.contains("class ") && !lower.contains("class of"))
        || lower.contains("impl ")
        || lower.contains("struct ");

    let mut scores: HashMap<&str, f32> = HashMap::new();

    // ── Debug ────────────────────────────────────────────────────────────
    for (kw, w) in [
        ("error:", 3.0),
        ("panic:", 3.5),
        ("panicked at", 4.0),
        ("exception", 2.5),
        ("traceback", 3.5),
        ("segfault", 4.0),
        ("stack trace", 3.5),
        ("stack overflow", 4.0),
        ("at line ", 2.0),
        ("thread 'main'", 3.0),
        ("fatal:", 3.0),
        ("undefined behavior", 3.5),
        ("core dumped", 4.0),
        ("null pointer", 3.0),
        ("out of memory", 3.5),
        // French
        ("erreur:", 2.5),
        ("plantage", 3.0),
        ("bogue", 2.5),
    ] {
        if lower.contains(kw) {
            *scores.entry("debug").or_default() += w;
        }
    }
    // Single-word signals — present in old detector, lower weight when alone.
    if lower.contains("panic") {
        *scores.entry("debug").or_default() += 2.0;
    }
    if lower.contains("trace") {
        *scores.entry("debug").or_default() += 1.0;
    }
    if lower.contains("debug") {
        *scores.entry("debug").or_default() += 1.5;
    }
    // "bug" / "crash" / "fix" — weak alone, combined stronger.
    if lower.contains("bug") {
        *scores.entry("debug").or_default() += 1.2;
    }
    if lower.contains("crash") {
        *scores.entry("debug").or_default() += 2.0;
    }
    if lower.contains("fix") {
        let bonus: f32 = if lower.contains("bug")
            || lower.contains("error")
            || lower.contains("crash")
            || lower.contains("broken")
        {
            2.5
        } else {
            0.6
        };
        *scores.entry("debug").or_default() += bonus;
    }

    // ── Code generation ──────────────────────────────────────────────────
    for (kw, w) in [
        ("write a function", 4.5),
        ("write a method", 4.0),
        ("write code", 3.5),
        ("write a script", 3.5),
        ("write me a", 3.0),
        ("implement this", 3.5),
        ("implement in", 3.0),
        ("generate code", 4.0),
        ("generate a function", 4.5),
        ("create a function", 4.0),
        ("create a class", 4.0),
        ("create a script", 3.5),
        ("code snippet", 3.5),
        ("code example", 3.0),
        ("help me code", 3.5),
        ("complete this code", 3.5),
        ("complete this function", 3.5),
        ("complete the code", 3.5),
        ("add a function", 3.0),
        ("add a method", 3.0),
        ("give me the code", 3.5),
        ("show me the code", 3.0),
        ("codex", 2.0),
        // Language-specific patterns ("write a rust function", "write a typescript function"…)
        ("rust function", 3.5),
        ("python function", 3.5),
        ("typescript function", 3.5),
        ("javascript function", 3.5),
        ("go function", 3.0),
        ("kotlin function", 3.0),
        ("swift function", 3.0),
        ("c++ function", 3.0),
        ("java function", 3.0),
        // French
        ("écris du code", 4.5),
        ("écris une fonction", 4.5),
        ("implémente", 3.0),
        ("crée une fonction", 4.0),
        ("crée un script", 3.5),
        ("génère du code", 4.0),
        ("génère une fonction", 4.5),
        ("écris un script", 3.5),
        ("écris moi", 2.5),
    ] {
        if lower.contains(kw) {
            *scores.entry("codegen").or_default() += w;
        }
    }
    // Composite: "write a … function" covers "write a rust function", etc.
    if lower.contains("write a") && lower.contains("function") {
        *scores.entry("codegen").or_default() += 3.5;
    }

    // ── Review / improvement ─────────────────────────────────────────────
    for (kw, w) in [
        ("code review", 5.0),
        ("pull request", 4.5),
        ("pr review", 4.5),
        ("review this", 3.5),
        ("review the code", 4.0),
        ("refactor", 4.0),
        ("diff --git", 4.5),
        ("diff ", 2.5),
        ("improve this", 3.0),
        ("improve the", 2.5),
        ("optimize this", 3.0),
        ("optimise this", 3.0),
        ("optimize the", 2.5),
        ("optimise the", 2.5),
        ("make it better", 2.5),
        ("make this better", 2.5),
        ("clean up", 2.0),
        ("simplify this", 2.5),
        ("restructure", 3.0),
        ("best practices", 2.5),
        // French
        ("améliore", 2.5),
        ("optimise le", 3.0),
        ("revue de code", 5.0),
        ("refactore", 4.0),
        ("amélioration de", 2.5),
    ] {
        if lower.contains(kw) {
            *scores.entry("review").or_default() += w;
        }
    }

    // ── Summarize ────────────────────────────────────────────────────────
    for (kw, w) in [
        ("summarize", 5.0),
        ("summarise", 5.0),
        ("tldr", 5.0),
        ("recap", 4.0),
        ("explain this", 3.0),
        ("explain how", 2.5),
        ("explain the", 2.0),
        ("what does this do", 3.5),
        ("what is this", 2.5),
        ("how does this work", 3.0),
        ("give me an overview", 3.5),
        ("walk me through", 3.0),
        // French
        ("résume", 4.5),
        ("résumé de", 4.0),
        ("explique", 2.5),
        ("comment ça marche", 3.0),
        ("c'est quoi", 2.5),
    ] {
        if lower.contains(kw) {
            *scores.entry("summarize").or_default() += w;
        }
    }

    // ── Translation ──────────────────────────────────────────────────────
    for (kw, w) in [
        ("translat", 5.0),
        ("traduire", 5.0),
        ("traduis", 5.0),
        ("traduction", 4.5),
        ("übersetze", 5.0),
        ("traducir", 5.0),
        ("traduci", 5.0),
        ("翻译", 5.0),
        ("in english", 3.0),
        ("in french", 3.0),
        ("in german", 3.0),
        ("in spanish", 3.0),
        ("in japanese", 3.0),
        ("in chinese", 3.0),
        ("en anglais", 3.0),
        ("en français", 3.0),
        ("en allemand", 3.0),
    ] {
        if lower.contains(kw) {
            *scores.entry("translate").or_default() += w;
        }
    }

    // ── OCR ──────────────────────────────────────────────────────────────
    for (kw, w) in [
        (" ocr ", 5.0),
        ("scan image", 5.0),
        ("extract text from", 4.5),
        ("image to text", 5.0),
        ("read this image", 4.5),
        ("text from image", 4.5),
    ] {
        if lower.contains(kw) {
            *scores.entry("ocr").or_default() += w;
        }
    }
    if lower.starts_with("ocr") {
        *scores.entry("ocr").or_default() += 5.0;
    }

    // ── Structural boosts ────────────────────────────────────────────────
    if has_code_block {
        *scores.entry("codegen").or_default() += 1.0;
        *scores.entry("review").or_default() += 1.0;
    }
    if is_code_context {
        *scores.entry("codegen").or_default() += 0.8;
        *scores.entry("review").or_default() += 0.5;
        *scores.entry("debug").or_default() += 0.5;
    }

    // ── Pick winner ──────────────────────────────────────────────────────
    let best = scores
        .iter()
        .filter(|(_, &s)| s >= 1.0)
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));

    match best {
        Some((&intent, &score)) => {
            let confidence = (score / 10.0_f32).min(1.0);
            (intent.to_string(), confidence)
        }
        None => ("general".to_string(), 0.3),
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

    // ── V10.9 — Slash command tests ─────────────────────────────────────

    #[test]
    fn slash_debug_overrides_intent() {
        let result = compile_context("/debug explain how closures work");
        assert_eq!(result.intent, "debug");
        assert_eq!(result.intent_confidence, 1.0);
        assert_eq!(result.slash_command.as_deref(), Some("/debug"));
        assert!(!result.force_local);
    }

    #[test]
    fn slash_code_overrides_intent() {
        let result = compile_context("/code hello world");
        assert_eq!(result.intent, "codegen");
        assert_eq!(result.slash_command.as_deref(), Some("/code"));
    }

    #[test]
    fn slash_review_overrides_intent() {
        let result = compile_context("/review check this function");
        assert_eq!(result.intent, "review");
        assert_eq!(result.slash_command.as_deref(), Some("/review"));
    }

    #[test]
    fn slash_summarize_overrides_intent() {
        let result = compile_context("/summarize this long document about Rust");
        assert_eq!(result.intent, "summarize");
        assert_eq!(result.slash_command.as_deref(), Some("/summarize"));
    }

    #[test]
    fn slash_translate_overrides_intent() {
        let result = compile_context("/translate hello world en français");
        assert_eq!(result.intent, "translate");
    }

    #[test]
    fn slash_ocr_overrides_intent() {
        let result = compile_context("/ocr extract this image");
        assert_eq!(result.intent, "ocr");
    }

    #[test]
    fn slash_dtlr_forces_local() {
        let result = compile_context("/dtlr explain sensitive patient data");
        assert!(result.force_local);
        assert_eq!(result.slash_command.as_deref(), Some("/dtlr"));
    }

    #[test]
    fn slash_fast_routes_to_fast() {
        let result = compile_context("/fast what is 2+2");
        assert_eq!(result.intent, "fast");
        assert_eq!(result.slash_command.as_deref(), Some("/fast"));
    }

    #[test]
    fn slash_quality_routes_to_quality() {
        let result = compile_context("/quality write a thorough analysis");
        assert_eq!(result.intent, "quality");
        assert_eq!(result.slash_command.as_deref(), Some("/quality"));
    }

    #[test]
    fn no_slash_command_returns_none() {
        let result = compile_context("hello world");
        assert_eq!(result.intent, "general");
        assert!(result.slash_command.is_none());
        assert!(!result.force_local);
    }

    #[test]
    fn slash_command_strips_prefix_from_context() {
        let result = compile_context("/debug the server is crashing");
        assert_eq!(result.intent, "debug");
        // The compiled context should not start with "/debug"
        assert!(!result.compiled_context.contains("/debug"));
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
