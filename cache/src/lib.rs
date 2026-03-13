use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub fingerprint: u64,
    pub intent: String,
    pub raw_tokens_estimate: usize,
    pub compiled_tokens_estimate: usize,
    pub summary: String,
    pub compiled_context: String,
}

#[derive(Debug, Default)]
pub struct SemanticCache {
    store: HashMap<u64, CacheEntry>,
}

impl SemanticCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, fingerprint: u64) -> Option<&CacheEntry> {
        self.store.get(&fingerprint)
    }

    pub fn insert(&mut self, entry: CacheEntry) {
        self.store.insert(entry.fingerprint, entry);
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

    #[test]
    fn cache_insert_and_get() {
        let mut cache = SemanticCache::new();
        cache.insert(CacheEntry {
            fingerprint: 42,
            intent: "general".into(),
            raw_tokens_estimate: 100,
            compiled_tokens_estimate: 40,
            summary: "summary".into(),
            compiled_context: "cached response".into(),
        });
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
    fn cache_len() {
        let mut cache = SemanticCache::new();
        assert!(cache.is_empty());
        cache.insert(CacheEntry {
            fingerprint: 1,
            intent: "general".into(),
            raw_tokens_estimate: 1,
            compiled_tokens_estimate: 1,
            summary: "a".into(),
            compiled_context: "a".into(),
        });
        cache.insert(CacheEntry {
            fingerprint: 2,
            intent: "general".into(),
            raw_tokens_estimate: 1,
            compiled_tokens_estimate: 1,
            summary: "b".into(),
            compiled_context: "b".into(),
        });
        assert_eq!(cache.len(), 2);
    }
}
