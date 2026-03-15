use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Default TTL: 24 hours. Override with `DISTIRA_CACHE_TTL_SECS` env var.
const DEFAULT_TTL_SECS: u64 = 24 * 60 * 60;

fn cache_ttl_secs() -> u64 {
    std::env::var("DISTIRA_CACHE_TTL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_TTL_SECS)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub fingerprint: u64,
    pub intent: String,
    pub raw_tokens_estimate: usize,
    pub compiled_tokens_estimate: usize,
    pub summary: String,
    pub compiled_context: String,
    /// Unix timestamp (seconds) when this entry was created.
    #[serde(default = "now_secs")]
    pub created_at: u64,
    /// V10.17 — Whether RCT2I prompt restructuring was applied.
    #[serde(default)]
    pub rct2i_applied: bool,
    /// V10.17 — Number of RCT2I sections injected.
    #[serde(default)]
    pub rct2i_sections: u8,
}

#[derive(Debug, Default)]
pub struct SemanticCache {
    store: HashMap<u64, CacheEntry>,
}

impl SemanticCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `None` if the entry is absent or has expired.
    pub fn get(&self, fingerprint: u64) -> Option<&CacheEntry> {
        let entry = self.store.get(&fingerprint)?;
        let age = now_secs().saturating_sub(entry.created_at);
        if age > cache_ttl_secs() {
            return None;
        }
        Some(entry)
    }

    pub fn insert(&mut self, entry: CacheEntry) {
        self.store.insert(entry.fingerprint, entry);
    }

    /// Remove all entries whose age exceeds `ttl_secs`.
    pub fn evict_expired(&mut self, ttl_secs: u64) {
        let cutoff = now_secs().saturating_sub(ttl_secs);
        self.store.retain(|_, e| e.created_at >= cutoff);
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    pub fn entries(&self) -> Vec<CacheEntry> {
        self.store.values().cloned().collect()
    }

    pub fn load_entries(&mut self, entries: Vec<CacheEntry>) {
        self.store.clear();
        for entry in entries {
            self.store.insert(entry.fingerprint, entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_entry(fp: u64, context: &str) -> CacheEntry {
        CacheEntry {
            fingerprint: fp,
            intent: "general".into(),
            raw_tokens_estimate: 100,
            compiled_tokens_estimate: 40,
            summary: "summary".into(),
            compiled_context: context.into(),
            created_at: now_secs(),
            rct2i_applied: false,
            rct2i_sections: 0,
        }
    }

    #[test]
    fn cache_insert_and_get() {
        let mut cache = SemanticCache::new();
        cache.insert(fresh_entry(42, "cached response"));
        let entry = cache.get(42).unwrap();
        assert_eq!(entry.compiled_context, "cached response");
        assert_eq!(entry.intent, "general");
    }

    #[test]
    fn cache_miss() {
        let cache = SemanticCache::new();
        assert!(cache.get(99).is_none());
    }

    #[test]
    fn cache_expired_entry_is_miss() {
        let mut cache = SemanticCache::new();
        cache.insert(CacheEntry {
            fingerprint: 7,
            intent: "general".into(),
            raw_tokens_estimate: 10,
            compiled_tokens_estimate: 5,
            summary: "old".into(),
            compiled_context: "old response".into(),
            // Simulate an entry created 2 days ago
            created_at: now_secs().saturating_sub(DEFAULT_TTL_SECS + 1),
            rct2i_applied: false,
            rct2i_sections: 0,
        });
        // Should be treated as a miss because TTL has passed
        assert!(cache.get(7).is_none());
    }

    #[test]
    fn evict_expired_removes_stale_entries() {
        let mut cache = SemanticCache::new();
        cache.insert(CacheEntry {
            fingerprint: 10,
            intent: "general".into(),
            raw_tokens_estimate: 10,
            compiled_tokens_estimate: 5,
            summary: "old".into(),
            compiled_context: "stale".into(),
            created_at: now_secs().saturating_sub(DEFAULT_TTL_SECS + 1),
            rct2i_applied: false,
            rct2i_sections: 0,
        });
        cache.insert(fresh_entry(20, "fresh"));
        assert_eq!(cache.len(), 2);
        cache.evict_expired(DEFAULT_TTL_SECS);
        assert_eq!(cache.len(), 1);
        assert!(cache.store.get(&20).is_some());
    }

    #[test]
    fn cache_len() {
        let mut cache = SemanticCache::new();
        assert!(cache.is_empty());
        cache.insert(fresh_entry(1, "a"));
        cache.insert(fresh_entry(2, "b"));
        assert_eq!(cache.len(), 2);
    }
}
