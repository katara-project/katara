use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub provider: String,
    pub reason: String,
}

pub fn choose_provider(intent: &str, sensitive: bool) -> RouteDecision {
    if sensitive {
        RouteDecision {
            provider: "ollama-local".into(),
            reason: "Sensitive context routed locally.".into(),
        }
    } else if intent == "debug" {
        RouteDecision {
            provider: "mistral-cloud".into(),
            reason: "Debug task routed to mid-tier provider.".into(),
        }
    } else {
        RouteDecision {
            provider: "openai-compatible".into(),
            reason: "Default hybrid route.".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_routes_locally() {
        let decision = choose_provider("general", true);
        assert_eq!(decision.provider, "ollama-local");
    }

    #[test]
    fn debug_routes_to_mistral() {
        let decision = choose_provider("debug", false);
        assert_eq!(decision.provider, "mistral-cloud");
    }

    #[test]
    fn general_routes_to_cloud() {
        let decision = choose_provider("general", false);
        assert_eq!(decision.provider, "openai-compatible");
    }

    #[test]
    fn sensitive_overrides_intent() {
        let decision = choose_provider("debug", true);
        assert_eq!(decision.provider, "ollama-local");
    }
}
