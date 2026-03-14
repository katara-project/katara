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
    /// Optional daily request budget.  When set, DISTIRA automatically falls
    /// back to the next available provider once the budget is exhausted.
    /// 0 or absent = unlimited.
    #[serde(default)]
    pub max_requests_per_day: u64,
    /// Quality tier: "low" | "standard" | "high".
    /// Used for quality-aware routing decisions.  Absent = "standard".
    #[serde(default)]
    pub quality_tier: Option<String>,
    /// Per-provider ordered fallback chain (provider keys).
    /// When non-empty, overrides the global fallback sequence
    /// when this provider's daily budget is exhausted.
    #[serde(default)]
    pub fallback_chain: Vec<String>,
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
    /// Inject a conciseness directive into every forwarded LLM request.
    /// Reduces output token usage by instructing the model to be brief.
    concise_mode: Option<bool>,
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
    pub max_requests_per_day: u64,
    pub quality_tier: String,
}

/// Holds the loaded config. Created once at startup, then shared.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    providers: HashMap<String, ProviderConfig>,
    default_provider: String,
    fallback_provider: String,
    sensitive_override: String,
    task_routing: HashMap<String, String>,
    /// When true, DISTIRA injects a response-conciseness directive into every
    /// forwarded LLM request to reduce output token usage.
    concise_mode: bool,
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
            concise_mode: r.concise_mode.unwrap_or(false),
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
                max_requests_per_day: 0,
                quality_tier: Some("standard".into()),
                fallback_chain: vec![],
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
                max_requests_per_day: 0,
                quality_tier: Some("high".into()),
                fallback_chain: vec![],
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
            concise_mode: false,
        }
    }

    /// Route a request based on intent, sensitivity, and per-provider daily budget.
    /// Delegates to `choose_provider_with_budget` with empty counters (unlimited).
    pub fn choose_provider(&self, intent: &str, sensitive: bool) -> RouteDecision {
        self.choose_provider_with_budget(intent, sensitive, &HashMap::new())
    }

    /// Budget-aware routing.  `daily_counts` maps provider key → requests used today.
    /// When a preferred provider's budget is exhausted, falls back through:
    /// task_routing → default → fallback → any available provider.
    pub fn choose_provider_with_budget(
        &self,
        intent: &str,
        sensitive: bool,
        daily_counts: &HashMap<String, u64>,
    ) -> RouteDecision {
        // Sensitive → forced local override (budget limits never apply)
        if sensitive {
            let (name, config) = self.resolve_provider(&self.sensitive_override);
            return RouteDecision {
                provider: name.clone(),
                model: config.model.clone().unwrap_or_default(),
                base_url: config.base_url.clone(),
                reason: format!("Sensitive context → forced to {name} (local)."),
                api_key_env: config.api_key_env.clone(),
            };
        }

        // Build ordered candidate list.
        // If the preferred provider declares a per-provider fallback_chain, use
        // it; otherwise fall through to the global default → fallback sequence.
        let preferred = if let Some(name) = self.task_routing.get(intent) {
            name.clone()
        } else {
            self.default_provider.clone()
        };
        let per_provider_chain: Vec<String> = self
            .providers
            .get(&preferred)
            .map(|cfg| cfg.fallback_chain.clone())
            .unwrap_or_default();
        let mut candidate_keys: Vec<String> = vec![preferred.clone()];
        if per_provider_chain.is_empty() {
            candidate_keys.push(self.default_provider.clone());
            candidate_keys.push(self.fallback_provider.clone());
        } else {
            candidate_keys.extend(per_provider_chain);
            candidate_keys.push(self.fallback_provider.clone());
        }

        let mut seen = std::collections::HashSet::new();
        for candidate in &candidate_keys {
            let candidate = candidate.as_str();
            if !seen.insert(candidate) {
                continue; // skip duplicates
            }
            if let Some(cfg) = self.providers.get(candidate) {
                let budget = cfg.max_requests_per_day;
                let used = daily_counts.get(candidate).copied().unwrap_or(0);
                if budget == 0 || used < budget {
                    let reason = if candidate == preferred && self.task_routing.contains_key(intent)
                    {
                        format!("Intent [{intent}] → routed to {candidate}.")
                    } else if budget > 0 && used >= budget {
                        format!("Budget exhausted → fallback to {candidate}.")
                    } else {
                        format!("Default route → {candidate}.")
                    };
                    return RouteDecision {
                        provider: candidate.to_string(),
                        model: cfg.model.clone().unwrap_or_default(),
                        base_url: cfg.base_url.clone(),
                        reason,
                        api_key_env: cfg.api_key_env.clone(),
                    };
                }
                // budget exhausted — try next candidate
            }
        }

        // All named candidates exhausted — resolve fallback unconditionally
        let (name, config) = self.resolve_provider(&self.fallback_provider);
        RouteDecision {
            provider: name.clone(),
            model: config.model.clone().unwrap_or_default(),
            base_url: config.base_url.clone(),
            reason: format!("All budgets exhausted → fallback to {name}."),
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
                max_requests_per_day: config.max_requests_per_day,
                quality_tier: config
                    .quality_tier
                    .clone()
                    .unwrap_or_else(|| "standard".into()),
            })
            .collect();
        summaries.sort_by(|left, right| left.key.cmp(&right.key));
        summaries
    }

    /// Whether DISTIRA injects a response-conciseness directive into every
    /// forwarded LLM request (reduces output token usage).
    pub fn concise_mode(&self) -> bool {
        self.concise_mode
    }

    /// Latency-aware routing. Like `choose_provider_with_budget` but when
    /// multiple candidates are available (budget not exhausted), the one with
    /// the lowest known average latency is preferred.
    /// `avg_latency` maps provider key → average response time in ms.
    /// Providers with no measured latency are treated as having infinite latency
    /// so that any measured provider wins over an unmeasured one — except when
    /// all candidates are unmeasured, in which case normal order is kept.
    pub fn choose_provider_latency_aware(
        &self,
        intent: &str,
        sensitive: bool,
        daily_counts: &HashMap<String, u64>,
        avg_latency: &HashMap<String, f64>,
    ) -> RouteDecision {
        if sensitive {
            return self.choose_provider_with_budget(intent, sensitive, daily_counts);
        }

        let preferred = if let Some(name) = self.task_routing.get(intent) {
            name.clone()
        } else {
            self.default_provider.clone()
        };

        let per_provider_chain: Vec<String> = self
            .providers
            .get(&preferred)
            .map(|cfg| cfg.fallback_chain.clone())
            .unwrap_or_default();
        let mut candidate_keys: Vec<String> = vec![preferred.clone()];
        if per_provider_chain.is_empty() {
            candidate_keys.push(self.default_provider.clone());
            candidate_keys.push(self.fallback_provider.clone());
        } else {
            candidate_keys.extend(per_provider_chain);
            candidate_keys.push(self.fallback_provider.clone());
        }

        // Collect all available (within-budget) candidates.
        let mut seen = std::collections::HashSet::new();
        let mut available: Vec<(&str, &ProviderConfig, f64)> = Vec::new();
        for key in &candidate_keys {
            let key = key.as_str();
            if !seen.insert(key) {
                continue;
            }
            if let Some(cfg) = self.providers.get(key) {
                let budget = cfg.max_requests_per_day;
                let used = daily_counts.get(key).copied().unwrap_or(0);
                if budget == 0 || used < budget {
                    let lat = avg_latency.get(key).copied().unwrap_or(f64::MAX);
                    available.push((key, cfg, lat));
                }
            }
        }

        if let Some((key, cfg, _)) = available
            .iter()
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        {
            // Only prefer by latency if we actually have a measured value
            // for the fastest candidate (not f64::MAX placeholder).
            let all_unmeasured = available.iter().all(|(_, _, lat)| *lat == f64::MAX);
            let chosen = if all_unmeasured {
                // Fall back to normal priority order (first available).
                available[0].0
            } else {
                key
            };
            let cfg = self.providers.get(chosen).unwrap_or(cfg);
            let reason = if chosen == preferred && self.task_routing.contains_key(intent) {
                format!("Intent [{intent}] → routed to {chosen}.")
            } else if avg_latency.contains_key(chosen) {
                format!(
                    "Latency-aware → {chosen} ({:.0}ms avg).",
                    avg_latency[chosen]
                )
            } else {
                format!("Default route → {chosen}.")
            };
            return RouteDecision {
                provider: chosen.to_string(),
                model: cfg.model.clone().unwrap_or_default(),
                base_url: cfg.base_url.clone(),
                reason,
                api_key_env: cfg.api_key_env.clone(),
            };
        }

        // All budgets exhausted.
        let (name, config) = self.resolve_provider(&self.fallback_provider);
        RouteDecision {
            provider: name.clone(),
            model: config.model.clone().unwrap_or_default(),
            base_url: config.base_url.clone(),
            reason: format!("All budgets exhausted → fallback to {name}."),
            api_key_env: config.api_key_env.clone(),
        }
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

    /// V10 — Adaptive routing: picks the provider with the lowest composite
    /// score = avg_latency × (1 + error_rate × 5). Unmeasured providers get
    /// f64::MAX and fall back to priority order; sensitive override bypasses.
    pub fn choose_provider_adaptive(
        &self,
        intent: &str,
        sensitive: bool,
        daily_counts: &HashMap<String, u64>,
        avg_latency: &HashMap<String, f64>,
        error_rates: &HashMap<String, f64>,
    ) -> RouteDecision {
        if sensitive {
            return self.choose_provider_with_budget(intent, sensitive, daily_counts);
        }

        let preferred = if let Some(name) = self.task_routing.get(intent) {
            name.clone()
        } else {
            self.default_provider.clone()
        };

        let per_provider_chain: Vec<String> = self
            .providers
            .get(&preferred)
            .map(|cfg| cfg.fallback_chain.clone())
            .unwrap_or_default();
        let mut candidate_keys: Vec<String> = vec![preferred.clone()];
        if per_provider_chain.is_empty() {
            candidate_keys.push(self.default_provider.clone());
            candidate_keys.push(self.fallback_provider.clone());
        } else {
            candidate_keys.extend(per_provider_chain);
            candidate_keys.push(self.fallback_provider.clone());
        }

        // Collect within-budget candidates with composite adaptive score.
        let mut seen = std::collections::HashSet::new();
        let mut available: Vec<(&str, &ProviderConfig, f64)> = Vec::new();
        for key in &candidate_keys {
            let key = key.as_str();
            if !seen.insert(key) {
                continue;
            }
            if let Some(cfg) = self.providers.get(key) {
                let budget = cfg.max_requests_per_day;
                let used = daily_counts.get(key).copied().unwrap_or(0);
                if budget == 0 || used < budget {
                    let lat = avg_latency.get(key).copied().unwrap_or(f64::MAX);
                    let err = error_rates.get(key).copied().unwrap_or(0.0);
                    // Composite: penalise error rate; unmeasured stays MAX.
                    let score = if lat == f64::MAX {
                        f64::MAX
                    } else {
                        lat * (1.0 + err * 5.0)
                    };
                    available.push((key, cfg, score));
                }
            }
        }

        if let Some((key, cfg, _)) = available
            .iter()
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        {
            let all_unmeasured = available.iter().all(|(_, _, s)| *s == f64::MAX);
            let chosen = if all_unmeasured { available[0].0 } else { key };
            let cfg = self.providers.get(chosen).unwrap_or(cfg);
            let reason = if chosen == preferred && self.task_routing.contains_key(intent) {
                format!("Intent [{intent}] → routed to {chosen}.")
            } else {
                let lat = avg_latency.get(chosen).copied().unwrap_or(0.0);
                let err = error_rates.get(chosen).copied().unwrap_or(0.0);
                if lat > 0.0 || err > 0.0 {
                    format!(
                        "Adaptive → {chosen} ({:.0}ms, {:.0}% err).",
                        lat,
                        err * 100.0
                    )
                } else {
                    format!("Default route → {chosen}.")
                }
            };
            return RouteDecision {
                provider: chosen.to_string(),
                model: cfg.model.clone().unwrap_or_default(),
                base_url: cfg.base_url.clone(),
                reason,
                api_key_env: cfg.api_key_env.clone(),
            };
        }

        // All budgets exhausted.
        let (name, config) = self.resolve_provider(&self.fallback_provider);
        RouteDecision {
            provider: name.clone(),
            model: config.model.clone().unwrap_or_default(),
            base_url: config.base_url.clone(),
            reason: format!("All budgets exhausted → fallback to {name}."),
            api_key_env: config.api_key_env.clone(),
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

    #[test]
    fn latency_aware_returns_valid_decision() {
        let cfg = RouterConfig::defaults();
        // No latency data → falls back to normal priority order.
        let d =
            cfg.choose_provider_latency_aware("general", false, &HashMap::new(), &HashMap::new());
        assert!(!d.provider.is_empty());
        assert!(d.base_url.starts_with("http"));
    }

    #[test]
    fn latency_aware_prefers_faster_provider() {
        let cfg = RouterConfig::defaults();
        let daily_counts = HashMap::new();
        // Assign higher latency to the preferred provider and lower to the fallback.
        let mut latency: HashMap<String, f64> = HashMap::new();
        latency.insert("ollama-local".into(), 500.0); // slow
        latency.insert("openai-compatible".into(), 50.0); // fast
        let d = cfg.choose_provider_latency_aware("general", false, &daily_counts, &latency);
        // Should route to openai-compatible (faster) even though ollama-local is preferred by intent.
        assert_eq!(d.provider, "openai-compatible");
        assert!(d.reason.contains("Latency-aware"));
    }

    #[test]
    fn latency_aware_sensitive_ignores_latency() {
        let cfg = RouterConfig::defaults();
        let mut latency: HashMap<String, f64> = HashMap::new();
        latency.insert("openai-compatible".into(), 1.0); // fastest
        let d = cfg.choose_provider_latency_aware("general", true, &HashMap::new(), &latency);
        // Sensitive: must always be local regardless of latency.
        assert_eq!(d.provider, "ollama-local");
        assert!(d.reason.contains("Sensitive"));
    }

    // ── V10 adaptive routing tests ────────────────────────────────────────────

    #[test]
    fn adaptive_returns_valid_decision() {
        let cfg = RouterConfig::defaults();
        // No data → should still return a valid decision.
        let d = cfg.choose_provider_adaptive(
            "general",
            false,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(!d.provider.is_empty());
        assert!(d.base_url.starts_with("http"));
    }

    #[test]
    fn adaptive_penalizes_high_error_rate() {
        let cfg = RouterConfig::defaults();
        let daily_counts = HashMap::new();
        // ollama-local is fast but has 80% error rate.
        let mut latency: HashMap<String, f64> = HashMap::new();
        latency.insert("ollama-local".into(), 50.0); // fast
        latency.insert("openai-compatible".into(), 200.0); // slower
        let mut errors: HashMap<String, f64> = HashMap::new();
        errors.insert("ollama-local".into(), 0.80); // 80% error
        errors.insert("openai-compatible".into(), 0.0);
        let d = cfg.choose_provider_adaptive("general", false, &daily_counts, &latency, &errors);
        // composite(ollama-local) = 50 * (1 + 0.80*5) = 50 * 5 = 250 > 200 → openai-compatible wins.
        assert_eq!(d.provider, "openai-compatible");
        assert!(d.reason.contains("Adaptive"));
    }

    #[test]
    fn adaptive_sensitive_ignores_scores() {
        let cfg = RouterConfig::defaults();
        let mut latency: HashMap<String, f64> = HashMap::new();
        latency.insert("openai-compatible".into(), 1.0); // fastest
        let d = cfg.choose_provider_adaptive(
            "general",
            true,
            &HashMap::new(),
            &latency,
            &HashMap::new(),
        );
        // Sensitive: must always be local regardless of scores.
        assert_eq!(d.provider, "ollama-local");
        assert!(d.reason.contains("Sensitive"));
    }
}
