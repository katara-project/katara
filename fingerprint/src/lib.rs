use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

/// Returns a stable, deterministic u64 fingerprint for the given input.
/// Uses FnvHasher (Fowler-Noll-Vo) which is deterministic across Rust
/// versions and process restarts — unlike std's DefaultHasher.
pub fn fingerprint(input: &str) -> u64 {
    let mut hasher = FnvHasher::default();
    input.trim().to_lowercase().hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_hash() {
        assert_eq!(fingerprint("hello world"), fingerprint("hello world"));
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(fingerprint("Hello World"), fingerprint("hello world"));
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(fingerprint("  hello  "), fingerprint("hello"));
    }

    #[test]
    fn different_input_different_hash() {
        assert_ne!(fingerprint("hello"), fingerprint("world"));
    }
}
