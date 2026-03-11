use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub fingerprint: u64,
    pub response: String,
}

#[derive(Debug, Default)]
pub struct SemanticCache {
    store: HashMap<u64, String>,
}

impl SemanticCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, fingerprint: u64) -> Option<&str> {
        self.store.get(&fingerprint).map(String::as_str)
    }

    pub fn insert(&mut self, fingerprint: u64, response: String) {
        self.store.insert(fingerprint, response);
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_insert_and_get() {
        let mut cache = SemanticCache::new();
        cache.insert(42, "cached response".into());
        assert_eq!(cache.get(42), Some("cached response"));
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
        cache.insert(1, "a".into());
        cache.insert(2, "b".into());
        assert_eq!(cache.len(), 2);
    }
}
