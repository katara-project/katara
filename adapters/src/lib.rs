use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adapter {
    pub name: &'static str,
    pub api_style: &'static str,
}

pub const ADAPTERS: &[Adapter] = &[
    Adapter { name: "ollama", api_style: "openai-compatible" },
    Adapter { name: "openai-compatible", api_style: "openai-compatible" },
    Adapter { name: "mistral", api_style: "openai-compatible" },
    Adapter { name: "gemini", api_style: "google" },
];

pub fn supported_adapters() -> Vec<&'static str> {
    ADAPTERS.iter().map(|a| a.name).collect()
}

pub fn find_adapter(name: &str) -> Option<&'static Adapter> {
    ADAPTERS.iter().find(|a| a.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_all_adapters() {
        let names = supported_adapters();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"ollama"));
        assert!(names.contains(&"gemini"));
    }

    #[test]
    fn find_existing_adapter() {
        let adapter = find_adapter("mistral");
        assert!(adapter.is_some());
        assert_eq!(adapter.unwrap().api_style, "openai-compatible");
    }

    #[test]
    fn find_missing_adapter() {
        assert!(find_adapter("unknown").is_none());
    }
}
