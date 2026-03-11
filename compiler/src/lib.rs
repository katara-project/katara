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
    if lower.contains("error") || lower.contains("trace") {
        "debug".into()
    } else if lower.contains("summar") {
        "summarize".into()
    } else if lower.contains("diff") || lower.contains("pull request") {
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
        let input = (0..120).map(|i| format!("w{i}")).collect::<Vec<_>>().join(" ");
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
        let input = (0..99).map(|i| format!("w{i}")).collect::<Vec<_>>().join(" ");
        let result = compile_context(&input);
        assert_eq!(result.compiled_tokens_estimate, 33);
    }

    #[test]
    fn detect_debug_intent() {
        assert_eq!(detect_intent("stack trace error"), "debug");
    }

    #[test]
    fn detect_summarize_intent() {
        assert_eq!(detect_intent("please summarize this"), "summarize");
    }

    #[test]
    fn detect_review_intent() {
        assert_eq!(detect_intent("review this diff"), "review");
    }

    #[test]
    fn detect_general_intent() {
        assert_eq!(detect_intent("hello world"), "general");
    }
}
