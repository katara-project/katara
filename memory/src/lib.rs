use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

fn token_count(s: &str) -> usize {
    s.split_whitespace().count()
}

/// In-memory context block store keyed by semantic fingerprint.
/// Registers compiled contexts and computes real token-reuse ratios.
#[derive(Debug, Default)]
pub struct ContextStore {
    blocks: HashMap<u64, ContextBlock>,
}

impl ContextStore {
    pub fn new() -> Self {
        ContextStore {
            blocks: HashMap::new(),
        }
    }

    /// Store a compiled context block keyed by fingerprint.
    pub fn register(&mut self, fingerprint: u64, compiled: &str) {
        self.blocks.insert(
            fingerprint,
            ContextBlock {
                id: fingerprint.to_string(),
                stability: 1.0,
                content: compiled.to_string(),
            },
        );
    }

    /// Return real reuse stats: if fingerprint matches a stored block, those
    /// compiled tokens are considered reused; otherwise zero reuse.
    pub fn compute_reuse(&self, fingerprint: u64, raw_tokens: usize) -> MemorySummary {
        if let Some(block) = self.blocks.get(&fingerprint) {
            let reused = token_count(&block.content).min(raw_tokens);
            let delta = raw_tokens.saturating_sub(reused);
            let ratio = if raw_tokens > 0 {
                (reused as f32 / raw_tokens as f32).min(1.0)
            } else {
                0.0
            };
            MemorySummary {
                reused_tokens: reused,
                delta_tokens: delta,
                context_reuse_ratio: ratio,
            }
        } else {
            MemorySummary {
                reused_tokens: 0,
                delta_tokens: raw_tokens,
                context_reuse_ratio: 0.0,
            }
        }
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn blocks(&self) -> Vec<ContextBlock> {
        self.blocks.values().cloned().collect()
    }

    pub fn load_blocks(&mut self, blocks: Vec<ContextBlock>) {
        self.blocks.clear();
        for block in blocks {
            if let Ok(fingerprint) = block.id.parse::<u64>() {
                self.blocks.insert(fingerprint, block);
            }
        }
    }
}

/// Lightweight standalone helper — kept for cases where a fingerprint is unavailable.
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

    #[test]
    fn context_store_registers_and_computes_reuse() {
        let mut store = ContextStore::new();
        store.register(42, "hello world compiled");

        let reuse = store.compute_reuse(42, 10);
        // compiled has 3 tokens, raw=10 → reused=3, delta=7
        assert_eq!(reuse.reused_tokens, 3);
        assert_eq!(reuse.delta_tokens, 7);
        assert!((reuse.context_reuse_ratio - 0.3).abs() < 0.01);
    }

    #[test]
    fn context_store_miss_returns_zero() {
        let store = ContextStore::new();
        let reuse = store.compute_reuse(999, 50);
        assert_eq!(reuse.reused_tokens, 0);
        assert_eq!(reuse.delta_tokens, 50);
        assert_eq!(reuse.context_reuse_ratio, 0.0);
    }

    #[test]
    fn context_store_caps_reuse_at_raw() {
        let mut store = ContextStore::new();
        // compiled has more tokens than raw — reuse capped at raw
        store.register(1, "alpha beta gamma delta epsilon");
        let reuse = store.compute_reuse(1, 3); // raw=3, compiled=5
        assert_eq!(reuse.reused_tokens, 3);
        assert_eq!(reuse.delta_tokens, 0);
        assert!((reuse.context_reuse_ratio - 1.0).abs() < f32::EPSILON);
    }
}
