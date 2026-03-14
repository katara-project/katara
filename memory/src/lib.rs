use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Minimum stability before a block is evicted (V9.13).
const DECAY_EVICT_THRESHOLD: f32 = 0.10;
/// Decay multiplier applied to non-accessed blocks on each register call (V9.13).
const DECAY_FACTOR: f32 = 0.92;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    pub id: String,
    pub stability: f32,
    pub content: String,
    /// Intent the block was compiled for (V9.13 — intent-scoped injection).
    #[serde(default)]
    pub intent: String,
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

    /// Store a compiled context block keyed by fingerprint (V9.13: with intent).
    /// Decays all other blocks' stability and evicts those below the threshold.
    pub fn register(&mut self, fingerprint: u64, compiled: &str, intent: &str) {
        // Decay all existing blocks except the one being registered.
        let keys: Vec<u64> = self.blocks.keys().copied().collect();
        for key in keys {
            if key == fingerprint {
                continue;
            }
            if let Some(block) = self.blocks.get_mut(&key) {
                block.stability *= DECAY_FACTOR;
            }
        }
        // Evict blocks that have decayed below the threshold.
        self.blocks
            .retain(|_, b| b.stability >= DECAY_EVICT_THRESHOLD);

        // Upsert the current block with full stability.
        self.blocks.insert(
            fingerprint,
            ContextBlock {
                id: fingerprint.to_string(),
                stability: 1.0,
                content: compiled.to_string(),
                intent: intent.to_string(),
            },
        );
    }

    /// Return reuse stats for an exact fingerprint hit (V9.13: intent-scoped).
    /// Returns zero reuse if the block exists but was compiled for a different intent.
    pub fn compute_reuse(
        &self,
        fingerprint: u64,
        raw_tokens: usize,
        intent: &str,
    ) -> MemorySummary {
        if let Some(block) = self.blocks.get(&fingerprint) {
            // Intent-scope guard: only reuse if intent matches or block has no intent tag.
            if !block.intent.is_empty() && block.intent != intent {
                return MemorySummary {
                    reused_tokens: 0,
                    delta_tokens: raw_tokens,
                    context_reuse_ratio: 0.0,
                };
            }
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
                // Evict already-decayed blocks that survived serialization
                if block.stability >= DECAY_EVICT_THRESHOLD {
                    self.blocks.insert(fingerprint, block);
                }
            }
        }
    }

    /// Estimate how many tokens of `compiled` are covered by prior stable blocks
    /// of the same intent (lexical word-set intersection).
    ///
    /// Used in the compile handler on a cache **miss** to credit partial reuse:
    /// if the system has already compiled similar content (same intent, shared
    /// vocabulary) those overlapping tokens represent knowledge the LLM has
    /// already seen — they are genuinely "reused" context.
    pub fn estimate_coverage(&self, compiled: &str, intent: &str) -> MemorySummary {
        if self.blocks.is_empty() || compiled.trim().is_empty() {
            return MemorySummary {
                reused_tokens: 0,
                delta_tokens: token_count(compiled),
                context_reuse_ratio: 0.0,
            };
        }

        let compiled_words: HashSet<&str> = compiled.split_whitespace().collect();
        let total = compiled_words.len();
        if total == 0 {
            return MemorySummary {
                reused_tokens: 0,
                delta_tokens: 0,
                context_reuse_ratio: 0.0,
            };
        }

        // Union of all words seen in stable blocks of the same (or untagged) intent.
        let known_words: HashSet<&str> = self
            .blocks
            .values()
            .filter(|b| b.intent.is_empty() || b.intent == intent)
            .flat_map(|b| b.content.split_whitespace())
            .collect();

        let covered = compiled_words.intersection(&known_words).count();
        let delta = total.saturating_sub(covered);
        let ratio = (covered as f32 / total as f32).min(1.0);

        MemorySummary {
            reused_tokens: covered,
            delta_tokens: delta,
            context_reuse_ratio: ratio,
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

/// Delta-forwarding helper — the core of Context Memory Lensing.
///
/// In a multi-turn chat flow, `prior_tokens` are already resident in the LLM's
/// context window from previous turns: they are **reused** without being
/// re-compiled or re-sent at cost.  `new_tokens` is the genuine delta — the
/// latest user turn that is actually new this request.
///
/// This produces a non-zero `context_reuse_ratio` for any multi-turn session,
/// making the Memory Lensing gain immediately visible in the dashboard.
pub fn compute_delta(prior_tokens: usize, new_tokens: usize) -> MemorySummary {
    let total = prior_tokens + new_tokens;
    let ratio = if total > 0 {
        (prior_tokens as f32 / total as f32).min(1.0)
    } else {
        0.0
    };
    MemorySummary {
        reused_tokens: prior_tokens,
        delta_tokens: new_tokens,
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
        store.register(42, "hello world compiled", "general");

        let reuse = store.compute_reuse(42, 10, "general");
        // compiled has 3 tokens, raw=10 → reused=3, delta=7
        assert_eq!(reuse.reused_tokens, 3);
        assert_eq!(reuse.delta_tokens, 7);
        assert!((reuse.context_reuse_ratio - 0.3).abs() < 0.01);
    }

    #[test]
    fn context_store_miss_returns_zero() {
        let store = ContextStore::new();
        let reuse = store.compute_reuse(999, 50, "general");
        assert_eq!(reuse.reused_tokens, 0);
        assert_eq!(reuse.delta_tokens, 50);
        assert_eq!(reuse.context_reuse_ratio, 0.0);
    }

    #[test]
    fn context_store_caps_reuse_at_raw() {
        let mut store = ContextStore::new();
        // compiled has more tokens than raw — reuse capped at raw
        store.register(1, "alpha beta gamma delta epsilon", "debug");
        let reuse = store.compute_reuse(1, 3, "debug"); // raw=3, compiled=5
        assert_eq!(reuse.reused_tokens, 3);
        assert_eq!(reuse.delta_tokens, 0);
        assert!((reuse.context_reuse_ratio - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn context_store_intent_mismatch_returns_zero() {
        let mut store = ContextStore::new();
        store.register(10, "compiled debug content", "debug");
        // Same fingerprint but different intent → no reuse
        let reuse = store.compute_reuse(10, 20, "general");
        assert_eq!(reuse.reused_tokens, 0);
        assert_eq!(reuse.delta_tokens, 20);
    }

    #[test]
    fn context_store_stability_decays_on_register() {
        let mut store = ContextStore::new();
        store.register(1, "block one", "general");
        // Register a second block — first should decay
        store.register(2, "block two", "general");
        let block1 = store.blocks.get(&1).unwrap();
        assert!(block1.stability < 1.0, "block1 should have decayed");
        assert!((block1.stability - DECAY_FACTOR).abs() < 0.001);
    }

    #[test]
    fn context_store_evicts_fully_decayed_blocks() {
        let mut store = ContextStore::new();
        store.register(1, "old block", "general");
        // Register enough new blocks to decay block 1 below threshold
        // After N registers: stability = DECAY_FACTOR^N
        // DECAY_FACTOR=0.92, threshold=0.10 → need ~28 registers
        for i in 2..35u64 {
            store.register(i, "newer block", "general");
        }
        assert!(
            !store.blocks.contains_key(&1),
            "fully decayed block should be evicted"
        );
    }

    // ── compute_delta (V9.0 Memory Lensing) ────────────────────────────────

    #[test]
    fn compute_delta_multi_turn_gives_prior_as_reused() {
        // 5-turn conversation: 80 prior tokens, 20 new → 80% reuse
        let summary = compute_delta(80, 20);
        assert_eq!(summary.reused_tokens, 80);
        assert_eq!(summary.delta_tokens, 20);
        assert!((summary.context_reuse_ratio - 0.8).abs() < 0.001);
    }

    #[test]
    fn compute_delta_first_turn_zero_reuse() {
        // First message: no prior context → reuse = 0
        let summary = compute_delta(0, 50);
        assert_eq!(summary.reused_tokens, 0);
        assert_eq!(summary.delta_tokens, 50);
        assert_eq!(summary.context_reuse_ratio, 0.0);
    }

    #[test]
    fn compute_delta_empty_is_zero() {
        let summary = compute_delta(0, 0);
        assert_eq!(summary.reused_tokens, 0);
        assert_eq!(summary.delta_tokens, 0);
        assert_eq!(summary.context_reuse_ratio, 0.0);
    }

    #[test]
    fn compute_delta_ratio_correct() {
        // 3-turn session: system(20) + assistant(30) + prior user(50) = 100 prior, 25 new
        let summary = compute_delta(100, 25);
        assert_eq!(summary.reused_tokens, 100);
        assert_eq!(summary.delta_tokens, 25);
        assert!((summary.context_reuse_ratio - (100.0_f32 / 125.0)).abs() < 0.001);
    }

    // ── estimate_coverage (V10.1 — partial reuse on cache miss) ─────────────

    #[test]
    fn estimate_coverage_empty_store_returns_zero() {
        let store = ContextStore::new();
        let summary = store.estimate_coverage("some compiled output here", "general");
        assert_eq!(summary.reused_tokens, 0);
    }

    #[test]
    fn estimate_coverage_full_overlap_returns_all() {
        let mut store = ContextStore::new();
        store.register(1, "alpha beta gamma", "general");
        // Compiled context uses exact same words → 100 % coverage
        let summary = store.estimate_coverage("alpha beta gamma", "general");
        assert_eq!(summary.reused_tokens, 3);
        assert_eq!(summary.delta_tokens, 0);
        assert!((summary.context_reuse_ratio - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn estimate_coverage_partial_overlap() {
        let mut store = ContextStore::new();
        store.register(1, "alpha beta gamma delta", "general");
        // 2 of 4 words are new
        let summary = store.estimate_coverage("alpha beta epsilon zeta", "general");
        assert_eq!(summary.reused_tokens, 2);
        assert_eq!(summary.delta_tokens, 2);
        assert!((summary.context_reuse_ratio - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn estimate_coverage_intent_mismatch_ignored() {
        let mut store = ContextStore::new();
        // Block registered under "debug" intent — should not contribute to "general"
        store.register(1, "alpha beta gamma", "debug");
        let summary = store.estimate_coverage("alpha beta gamma", "general");
        assert_eq!(summary.reused_tokens, 0);
    }
}
