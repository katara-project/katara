use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    pub id: String,
    pub stability: f32,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    pub reused_tokens: usize,
    pub delta_tokens: usize,
    pub context_reuse_ratio: f32,
}

pub fn summarize_memory(raw_tokens: usize) -> MemorySummary {
    let reused_tokens = raw_tokens / 2;
    let delta_tokens = raw_tokens.saturating_sub(reused_tokens);
    let ratio = if raw_tokens == 0 {
        0.0
    } else {
        reused_tokens as f32 / raw_tokens as f32
    };
    MemorySummary {
        reused_tokens,
        delta_tokens,
        context_reuse_ratio: ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_splits_evenly() {
        let summary = summarize_memory(100);
        assert_eq!(summary.reused_tokens, 50);
        assert_eq!(summary.delta_tokens, 50);
        assert!((summary.context_reuse_ratio - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn summarize_zero_tokens() {
        let summary = summarize_memory(0);
        assert_eq!(summary.reused_tokens, 0);
        assert_eq!(summary.delta_tokens, 0);
        assert_eq!(summary.context_reuse_ratio, 0.0);
    }

    #[test]
    fn summarize_odd_tokens() {
        let summary = summarize_memory(101);
        assert_eq!(summary.reused_tokens, 50);
        assert_eq!(summary.delta_tokens, 51);
    }
}
