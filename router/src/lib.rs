use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ── Config structs (deserialized from YAML) ────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    #[serde(rename = "type")]
    pub provider_type: Option<String>,
    pub base_url: String,
    pub model: Option<String>,
    pub deployment: Option<String>,
    pub description: Option<String>,
    pub api_key_env: Option<String>,
    /// Cost in USD per 1 000 input tokens (0.0 for on-prem).
    #[serde(default)]
    pub cost_per_1k_input_tokens: f64,
    /// Cost in USD per 1 000 output tokens (0.0 for on-prem).
    #[serde(default)]
    pub cost_per_1k_output_tokens: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct ProvidersFile {
    providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct TaskRouting {
    debug: Option<String>,
    summarize: Option<String>,
    review: Option<String>,
    codegen: Option<String>, // was missing — caused codegen to fall through to default
    translate: Option<String>,
    general: Option<String>,
    ocr: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RoutingInner {
    default_provider: Option<String>,
    fallback_provider: Option<String>,
    sensitive_override: Option<String>,
    task_routing: Option<TaskRouting>,
}

#[derive(Debug, Clone, Deserialize)]
struct RoutingFile {
    routing: RoutingInner,
}

// ── Public types ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub reason: String,
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummary {
    pub key: String,
    pub provider_type: String,
    pub model: String,
    pub deployment: String,
    pub base_url: String,
    pub description: String,
    pub api_key_env: Option<String>,
    pub cost_per_1k_input_tokens: f64,
    pub cost_per_1k_output_tokens: f64,
}

/// Holds the loaded config. Created once at startup, then shared.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    providers: HashMap<String, ProviderConfig>,
    default_provider: String,
    fallback_provider: String,
    sensitive_override: String,
    task_routing: HashMap<String, String>,
}

impl RouterConfig {
    /// Load from YAML files on disk.
    pub fn load(providers_path: &Path, routing_path: &Path) -> Result<Self, String> {
        let prov_str = std::fs::read_to_string(providers_path)
            .map_err(|e| format!("Cannot read {}: {e}", providers_path.display()))?;
        let prov_file: ProvidersFile =
            serde_yaml::from_str(&prov_str).map_err(|e| format!("Invalid providers YAML: {e}"))?;

        let rout_str = std::fs::read_to_string(routing_path)
            .map_err(|e| format!("Cannot read {}: {e}", routing_path.display()))?;
        let rout_file: RoutingFile =
            serde_yaml::from_str(&rout_str).map_err(|e| format!("Invalid routing YAML: {e}"))?;

        let r = &rout_file.routing;
        let default = r.default_provider.clone().unwrap_or("ollama-local".into());
        let fallback = r.fallback_provider.clone().unwrap_or(default.clone());
        let sensitive = r.sensitive_override.clone().unwrap_or(default.clone());

        let mut task_map = HashMap::new();
        if let Some(tr) = &r.task_routing {
            if let Some(v) = &tr.debug {
                task_map.insert("debug".into(), v.clone());
            }
            if let Some(v) = &tr.summarize {
                task_map.insert("summarize".into(), v.clone());
            }
            if let Some(v) = &tr.review {
                task_map.insert("review".into(), v.clone());
            }
            if let Some(v) = &tr.codegen {
                task_map.insert("codegen".into(), v.clone());
            }
            if let Some(v) = &tr.translate {
                task_map.insert("translate".into(), v.clone());
            }
            if let Some(v) = &tr.general {
                task_map.insert("general".into(), v.clone());
            }
            if let Some(v) = &tr.ocr {
                task_map.insert("ocr".into(), v.clone());
            }
        }

        Ok(Self {
            providers: prov_file.providers,
            default_provider: default,
            fallback_provider: fallback,
            sensitive_override: sensitive,
            task_routing: task_map,
        })
    }

    /// Build from inline defaults (no files needed — for tests & fallback).
    pub fn defaults() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "ollama-local".into(),
            ProviderConfig {
                provider_type: Some("openai-compatible".into()),
                base_url: "http://localhost:11434/v1".into(),
                model: Some("llama3.1".into()),
                deployment: Some("on-prem".into()),
                description: Some("Local Ollama".into()),
                api_key_env: None,
                cost_per_1k_input_tokens: 0.0,
                cost_per_1k_output_tokens: 0.0,
            },
        );
        providers.insert(
            "openai-compatible".into(),
            ProviderConfig {
                provider_type: Some("openai-compatible".into()),
                base_url: "https://api.openai.com/v1".into(),
                model: Some("gpt-4o-mini".into()),
                deployment: Some("cloud".into()),
                description: Some("OpenAI cloud".into()),
                api_key_env: Some("OPENAI_API_KEY".into()),
                cost_per_1k_input_tokens: 0.15,
                cost_per_1k_output_tokens: 0.60,
            },
        );

        let mut task_routing = HashMap::new();
        task_routing.insert("debug".into(), "ollama-local".into());
        task_routing.insert("general".into(), "ollama-local".into());

        Self {
            providers,
            default_provider: "ollama-local".into(),
            fallback_provider: "openai-compatible".into(),
            sensitive_override: "ollama-local".into(),
            task_routing,
        }
    }

    /// Route a request based on intent and sensitivity.
    pub fn choose_provider(&self, intent: &str, sensitive: bool) -> RouteDecision {
        // 1. Sensitive → forced local override
        let provider_name = if sensitive {
            self.sensitive_override.clone()
        } else if let Some(name) = self.task_routing.get(intent) {
            name.clone()
        } else {
            self.default_provider.clone()
        };

        // 2. Resolve provider config (fallback if missing)
        let (name, config) = self.resolve_provider(&provider_name);

        let reason = if sensitive {
            format!("Sensitive context → forced to {name} (local).")
        } else if self.task_routing.contains_key(intent) {
            format!("Intent [{intent}] → routed to {name}.")
        } else {
            format!("Default route → {name}.")
        };

        RouteDecision {
            provider: name,
            model: config.model.clone().unwrap_or_default(),
            base_url: config.base_url.clone(),
            reason,
            api_key_env: config.api_key_env.clone(),
        }
    }

    fn resolve_provider(&self, name: &str) -> (String, &ProviderConfig) {
        if let Some(cfg) = self.providers.get(name) {
            (name.to_string(), cfg)
        } else if let Some(cfg) = self.providers.get(&self.fallback_provider) {
            (self.fallback_provider.clone(), cfg)
        } else {
            // Ultimate fallback — first available provider
            let (k, v) = self
                .providers
                .iter()
                .next()
                .expect("No providers configured");
            (k.clone(), v)
        }
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn list_provider_summaries(&self) -> Vec<ProviderSummary> {
        let mut summaries: Vec<_> = self
            .providers
            .iter()
            .map(|(key, config)| ProviderSummary {
                key: key.clone(),
                provider_type: config
                    .provider_type
                    .clone()
                    .unwrap_or_else(|| "unknown".into()),
                model: config.model.clone().unwrap_or_default(),
                deployment: config
                    .deployment
                    .clone()
                    .unwrap_or_else(|| "unknown".into()),
                base_url: config.base_url.clone(),
                description: config.description.clone().unwrap_or_default(),
                api_key_env: config.api_key_env.clone(),
                cost_per_1k_input_tokens: config.cost_per_1k_input_tokens,
                cost_per_1k_output_tokens: config.cost_per_1k_output_tokens,
            })
            .collect();
        summaries.sort_by(|left, right| left.key.cmp(&right.key));
        summaries
    }

    /// Estimate the USD cost of a request for the given provider.
    /// `input_tokens` and `output_tokens` should be the estimated token counts.
    pub fn cost_estimate_usd(
        &self,
        provider_name: &str,
        input_tokens: usize,
        output_tokens: usize,
    ) -> f64 {
        let cfg = self.providers.get(provider_name);
        if let Some(c) = cfg {
            (input_tokens as f64 / 1000.0) * c.cost_per_1k_input_tokens
                + (output_tokens as f64 / 1000.0) * c.cost_per_1k_output_tokens
        } else {
            0.0
        }
    }
}

// ── Legacy compat — keep simple function for basic usage ──

pub fn choose_provider(intent: &str, sensitive: bool) -> RouteDecision {
    RouterConfig::defaults().choose_provider(intent, sensitive)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_routes_locally() {
        let cfg = RouterConfig::defaults();
        let d = cfg.choose_provider("general", true);
        assert_eq!(d.provider, "ollama-local");
        assert!(!d.model.is_empty());
    }

    #[test]
    fn debug_routes_by_task() {
        let cfg = RouterConfig::defaults();
        let d = cfg.choose_provider("debug", false);
        assert_eq!(d.provider, "ollama-local");
    }

    #[test]
    fn general_routes_to_default() {
        let cfg = RouterConfig::defaults();
        let d = cfg.choose_provider("general", false);
        assert_eq!(d.provider, "ollama-local");
    }

    #[test]
    fn sensitive_overrides_intent() {
        let cfg = RouterConfig::defaults();
        let d = cfg.choose_provider("debug", true);
        assert_eq!(d.provider, "ollama-local");
        assert!(d.reason.contains("Sensitive"));
    }

    #[test]
    fn unknown_intent_gets_default() {
        let cfg = RouterConfig::defaults();
        let d = cfg.choose_provider("unknown-task", false);
        assert_eq!(d.provider, "ollama-local");
        assert!(d.reason.contains("Default"));
    }

    #[test]
    fn route_decision_has_base_url() {
        let cfg = RouterConfig::defaults();
        let d = cfg.choose_provider("general", false);
        assert!(d.base_url.starts_with("http"));
    }

    #[test]
    fn legacy_fn_still_works() {
        let d = choose_provider("general", false);
        assert!(!d.provider.is_empty());
        assert!(!d.base_url.is_empty());
    }

    #[test]
    fn provider_summaries_include_models() {
        let cfg = RouterConfig::defaults();
        let summaries = cfg.list_provider_summaries();
        assert!(!summaries.is_empty());
        assert!(summaries.iter().any(|summary| !summary.model.is_empty()));
    }
}
