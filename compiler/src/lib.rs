use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub intent: String,
    pub raw_tokens_estimate: usize,
    pub compiled_tokens_estimate: usize,
    pub summary: String,
}

pub fn compile_context(raw: &str) -> CompileResult {
    let raw_tokens_estimate = raw.split_whitespace().count();
    let compiled_tokens_estimate = raw_tokens_estimate
        .saturating_div(3)
        .max(32)
        .min(raw_tokens_estimate);
    CompileResult {
        intent: detect_intent(raw),
        raw_tokens_estimate,
        compiled_tokens_estimate,
        summary: "Reduce noise, deduplicate repeated sections, retain minimal useful context."
            .into(),
    }
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
    } else if lower.contains("summar")
        || lower.contains("explain")
        || lower.contains("tldr")
        || lower.contains("recap")
    {
        "summarize".into()
    } else if lower.contains("diff")
        || lower.contains("pull request")
        || lower.contains("review")
        || lower.contains("code review")
        || lower.contains("refactor")
    {
        "review".into()
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
    fn detect_general_intent() {
        assert_eq!(detect_intent("hello world"), "general");
    }
}
