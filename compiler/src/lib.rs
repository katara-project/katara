use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub intent: String,
    pub raw_tokens_estimate: usize,
    pub compiled_tokens_estimate: usize,
    pub summary: String,
    pub compiled_context: String,
}

pub fn compile_context(raw: &str) -> CompileResult {
    let raw_tokens_estimate = token_count(raw);
    let target_tokens = raw_tokens_estimate
        .saturating_div(3)
        .max(32)
        .min(raw_tokens_estimate);
    let intent = detect_intent(raw);
    let compiled_context = build_compiled_context(raw, &intent, target_tokens);
    let compiled_tokens_estimate = token_count(&compiled_context);

    CompileResult {
        intent: intent.clone(),
        raw_tokens_estimate,
        compiled_tokens_estimate,
        summary: build_summary(&intent, raw_tokens_estimate, compiled_tokens_estimate),
        compiled_context,
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
        "summarize" => reduce_summarize_context(raw),
        "ocr" => reduce_ocr_context(raw),
        _ => reduce_general_context(raw),
    };

    let truncated = truncate_to_token_budget(&reduced, target_tokens);
    if truncated.trim().is_empty() {
        truncate_to_token_budget(raw, target_tokens)
    } else {
        truncated
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

fn token_count(raw: &str) -> usize {
    raw.split_whitespace().count()
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
    let lines = normalize_lines(raw);
    join_lines(&keep_head_tail(&lines, 6, 8))
}

fn reduce_general_context(raw: &str) -> String {
    let lines = normalize_lines(raw);
    join_lines(&keep_head_tail(&lines, 8, 8))
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
        || lower.contains("implement this in")
        || lower.contains("implement in rust")
        || lower.contains("implement in python")
        || lower.contains("generate code")
        || lower.contains("generate a function")
        || lower.contains("code example in")
        || lower.contains("code snippet in")
        || lower.contains("snippet in rust")
        || lower.contains("snippet in python")
        || lower.contains("écris du code")
        || lower.contains("écris une fonction")
        || lower.contains("implémente en")
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
        assert_eq!(result.raw_tokens_estimate, 120);
        assert_eq!(result.compiled_tokens_estimate, 40);
        assert!(result.compiled_tokens_estimate <= result.raw_tokens_estimate);
        assert_eq!(
            token_count(&result.compiled_context),
            result.compiled_tokens_estimate
        );
    }

    #[test]
    fn compile_enforces_minimum() {
        let result = compile_context("hi");
        // min(max(0, 32), 1) = 1 — capped at raw
        assert_eq!(result.compiled_tokens_estimate, 1);
    }

    #[test]
    fn compile_floor_applies_above_threshold() {
        // 99 words: 99/3 = 33, max(33,32) = 33, min(33,99) = 33
        let input = (0..99)
            .map(|i| format!("w{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let result = compile_context(&input);
        assert_eq!(result.compiled_tokens_estimate, 33);
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
        assert!(result.compiled_context.contains("diff --git"));
        assert!(result.compiled_context.contains("+ new"));
        assert!(!result.compiled_context.contains(" context"));
    }
}
