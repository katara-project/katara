use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn fingerprint(input: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
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
