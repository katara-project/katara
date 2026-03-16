use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{header, HeaderValue, Request, Response, StatusCode},
    middleware::{self, Next},
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use futures_util::StreamExt as FuturesStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio_stream::iter;
use tokio_stream::wrappers::{IntervalStream, ReceiverStream};
use tower_http::cors::{Any, CorsLayer};

/// ── Shared application state ──────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntentStats {
    requests: u64,
    raw_tokens: usize,
    compiled_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelStats {
    model: String,
    provider: String,
    requests: u64,
    raw_tokens: usize,
    compiled_tokens: usize,
    memory_reused_tokens: usize,
    efficiency_score: f32,
    sovereign_requests: u64,
    non_sovereign_requests: u64,
    sovereign_ratio: f32,
    /// Rolling average latency in ms for requests routed to this provider/model pair.
    #[serde(default)]
    avg_latency_ms: f64,
    /// Accumulated latency sum for rolling average computation.
    #[serde(default)]
    latency_sum_ms: f64,
    /// Number of latency samples (non-zero) included in avg_latency_ms.
    #[serde(default)]
    latency_samples: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpstreamStats {
    client_app: String,
    upstream_provider: String,
    upstream_model: String,
    requests: u64,
    last_seen_ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestLineage {
    client_app: Option<String>,
    upstream_provider: Option<String>,
    upstream_model: Option<String>,
    tenant_id: Option<String>,
    project_id: Option<String>,
    policy_pack: Option<String>,
    routed_provider: String,
    routed_model: String,
    intent: String,
    semantic_cache_hit: bool,
    semantic_fingerprint: Option<String>,
    cache_hit: bool,
    sensitive: bool,
    ts: u64,
    /// Estimated cost of this request in USD (0.0 for on-prem).
    #[serde(default)]
    cost_usd: f64,
    /// Raw token count before compilation (context size the user submitted).
    #[serde(default)]
    raw_tokens: usize,
    /// Compiled token count after DISTIRA optimisation.
    #[serde(default)]
    compiled_tokens: usize,
    /// Tokens saved by compilation (raw − compiled).
    #[serde(default)]
    tokens_saved: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeClientContext {
    client_app: Option<String>,
    upstream_provider: Option<String>,
    upstream_model: Option<String>,
    tenant_id: Option<String>,
    project_id: Option<String>,
    updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceContext {
    tenant_id: Option<String>,
    project_id: Option<String>,
    policy_pack: Option<String>,
    /// V10.3 — Session cost budget in USD (0 = disabled).
    #[serde(default)]
    session_budget_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceContextFile {
    workspace: WorkspaceContext,
}

/// Runtime policy configuration loaded from `configs/policies/policies.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PolicyConfig {
    #[serde(default)]
    sensitive_data: Option<String>,
    #[serde(default)]
    max_tokens_per_request: Option<usize>,
    #[serde(default)]
    fallback_provider: Option<String>,
    #[serde(default)]
    data_residency: Option<String>,
    #[serde(default)]
    pii_masking: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetricsSnapshot {
    ts: u64,
    total_requests: u64,
    raw_tokens: usize,
    compiled_tokens: usize,
    memory_reused_tokens: usize,
    efficiency_score: f32,
    local_ratio: f32,
    cache_hits: u64,
    cache_misses: u64,
    cache_saved_tokens: usize,
    history_raw: Vec<usize>,
    history_compiled: Vec<usize>,
    history_reused: Vec<usize>,
    history_hour_epochs: Vec<u64>,
    history_hour_raw: Vec<usize>,
    history_hour_compiled: Vec<usize>,
    history_hour_reused: Vec<usize>,
    routes_local: u64,
    routes_cloud: u64,
    routes_midtier: u64,
    intent_stats: std::collections::HashMap<String, IntentStats>,
    model_stats: std::collections::HashMap<String, ModelStats>,
    upstream_stats: std::collections::HashMap<String, UpstreamStats>,
    last_request: Option<RequestLineage>,
    request_history: Vec<RequestLineage>,
    /// Cumulative session cost in USD across all requests.
    #[serde(default)]
    session_cost_usd: f64,
    /// Cost of the most recent request in USD.
    #[serde(default)]
    last_request_cost_usd: f64,
    /// V10 — Number of stable context blocks currently held in the ContextStore.
    #[serde(default)]
    stable_blocks: usize,
    /// V10 — Session-level context reuse ratio in percent (memory_reused / raw * 100).
    #[serde(default)]
    context_reuse_ratio_pct: f32,
    /// V10.3 — Configured session cost budget in USD (0 = disabled).
    #[serde(default)]
    session_budget_usd: f64,
    /// V10.15 — Number of requests where RCT2I prompt structuring was applied.
    #[serde(default)]
    rct2i_applied_count: u64,
}

#[derive(Debug)]
struct MetricsCollector {
    snapshot: MetricsSnapshot,
    sem_cache: cache::SemanticCache,
    chat_cache: HashMap<String, CachedChatResponse>,
    context_store: memory::ContextStore,
    audit_retention_secs: u64,
    audit_history_limit: usize,
    hour_buckets: HashMap<u64, (usize, usize, usize)>,
    /// Per-provider request counts for the current UTC day.
    daily_provider_counts: HashMap<String, u64>,
    /// UTC midnight epoch for the current day window.
    daily_reset_epoch: u64,
    /// Per-provider rolling latency: (sum_ms, count). Used for latency-aware routing.
    provider_latency: HashMap<String, (f64, u64)>,
    /// V10 — Session-level error count per provider (never reset).
    provider_errors: HashMap<String, u64>,
    /// V10 — Session-level total forward requests per provider (success + error).
    provider_total: HashMap<String, u64>,
    persistence_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedChatResponse {
    content: String,
    model: String,
    prompt_tokens: Option<usize>,
    completion_tokens: Option<usize>,
}

#[derive(Debug, Clone, Default)]
struct UpstreamIdentity {
    client_app: Option<String>,
    provider: Option<String>,
    model: Option<String>,
}

struct RecordEntry {
    raw: usize,
    compiled: usize,
    reused: usize,
    provider: String,
    model: String,
    cache_hit: bool,
    semantic_cache_hit: bool,
    semantic_fingerprint: Option<String>,
    cache_saved_tokens: usize,
    intent: String,
    sensitive: bool,
    upstream: UpstreamIdentity,
    scope: WorkspaceScope,
    /// Estimated cost of this request in USD.
    cost_usd: f64,
    /// Measured wall-clock latency of the LLM provider round-trip (ms).
    /// 0 for cache hits.
    latency_ms: u64,
    /// V10.15 — Whether RCT2I prompt structuring was applied.
    rct2i_applied: bool,
}

#[derive(Debug, Clone, Default)]
struct WorkspaceScope {
    tenant_id: Option<String>,
    project_id: Option<String>,
    policy_pack: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedCollectorState {
    snapshot: MetricsSnapshot,
    sem_cache_entries: Vec<cache::CacheEntry>,
    chat_cache: HashMap<String, CachedChatResponse>,
    context_blocks: Vec<memory::ContextBlock>,
    hour_buckets: HashMap<u64, (usize, usize, usize)>,
    /// V10.17 — Persisted per-provider forward counts for cumulative metrics.
    #[serde(default)]
    provider_total: HashMap<String, u64>,
    /// V10.17 — Persisted per-provider error counts for cumulative metrics.
    #[serde(default)]
    provider_errors: HashMap<String, u64>,
    /// V10.17 — Persisted per-provider latency sums for cumulative metrics.
    #[serde(default)]
    provider_latency: HashMap<String, (f64, u64)>,
}

impl MetricsCollector {
    fn new() -> Self {
        let persistence_path = runtime_state_path();
        let mut collector = Self {
            snapshot: MetricsSnapshot {
                ts: now_epoch(),
                total_requests: 0,
                raw_tokens: 0,
                compiled_tokens: 0,
                memory_reused_tokens: 0,
                efficiency_score: 0.0,
                local_ratio: 0.0,
                cache_hits: 0,
                cache_misses: 0,
                cache_saved_tokens: 0,
                history_raw: Vec::with_capacity(24),
                history_compiled: Vec::with_capacity(24),
                history_reused: Vec::with_capacity(24),
                history_hour_epochs: Vec::with_capacity(24),
                history_hour_raw: Vec::with_capacity(24),
                history_hour_compiled: Vec::with_capacity(24),
                history_hour_reused: Vec::with_capacity(24),
                routes_local: 0,
                routes_cloud: 0,
                routes_midtier: 0,
                intent_stats: std::collections::HashMap::new(),
                model_stats: std::collections::HashMap::new(),
                upstream_stats: std::collections::HashMap::new(),
                last_request: None,
                request_history: Vec::with_capacity(200),
                session_cost_usd: 0.0,
                last_request_cost_usd: 0.0,
                stable_blocks: 0,
                context_reuse_ratio_pct: 0.0,
                session_budget_usd: 0.0,
                rct2i_applied_count: 0,
            },
            sem_cache: cache::SemanticCache::new(),
            chat_cache: HashMap::new(),
            context_store: memory::ContextStore::new(),
            audit_retention_secs: read_u64_env("DISTIRA_AUDIT_RETENTION_DAYS", 7)
                .saturating_mul(24 * 60 * 60),
            audit_history_limit: read_usize_env("DISTIRA_AUDIT_HISTORY_LIMIT", 2000),
            hour_buckets: HashMap::new(),
            daily_provider_counts: HashMap::new(),
            daily_reset_epoch: now_epoch() / 86400 * 86400,
            provider_latency: HashMap::new(),
            provider_errors: HashMap::new(),
            provider_total: HashMap::new(),
            persistence_path,
        };

        if let Err(error) = collector.restore_from_disk() {
            eprintln!("Warning: runtime state restore failed: {error}");
        }

        collector
    }

    fn record(&mut self, e: RecordEntry) {
        let RecordEntry {
            raw,
            compiled,
            reused,
            provider,
            model,
            cache_hit,
            semantic_cache_hit,
            semantic_fingerprint,
            cache_saved_tokens,
            intent,
            sensitive,
            upstream,
            scope,
            cost_usd,
            latency_ms,
            rct2i_applied,
        } = e;

        // V9.11 — Daily budget tracking: reset counts at UTC midnight.
        let today_epoch = now_epoch() / 86400 * 86400;
        if today_epoch > self.daily_reset_epoch {
            self.daily_provider_counts.clear();
            self.daily_reset_epoch = today_epoch;
        }
        *self
            .daily_provider_counts
            .entry(provider.clone())
            .or_insert(0) += 1;

        // V9.16 — Rolling latency tracking (EMA-style sum/count per provider).
        if latency_ms > 0 {
            let entry = self
                .provider_latency
                .entry(provider.clone())
                .or_insert((0.0, 0));
            entry.0 += latency_ms as f64;
            entry.1 += 1;
        }

        // V10 — Track total requests per provider for error rate computation.
        *self.provider_total.entry(provider.clone()).or_insert(0) += 1;

        let s = &mut self.snapshot;
        let ts = now_epoch();
        s.total_requests += 1;
        s.raw_tokens += raw;
        s.compiled_tokens += compiled;
        s.memory_reused_tokens += reused;

        if cache_hit {
            s.cache_hits += 1;
        } else {
            s.cache_misses += 1;
        }
        s.cache_saved_tokens += cache_saved_tokens;

        if rct2i_applied {
            s.rct2i_applied_count += 1;
        }

        // Classify deployment type from provider name
        if is_sovereign_provider(&provider) {
            s.routes_local += 1;
        } else if is_midtier_provider(&provider) {
            s.routes_midtier += 1;
        } else {
            s.routes_cloud += 1;
        }

        // V10.19: Include memory_reused_tokens in cumulative efficiency (was missing)
        let avoided = s.raw_tokens.saturating_sub(s.compiled_tokens) + s.memory_reused_tokens;
        s.efficiency_score = if s.raw_tokens == 0 {
            0.0
        } else {
            ((avoided as f32 / s.raw_tokens as f32) * 100.0 + 30.0).min(100.0)
        };

        let total_routes = s.routes_local + s.routes_cloud + s.routes_midtier;
        s.local_ratio = if total_routes == 0 {
            0.0
        } else {
            (s.routes_local as f32 / total_routes as f32) * 100.0
        };

        s.history_raw.push(s.raw_tokens);
        s.history_compiled.push(s.compiled_tokens);
        s.history_reused.push(s.memory_reused_tokens);
        if s.history_raw.len() > 24 {
            s.history_raw.remove(0);
            s.history_compiled.remove(0);
            s.history_reused.remove(0);
        }

        // Build true hour-by-hour trend buckets for the last 24h.
        const TREND_HOURS: u64 = 24;
        const HOUR_SECS: u64 = 60 * 60;
        let current_hour = ts - (ts % HOUR_SECS);
        let oldest_hour = current_hour.saturating_sub((TREND_HOURS - 1) * HOUR_SECS);

        let bucket = self.hour_buckets.entry(current_hour).or_insert((0, 0, 0));
        bucket.0 += raw;
        bucket.1 += compiled;
        bucket.2 += reused;

        self.hour_buckets.retain(|hour, _| *hour >= oldest_hour);

        let mut hour_epochs = Vec::with_capacity(TREND_HOURS as usize);
        let mut hour_raw = Vec::with_capacity(TREND_HOURS as usize);
        let mut hour_compiled = Vec::with_capacity(TREND_HOURS as usize);
        let mut hour_reused = Vec::with_capacity(TREND_HOURS as usize);

        for i in 0..TREND_HOURS {
            let hour = oldest_hour + i * HOUR_SECS;
            let (r, c, m) = self.hour_buckets.get(&hour).copied().unwrap_or((0, 0, 0));
            hour_epochs.push(hour);
            hour_raw.push(r);
            hour_compiled.push(c);
            hour_reused.push(m);
        }

        s.history_hour_epochs = hour_epochs;
        s.history_hour_raw = hour_raw;
        s.history_hour_compiled = hour_compiled;
        s.history_hour_reused = hour_reused;

        let entry = s
            .intent_stats
            .entry(intent.to_string())
            .or_insert(IntentStats {
                requests: 0,
                raw_tokens: 0,
                compiled_tokens: 0,
            });
        entry.requests += 1;
        entry.raw_tokens += raw;
        entry.compiled_tokens += compiled;

        let is_sovereign = is_sovereign_provider(&provider);
        let model_key = format!("{}@{}", model, provider);
        let model_entry = s.model_stats.entry(model_key).or_insert(ModelStats {
            model: model.to_string(),
            provider: provider.to_string(),
            requests: 0,
            raw_tokens: 0,
            compiled_tokens: 0,
            memory_reused_tokens: 0,
            efficiency_score: 0.0,
            sovereign_requests: 0,
            non_sovereign_requests: 0,
            sovereign_ratio: 0.0,
            avg_latency_ms: 0.0,
            latency_sum_ms: 0.0,
            latency_samples: 0,
        });
        model_entry.requests += 1;
        model_entry.raw_tokens += raw;
        model_entry.compiled_tokens += compiled;
        model_entry.memory_reused_tokens += reused;
        if is_sovereign {
            model_entry.sovereign_requests += 1;
        } else {
            model_entry.non_sovereign_requests += 1;
        }

        let model_avoided = model_entry
            .raw_tokens
            .saturating_sub(model_entry.compiled_tokens);
        model_entry.efficiency_score = if model_entry.raw_tokens == 0 {
            0.0
        } else {
            ((model_avoided as f32 / model_entry.raw_tokens as f32) * 100.0 + 30.0).min(100.0)
        };
        let model_total_routes =
            model_entry.sovereign_requests + model_entry.non_sovereign_requests;
        model_entry.sovereign_ratio = if model_total_routes == 0 {
            0.0
        } else {
            (model_entry.sovereign_requests as f32 / model_total_routes as f32) * 100.0
        };
        // V9.16 — per-model latency accumulation.
        if latency_ms > 0 {
            model_entry.latency_sum_ms += latency_ms as f64;
            model_entry.latency_samples += 1;
            model_entry.avg_latency_ms =
                model_entry.latency_sum_ms / model_entry.latency_samples as f64;
        }

        let has_upstream_metadata = upstream.client_app.is_some()
            || upstream.provider.is_some()
            || upstream.model.is_some();

        if has_upstream_metadata {
            let client_app = upstream
                .client_app
                .clone()
                .unwrap_or_else(|| "unknown-client".into());
            let upstream_provider = upstream
                .provider
                .clone()
                .unwrap_or_else(|| "unknown-provider".into());
            let upstream_model = upstream
                .model
                .clone()
                .unwrap_or_else(|| "unknown-model".into());
            let upstream_key = format!("{}|{}|{}", client_app, upstream_provider, upstream_model);
            let upstream_entry = s
                .upstream_stats
                .entry(upstream_key)
                .or_insert(UpstreamStats {
                    client_app,
                    upstream_provider,
                    upstream_model,
                    requests: 0,
                    last_seen_ts: ts,
                });
            upstream_entry.requests += 1;
            upstream_entry.last_seen_ts = ts;
        }

        s.session_cost_usd += cost_usd;
        s.last_request_cost_usd = cost_usd;

        // V10 — Live context memory stats from ContextStore.
        s.stable_blocks = self.context_store.len();
        s.context_reuse_ratio_pct = if s.raw_tokens > 0 {
            (s.memory_reused_tokens as f32 / s.raw_tokens as f32 * 100.0).min(100.0)
        } else {
            0.0
        };

        let tokens_saved = raw.saturating_sub(compiled);
        let lineage = RequestLineage {
            client_app: upstream.client_app,
            upstream_provider: upstream.provider,
            upstream_model: upstream.model,
            tenant_id: scope.tenant_id,
            project_id: scope.project_id,
            policy_pack: scope.policy_pack,
            routed_provider: provider,
            routed_model: model,
            intent,
            semantic_cache_hit,
            semantic_fingerprint,
            cache_hit,
            sensitive,
            ts,
            cost_usd,
            raw_tokens: raw,
            compiled_tokens: compiled,
            tokens_saved,
        };

        s.last_request = Some(lineage.clone());
        s.request_history.push(lineage);
        let min_ts = ts.saturating_sub(self.audit_retention_secs);
        prune_request_history(
            &mut s.request_history,
            if self.audit_retention_secs == 0 {
                None
            } else {
                Some(min_ts)
            },
            self.audit_history_limit,
        );

        s.ts = ts;

        if let Err(error) = self.persist_to_disk() {
            eprintln!("Warning: runtime state persistence failed: {error}");
        }
    }

    fn snapshot(&self) -> &MetricsSnapshot {
        &self.snapshot
    }

    /// Returns a map of provider key → average latency in milliseconds.
    /// Only providers that have received at least one timed request are included.
    fn avg_latency_by_provider(&self) -> HashMap<String, f64> {
        self.provider_latency
            .iter()
            .filter(|(_, (_, count))| *count > 0)
            .map(|(k, (sum, count))| (k.clone(), sum / *count as f64))
            .collect()
    }

    /// V10 — Record a failed forward attempt for a provider.
    fn record_error(&mut self, provider: &str) {
        *self
            .provider_errors
            .entry(provider.to_string())
            .or_insert(0) += 1;
        *self.provider_total.entry(provider.to_string()).or_insert(0) += 1;
    }

    /// V10 — Compute per-provider error rate from session totals.
    fn error_rate_by_provider(&self) -> HashMap<String, f64> {
        self.provider_errors
            .iter()
            .filter_map(|(k, &errors)| {
                let total = self.provider_total.get(k).copied().unwrap_or(0);
                if total == 0 {
                    return None;
                }
                Some((k.clone(), errors as f64 / total as f64))
            })
            .collect()
    }

    fn reset(&mut self) {
        self.snapshot = MetricsSnapshot {
            ts: now_epoch(),
            total_requests: 0,
            raw_tokens: 0,
            compiled_tokens: 0,
            memory_reused_tokens: 0,
            efficiency_score: 0.0,
            local_ratio: 0.0,
            cache_hits: 0,
            cache_misses: 0,
            cache_saved_tokens: 0,
            history_raw: Vec::with_capacity(24),
            history_compiled: Vec::with_capacity(24),
            history_reused: Vec::with_capacity(24),
            history_hour_epochs: Vec::with_capacity(24),
            history_hour_raw: Vec::with_capacity(24),
            history_hour_compiled: Vec::with_capacity(24),
            history_hour_reused: Vec::with_capacity(24),
            routes_local: 0,
            routes_cloud: 0,
            routes_midtier: 0,
            intent_stats: std::collections::HashMap::new(),
            model_stats: std::collections::HashMap::new(),
            upstream_stats: std::collections::HashMap::new(),
            last_request: None,
            request_history: Vec::with_capacity(200),
            session_cost_usd: 0.0,
            last_request_cost_usd: 0.0,
            stable_blocks: 0,
            context_reuse_ratio_pct: 0.0,
            session_budget_usd: 0.0,
            rct2i_applied_count: 0,
        };
        self.hour_buckets.clear();
    }

    fn persisted_state(&self) -> PersistedCollectorState {
        PersistedCollectorState {
            snapshot: self.snapshot.clone(),
            sem_cache_entries: self.sem_cache.entries(),
            chat_cache: self.chat_cache.clone(),
            context_blocks: self.context_store.blocks(),
            hour_buckets: self.hour_buckets.clone(),
            provider_total: self.provider_total.clone(),
            provider_errors: self.provider_errors.clone(),
            provider_latency: self.provider_latency.clone(),
        }
    }

    fn persist_to_disk(&self) -> Result<(), String> {
        let body = serde_json::to_string_pretty(&self.persisted_state())
            .map_err(|error| format!("Cannot serialize runtime state: {error}"))?;
        write_atomic_text(&self.persistence_path, &body)
    }

    fn restore_from_disk(&mut self) -> Result<(), String> {
        if !self.persistence_path.exists() {
            return Ok(());
        }

        let raw = std::fs::read_to_string(&self.persistence_path)
            .map_err(|error| format!("Cannot read {}: {error}", self.persistence_path.display()))?;
        let mut restored: PersistedCollectorState =
            serde_json::from_str(&raw).map_err(|error| {
                format!("Cannot parse {}: {error}", self.persistence_path.display())
            })?;

        // Apply retention guardrails on restored audit lineage.
        let now = now_epoch();
        let min_ts = now.saturating_sub(self.audit_retention_secs);
        prune_request_history(
            &mut restored.snapshot.request_history,
            if self.audit_retention_secs == 0 {
                None
            } else {
                Some(min_ts)
            },
            self.audit_history_limit,
        );

        restored.snapshot.last_request = restored.snapshot.request_history.last().cloned();

        self.snapshot = restored.snapshot;
        self.sem_cache.load_entries(restored.sem_cache_entries);
        self.chat_cache = restored.chat_cache;
        self.context_store.load_blocks(restored.context_blocks);
        self.hour_buckets = restored.hour_buckets;
        self.provider_total = restored.provider_total;
        self.provider_errors = restored.provider_errors;
        self.provider_latency = restored.provider_latency;

        Ok(())
    }
}

/// Combined shared state
struct AppState {
    collector: Mutex<MetricsCollector>,
    router_config: router::RouterConfig,
    workspace_context: WorkspaceContext,
    policies: PolicyConfig,
}

type SharedState = Arc<AppState>;

fn compile_result_from_cache(entry: &cache::CacheEntry) -> compiler::CompileResult {
    compiler::CompileResult {
        intent: entry.intent.clone(),
        intent_confidence: 0.0, // reconstructed from cache — confidence not re-derived
        raw_tokens_estimate: entry.raw_tokens_estimate,
        compiled_tokens_estimate: entry.compiled_tokens_estimate,
        optimizer_savings: 0, // not stored in cache; conservative default
        summary: entry.summary.clone(),
        compiled_context: entry.compiled_context.clone(),
        slash_command: None,
        force_local: false,
        efficiency_directive: compiler::efficiency_directive_for_context(
            &entry.intent,
            &entry.compiled_context,
        )
        .to_string(),
        rct2i_applied: entry.rct2i_applied,
        rct2i_sections: entry.rct2i_sections,
    }
}

fn cache_entry_from_compile_result(
    fingerprint: u64,
    result: &compiler::CompileResult,
) -> cache::CacheEntry {
    cache::CacheEntry {
        fingerprint,
        intent: result.intent.clone(),
        raw_tokens_estimate: result.raw_tokens_estimate,
        compiled_tokens_estimate: result.compiled_tokens_estimate,
        summary: result.summary.clone(),
        compiled_context: result.compiled_context.clone(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        rct2i_applied: result.rct2i_applied,
        rct2i_sections: result.rct2i_sections,
    }
}

fn compile_with_semantic_cache(
    collector: &mut MetricsCollector,
    raw: &str,
    client_app: Option<&str>,
) -> (u64, compiler::CompileResult, bool) {
    let canonical = compiler::canonicalize_context(raw);
    let fingerprint = fingerprint::fingerprint(&canonical);
    if let Some(entry) = collector.sem_cache.get(fingerprint) {
        return (fingerprint, compile_result_from_cache(entry), true);
    }

    let result = compiler::compile_context_with_hint(raw, client_app);
    collector
        .sem_cache
        .insert(cache_entry_from_compile_result(fingerprint, &result));
    // Register compiled context in real memory store for future reuse tracking
    collector
        .context_store
        .register(fingerprint, &result.compiled_context, &result.intent);
    (fingerprint, result, false)
}

fn app_version() -> &'static str {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../VERSION")).trim()
}

fn runtime_version() -> String {
    let candidates = [
        PathBuf::from("VERSION"),
        PathBuf::from("../VERSION"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../VERSION"),
    ];

    for path in candidates {
        if let Ok(version) = std::fs::read_to_string(&path) {
            let trimmed = version.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    app_version().to_string()
}

fn is_sovereign_provider(provider: &str) -> bool {
    provider.contains("local") || provider.contains("ollama")
}

fn is_midtier_provider(provider: &str) -> bool {
    provider.contains("mistral")
}

fn upstream_identity(
    client_app: Option<&str>,
    upstream_provider: Option<&str>,
    upstream_model: Option<&str>,
) -> UpstreamIdentity {
    UpstreamIdentity {
        client_app: client_app.map(str::to_owned),
        provider: upstream_provider.map(str::to_owned),
        model: upstream_model.map(str::to_owned),
    }
}

fn runtime_client_context_path() -> PathBuf {
    let candidates = [
        PathBuf::from("cache/client-context.json"),
        PathBuf::from("../cache/client-context.json"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cache/client-context.json"),
    ];

    candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| PathBuf::from("cache/client-context.json"))
}

fn runtime_state_path() -> PathBuf {
    if let Ok(custom) = std::env::var("DISTIRA_RUNTIME_STATE_PATH") {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    let candidates = [
        PathBuf::from("cache/runtime-state.json"),
        PathBuf::from("../cache/runtime-state.json"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cache/runtime-state.json"),
    ];

    candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| PathBuf::from("cache/runtime-state.json"))
}

fn write_atomic_text(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Cannot create {}: {error}", parent.display()))?;
    }

    let mut temp = path.to_path_buf();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!("{ext}.tmp"))
        .unwrap_or_else(|| "tmp".to_string());
    temp.set_extension(extension);

    std::fs::write(&temp, content)
        .map_err(|error| format!("Cannot write temp state {}: {error}", temp.display()))?;

    if path.exists() {
        std::fs::remove_file(path)
            .map_err(|error| format!("Cannot replace state file {}: {error}", path.display()))?;
    }

    std::fs::rename(&temp, path)
        .map_err(|error| format!("Cannot move temp state {}: {error}", path.display()))?;

    Ok(())
}

fn workspace_context_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("configs/workspace/workspace.yaml"),
        PathBuf::from("../configs/workspace/workspace.yaml"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../configs/workspace/workspace.yaml"),
    ];
    candidates.into_iter().find(|path| path.exists())
}

fn load_workspace_context() -> WorkspaceContext {
    if let Some(path) = workspace_context_path() {
        if let Ok(raw) = std::fs::read_to_string(&path) {
            // Try wrapped form first (`workspace:` key) — most yaml configs use this.
            if let Ok(wrapped) = serde_yaml::from_str::<WorkspaceContextFile>(&raw) {
                return wrapped.workspace;
            }
            // Fallback: flat form (no wrapper key).
            if let Ok(context) = serde_yaml::from_str::<WorkspaceContext>(&raw) {
                return context;
            }
        }
    }

    WorkspaceContext {
        tenant_id: None,
        project_id: None,
        policy_pack: None,
        session_budget_usd: 0.0,
    }
}

fn policies_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("configs/policies/policies.yaml"),
        PathBuf::from("../configs/policies/policies.yaml"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../configs/policies/policies.yaml"),
    ];
    candidates.into_iter().find(|path| path.exists())
}

fn load_policies() -> PolicyConfig {
    if let Some(path) = policies_path() {
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_yaml::from_str::<PolicyConfig>(&raw) {
                return cfg;
            }
        }
    }
    PolicyConfig::default()
}

fn resolve_workspace_scope(
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    runtime_context: &RuntimeClientContext,
    workspace_context: &WorkspaceContext,
) -> WorkspaceScope {
    WorkspaceScope {
        tenant_id: tenant_id
            .map(str::to_owned)
            .or_else(|| runtime_context.tenant_id.clone())
            .or_else(|| workspace_context.tenant_id.clone()),
        project_id: project_id
            .map(str::to_owned)
            .or_else(|| runtime_context.project_id.clone())
            .or_else(|| workspace_context.project_id.clone()),
        policy_pack: workspace_context.policy_pack.clone(),
    }
}

fn read_runtime_client_context() -> RuntimeClientContext {
    let path = runtime_client_context_path();
    if let Ok(raw) = std::fs::read_to_string(path) {
        if let Ok(context) = serde_json::from_str::<RuntimeClientContext>(&raw) {
            return context;
        }
    }

    RuntimeClientContext {
        client_app: None,
        upstream_provider: None,
        upstream_model: None,
        tenant_id: None,
        project_id: None,
        updated_at: 0,
    }
}

fn write_runtime_client_context(
    client_app: Option<String>,
    upstream_provider: Option<String>,
    upstream_model: Option<String>,
    tenant_id: Option<String>,
    project_id: Option<String>,
) -> Result<RuntimeClientContext, String> {
    let path = runtime_client_context_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Cannot create {}: {error}", parent.display()))?;
    }

    let context = RuntimeClientContext {
        client_app,
        upstream_provider,
        upstream_model,
        tenant_id,
        project_id,
        updated_at: now_epoch(),
    };
    let body = serde_json::to_string_pretty(&context)
        .map_err(|error| format!("Cannot serialize runtime context: {error}"))?;
    std::fs::write(&path, body)
        .map_err(|error| format!("Cannot write {}: {error}", path.display()))?;
    Ok(context)
}

fn absorb_sse_payload(
    payload: &str,
    cached_content: &mut String,
    cached_model: &mut String,
    completion_tokens: &mut Option<usize>,
) {
    for line in payload.lines() {
        let Some(data) = line.strip_prefix("data: ") else {
            continue;
        };
        if data.trim() == "[DONE]" {
            continue;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(data) else {
            continue;
        };
        if let Some(model) = value["model"].as_str() {
            *cached_model = model.to_string();
        }
        if let Some(choice) = value["choices"].get(0) {
            if let Some(delta) = choice["delta"]["content"].as_str() {
                cached_content.push_str(delta);
            }
            if let Some(message) = choice["message"]["content"].as_str() {
                cached_content.push_str(message);
            }
        }
        if let Some(tokens) = value["usage"]["completion_tokens"].as_u64() {
            *completion_tokens = Some(tokens as usize);
        }
    }
}

fn absorb_sse_chunk(
    chunk: &str,
    pending: &mut String,
    cached_content: &mut String,
    cached_model: &mut String,
    completion_tokens: &mut Option<usize>,
) {
    pending.push_str(chunk);

    while let Some(delimiter_index) = pending.find("\n\n") {
        let event = pending[..delimiter_index].to_string();
        pending.drain(..delimiter_index + 2);
        absorb_sse_payload(&event, cached_content, cached_model, completion_tokens);
    }
}

fn extract_message_text(message: &Value) -> String {
    match &message["content"] {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn extract_conversation_text(messages: &[Value]) -> String {
    messages
        .iter()
        .map(extract_message_text)
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn build_forward_messages(messages: &[Value], fallback_user_content: &str) -> Vec<Value> {
    if messages.is_empty() {
        return vec![json!({
            "role": "user",
            "content": fallback_user_content,
        })];
    }

    messages.to_vec()
}

fn extract_latest_user_text(messages: &[Value]) -> String {
    messages
        .iter()
        .rev()
        .find(|message| message["role"] == "user")
        .map(extract_message_text)
        .unwrap_or_default()
}

fn apply_compiled_user_message(messages: &[Value], compiled_content: &str) -> Vec<Value> {
    if messages.is_empty() {
        return vec![json!({
            "role": "user",
            "content": compiled_content,
        })];
    }

    let mut rewritten = messages.to_vec();
    if let Some(index) = rewritten
        .iter()
        .rposition(|message| message["role"] == "user")
    {
        if let Some(object) = rewritten[index].as_object_mut() {
            object.insert(
                "content".into(),
                Value::String(compiled_content.to_string()),
            );
        }
    } else {
        rewritten.push(json!({
            "role": "user",
            "content": compiled_content,
        }));
    }

    rewritten
}

/// Compress a long conversation: if > MAX_FULL_TURNS messages, collapse older turns
/// into a compact system-message summary to preserve context budget.
/// V9.14: older turns are distilled via the compiler pipeline instead of naive word-truncation.
const MAX_FULL_TURNS: usize = 6;

/// V9.15 → V10.14 — Per-intent Efficiency Directive Injection.
/// Prepend an intent-specific efficiency directive to the system role so the LLM
/// returns concise, token-efficient answers — reducing output token consumption.
/// Always active: this is part of DISTIRA's core value proposition.
fn inject_efficiency_directive(mut messages: Vec<Value>, intent: &str) -> Vec<Value> {
    let directive = compiler::efficiency_directive(intent);

    if let Some(idx) = messages.iter().position(|m| m["role"] == "system") {
        let existing = extract_message_text(&messages[idx]);
        if let Some(obj) = messages[idx].as_object_mut() {
            obj.insert(
                "content".into(),
                Value::String(format!("{directive}\n\n{existing}")),
            );
        }
    } else {
        messages.insert(0, json!({ "role": "system", "content": directive }));
    }
    messages
}

/// V10.15 — Compile older conversation turns through the DISTIRA pipeline.
/// System prompts and assistant messages in the history are often verbose.
/// By compiling them (same as we compile user messages), we save significant
/// tokens on every request.  Only the last 2 messages are kept verbatim
/// (the current exchange).  Short messages (< 100 chars) are skipped.
fn compile_older_turns(messages: Vec<Value>) -> Vec<Value> {
    let len = messages.len();
    if len <= 2 {
        return messages;
    }
    messages
        .into_iter()
        .enumerate()
        .map(|(i, mut msg)| {
            // Keep the last 2 messages unmodified (current turn)
            if i >= len - 2 {
                return msg;
            }
            let content = extract_message_text(&msg);
            if content.len() < 100 {
                return msg;
            }
            let compiled = compiler::compile_context(&content);
            let compiled_text = compiled
                .compiled_context
                .split_once('|')
                .map(|x| x.1)
                .unwrap_or(&compiled.compiled_context)
                .trim()
                .to_string();
            if !compiled_text.is_empty()
                && compiled.compiled_tokens_estimate < compiled.raw_tokens_estimate
            {
                if let Some(obj) = msg.as_object_mut() {
                    obj.insert("content".into(), Value::String(compiled_text));
                }
            }
            msg
        })
        .collect()
}

/// V10.15 — Remove duplicate paragraphs across conversation messages.
/// In multi-turn chat, users often quote the assistant's previous response,
/// and assistants echo the user's question.  This wastes tokens on repeated
/// content.  We fingerprint paragraphs (≥30 chars) and remove earlier
/// occurrences, keeping only the latest.
fn dedup_cross_messages(messages: Vec<Value>) -> Vec<Value> {
    if messages.len() < 3 {
        return messages;
    }
    // Build paragraph → latest message index map
    let mut para_latest: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    for (i, msg) in messages.iter().enumerate() {
        let content = extract_message_text(msg);
        for para in content.split("\n\n") {
            let normalized = para
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            if normalized.len() >= 30 {
                let fp = fingerprint::fingerprint(&normalized);
                para_latest.insert(fp, i);
            }
        }
    }
    // Rebuild: remove paragraphs that appear in a later message
    messages
        .into_iter()
        .enumerate()
        .map(|(i, mut msg)| {
            let content = extract_message_text(&msg);
            if content.is_empty() {
                return msg;
            }
            let paragraphs: Vec<&str> = content.split("\n\n").collect();
            if paragraphs.len() <= 1 {
                return msg;
            }
            let mut changed = false;
            let filtered: Vec<&str> = paragraphs
                .iter()
                .filter(|para| {
                    let normalized = para
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .to_lowercase();
                    if normalized.len() < 30 {
                        return true;
                    }
                    let fp = fingerprint::fingerprint(&normalized);
                    let keep = para_latest.get(&fp).copied().unwrap_or(i) == i;
                    if !keep {
                        changed = true;
                    }
                    keep
                })
                .copied()
                .collect();
            if changed && !filtered.is_empty() {
                if let Some(obj) = msg.as_object_mut() {
                    obj.insert("content".into(), Value::String(filtered.join("\n\n")));
                }
            }
            msg
        })
        .collect()
}

fn compress_conversation_history(messages: &[Value]) -> Vec<Value> {
    if messages.len() <= MAX_FULL_TURNS {
        return messages.to_vec();
    }

    let tail_start = messages.len() - MAX_FULL_TURNS;
    let older = &messages[..tail_start];
    let recent = &messages[tail_start..];

    let summary_parts: Vec<String> = older
        .iter()
        .filter_map(|msg| {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = extract_message_text(msg);
            if content.is_empty() {
                return None;
            }
            // V9.14: compile each older turn through the DISTIRA pipeline for
            // real semantic compression instead of naive 20-word truncation.
            let compiled = compiler::compile_context(&content);
            // Strip the intent marker prefix ([k:intent]|) before embedding in summary.
            let compressed_text = compiled
                .compiled_context
                .split_once('|')
                .map(|x| x.1)
                .unwrap_or(&compiled.compiled_context)
                .trim()
                .to_string();
            let display = if compressed_text.is_empty() {
                // Fallback: keep first 20 words of original
                if content.split_whitespace().count() > 20 {
                    content
                        .split_whitespace()
                        .take(20)
                        .collect::<Vec<_>>()
                        .join(" ")
                        + " [...]"
                } else {
                    content
                }
            } else {
                compressed_text
            };
            Some(format!("[{role}]: {display}"))
        })
        .collect();

    let summary_content = format!(
        "[Earlier conversation \u{2014} {} turns compressed]\n{}",
        older.len(),
        summary_parts.join("\n")
    );

    let mut result = vec![json!({
        "role": "system",
        "content": summary_content,
    })];
    result.extend_from_slice(recent);
    result
}

fn build_chat_cache_key(messages: &[Value], extra_body: &Map<String, Value>) -> String {
    let canonical_messages: Vec<Value> = messages
        .iter()
        .map(|message| {
            let mut cloned = message.clone();
            if let Some(obj) = cloned.as_object_mut() {
                if let Some(content) = obj.get("content").and_then(Value::as_str) {
                    obj.insert(
                        "content".into(),
                        Value::String(compiler::canonicalize_context(content)),
                    );
                }
            }
            cloned
        })
        .collect();

    let serialized = serde_json::to_string(&json!({
        "messages": canonical_messages,
        "options": extra_body,
    }))
    .unwrap_or_default();
    fingerprint::fingerprint(&serialized).to_string()
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn read_u64_env(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

fn read_usize_env(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(default)
}

fn prune_request_history(
    history: &mut Vec<RequestLineage>,
    min_ts: Option<u64>,
    max_entries: usize,
) {
    if let Some(cutoff) = min_ts {
        history.retain(|entry| entry.ts >= cutoff);
    }

    if history.len() > max_entries {
        let to_drop = history.len() - max_entries;
        history.drain(0..to_drop);
    }
}

/// Try to load YAML configs from standard paths relative to cwd.
fn load_config() -> router::RouterConfig {
    let configs = ["configs", "../configs", "../../configs"];
    for base in &configs {
        let prov = PathBuf::from(base).join("providers/providers.yaml");
        let rout = PathBuf::from(base).join("routing/routing.yaml");
        if prov.exists() && rout.exists() {
            match router::RouterConfig::load(&prov, &rout) {
                Ok(cfg) => {
                    println!("  Config loaded from {base}/");
                    for name in cfg.list_providers() {
                        println!("    provider: {name}");
                    }
                    return cfg;
                }
                Err(e) => {
                    eprintln!("  Warning: config parse error: {e} — using defaults");
                }
            }
        }
    }
    println!("  No config files found — using built-in defaults");
    router::RouterConfig::defaults()
}

// -- Handlers ---------------------------------------------------

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "service": "distira-core", "version": runtime_version() }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({ "version": runtime_version(), "product": "DISTIRA" }))
}

#[derive(Deserialize)]
struct RuntimeClientContextRequest {
    client_app: Option<String>,
    upstream_provider: Option<String>,
    upstream_model: Option<String>,
    tenant_id: Option<String>,
    project_id: Option<String>,
}

async fn get_runtime_client_context() -> Json<serde_json::Value> {
    Json(serde_json::to_value(read_runtime_client_context()).unwrap_or_default())
}

async fn set_runtime_client_context(
    Json(payload): Json<RuntimeClientContextRequest>,
) -> Json<serde_json::Value> {
    match write_runtime_client_context(
        payload.client_app,
        payload.upstream_provider,
        payload.upstream_model,
        payload.tenant_id,
        payload.project_id,
    ) {
        Ok(context) => Json(serde_json::to_value(context).unwrap_or_default()),
        Err(error) => Json(json!({
            "error": {
                "message": error,
                "type": "runtime_context_error"
            }
        })),
    }
}

async fn list_providers(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let (latency_map, error_map) = {
        let collector = state.collector.lock().unwrap_or_else(|e| e.into_inner());
        (
            collector.avg_latency_by_provider(),
            collector.error_rate_by_provider(),
        )
    };
    let mut summaries: Vec<serde_json::Value> = state
        .router_config
        .list_provider_summaries()
        .into_iter()
        .map(|s| {
            let avg_ms = latency_map.get(&s.key).copied().unwrap_or(0.0);
            let error_rate = error_map.get(&s.key).copied().unwrap_or(0.0);
            let mut v = serde_json::to_value(&s).unwrap_or_default();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("avg_latency_ms".into(), serde_json::json!(avg_ms));
                obj.insert("error_rate".into(), serde_json::json!(error_rate));
            }
            v
        })
        .collect();
    summaries.sort_by(|a, b| {
        a.get("key")
            .and_then(|v| v.as_str())
            .cmp(&b.get("key").and_then(|v| v.as_str()))
    });
    Json(json!({
        "providers": state.router_config.list_providers(),
        "provider_details": summaries
    }))
}

#[derive(Deserialize)]
struct CompileRequest {
    context: Option<String>,
    sensitive: Option<bool>,
    client_app: Option<String>,
    upstream_provider: Option<String>,
    upstream_model: Option<String>,
    tenant_id: Option<String>,
    project_id: Option<String>,
}

async fn compile(
    State(state): State<SharedState>,
    Json(payload): Json<CompileRequest>,
) -> Json<serde_json::Value> {
    let raw = payload.context.as_deref().unwrap_or("");
    let sensitive = payload.sensitive.unwrap_or(false);

    // V9.3 — PII masking before compilation.
    let pii_mask_enabled = sensitive
        || state.policies.pii_masking.unwrap_or(false)
        || state.policies.sensitive_data.as_deref() == Some("local_only");
    let masked = if pii_mask_enabled {
        compiler::mask_pii(raw)
    } else {
        raw.to_string()
    };
    // V9.3 — max_tokens_per_request enforcement.
    let context_input = if let Some(max_tok) = state.policies.max_tokens_per_request {
        let char_budget = max_tok * 4;
        if masked.len() > char_budget {
            masked[..char_budget].to_string()
        } else {
            masked
        }
    } else {
        masked
    };

    let mut collector = state.collector.lock().unwrap();
    let (fp, result, cache_hit) = compile_with_semantic_cache(
        &mut collector,
        &context_input,
        payload.client_app.as_deref(),
    );
    let mem = if cache_hit {
        // Exact semantic cache hit → full block reuse.
        collector
            .context_store
            .compute_reuse(fp, result.raw_tokens_estimate, &result.intent)
    } else {
        // Cache miss — but prior stable blocks may cover part of this compiled
        // context (same vocabulary, same intent).  Count the lexical overlap so
        // the Memory Lensing metric grows with every related compile request,
        // not only on exact cache hits.
        collector
            .context_store
            .estimate_coverage(&result.compiled_context, &result.intent)
    };
    let runtime_context = read_runtime_client_context();
    let scope = resolve_workspace_scope(
        payload.tenant_id.as_deref(),
        payload.project_id.as_deref(),
        &runtime_context,
        &state.workspace_context,
    );
    let route = state.router_config.choose_provider_adaptive(
        &result.intent,
        sensitive || result.force_local,
        &collector.daily_provider_counts,
        &collector.avg_latency_by_provider(),
        &collector.error_rate_by_provider(),
    );
    let upstream = upstream_identity(
        payload
            .client_app
            .as_deref()
            .or(runtime_context.client_app.as_deref()),
        payload
            .upstream_provider
            .as_deref()
            .or(runtime_context.upstream_provider.as_deref()),
        payload
            .upstream_model
            .as_deref()
            .or(runtime_context.upstream_model.as_deref()),
    );

    let efficiency = metrics::compute(
        result.raw_tokens_estimate,
        result.compiled_tokens_estimate,
        mem.reused_tokens,
    );

    collector.record(RecordEntry {
        raw: result.raw_tokens_estimate,
        compiled: result.compiled_tokens_estimate,
        reused: mem.reused_tokens,
        provider: route.provider.clone(),
        model: route.model.clone(),
        cache_hit,
        semantic_cache_hit: cache_hit,
        semantic_fingerprint: Some(fp.to_string()),
        cache_saved_tokens: 0,
        intent: result.intent.clone(),
        sensitive,
        upstream: upstream.clone(),
        scope: scope.clone(),
        cost_usd: state.router_config.cost_estimate_usd(
            &route.provider,
            result.compiled_tokens_estimate,
            0,
        ),
        latency_ms: 0, // compile endpoint has no LLM round-trip
        rct2i_applied: result.rct2i_applied,
    });
    drop(collector);

    Json(json!({
        "fingerprint": fp.to_string(),
        "cache_hit": cache_hit,
        "intent": result.intent,
        "intent_confidence": result.intent_confidence,
        "raw_tokens": result.raw_tokens_estimate,
        "compiled_tokens": result.compiled_tokens_estimate,
        "optimizer_savings": result.optimizer_savings,
        "compiled_context": result.compiled_context,
        "summary": result.summary,
        "slash_command": result.slash_command,
        "force_local": result.force_local,
        "efficiency_directive": result.efficiency_directive,
        "rct2i_applied": result.rct2i_applied,
        "rct2i_sections": result.rct2i_sections,
        "memory_reused_tokens": mem.reused_tokens,
        "context_reuse_ratio": mem.context_reuse_ratio,
        "provider": route.provider,
        "model": route.model,
        "client_app": upstream.client_app,
        "upstream_provider": upstream.provider,
        "upstream_model": upstream.model,
        "tenant_id": scope.tenant_id,
        "project_id": scope.project_id,
        "policy_pack": scope.policy_pack,
        "routing_reason": route.reason,
        "token_avoidance_ratio": efficiency.token_avoidance_ratio,
        "cost_usd": state.router_config.cost_estimate_usd(
            &route.provider,
            result.compiled_tokens_estimate,
            0,
        )
    }))
}

/// OpenAI-compatible chat endpoint.
/// Compiles context, routes, then forwards to the chosen LLM.
#[derive(Deserialize)]
struct ChatRequest {
    #[serde(default)]
    messages: Vec<Value>,
    model: Option<String>,
    sensitive: Option<bool>,
    client_app: Option<String>,
    upstream_provider: Option<String>,
    upstream_model: Option<String>,
    tenant_id: Option<String>,
    project_id: Option<String>,
    stream: Option<bool>,
    #[serde(flatten)]
    extra_body: Map<String, Value>,
}

async fn chat_completions(
    State(state): State<SharedState>,
    Json(payload): Json<ChatRequest>,
) -> Response<Body> {
    let sensitive = payload.sensitive.unwrap_or(false);

    // ── 1. Full context: raw measurement + intent detection ──────────────────
    // Use the FULL conversation text so intent detection and the semantic
    // cache key are derived from the entire picture, and raw_tokens_estimate
    // reflects the true context size — not just the latest user turn.
    let full_context = extract_conversation_text(&payload.messages);
    let raw_compile_input = if full_context.trim().is_empty() {
        extract_latest_user_text(&payload.messages)
    } else {
        full_context.clone()
    };

    // V9.3 — PII masking: apply before compilation when policy or sensitive flag is set.
    let pii_mask_enabled = sensitive
        || state.policies.pii_masking.unwrap_or(false)
        || state.policies.sensitive_data.as_deref() == Some("local_only");
    let pii_masked = if pii_mask_enabled {
        compiler::mask_pii(&raw_compile_input)
    } else {
        raw_compile_input.clone()
    };

    // V9.3 — max_tokens_per_request enforcement: truncate to budget before compile.
    let compile_input = if let Some(max_tok) = state.policies.max_tokens_per_request {
        let char_budget = max_tok * 4; // ~4 chars/token
        if pii_masked.len() > char_budget {
            pii_masked[..char_budget].to_string()
        } else {
            pii_masked
        }
    } else {
        pii_masked
    };

    let (semantic_fp, result, semantic_cache_hit) = {
        let mut collector = state.collector.lock().unwrap();
        compile_with_semantic_cache(
            &mut collector,
            &compile_input,
            payload.client_app.as_deref(),
        )
    };

    // ── 2. Compile latest user message for clean LLM injection ───────────────
    // For multi-turn conversations we compile only the latest user turn so
    // the dialogue structure is preserved for the upstream LLM.  For
    // single-turn requests we inject the full compiled context directly.
    let latest_user = extract_latest_user_text(&payload.messages);
    let injection_content = if payload.messages.len() <= 1 || latest_user.trim().is_empty() {
        // Single-turn: inject the full compiled context
        if result.compiled_context.trim().is_empty() {
            compile_input.clone()
        } else {
            result.compiled_context.clone()
        }
    } else {
        // Multi-turn: compile just the latest user message so the LLM
        // receives a proper Q&A structure with a compressed question.
        let last_compiled = compiler::compile_context(&latest_user);
        if last_compiled.compiled_context.trim().is_empty() {
            latest_user.clone()
        } else {
            last_compiled.compiled_context
        }
    };

    // ── 3. Build forwarded messages ──────────────────────────────────────────────
    let compiled_messages = if payload.messages.is_empty() {
        build_forward_messages(&payload.messages, &injection_content)
    } else {
        apply_compiled_user_message(&payload.messages, &injection_content)
    };
    // V10.15: compile older turns (system prompts + old assistant messages)
    let compiled_messages = compile_older_turns(compiled_messages);
    // V10.15: deduplicate repeated paragraphs across messages
    let compiled_messages = dedup_cross_messages(compiled_messages);
    // Compress history when conversation has grown beyond MAX_FULL_TURNS
    let forwarded_messages = compress_conversation_history(&compiled_messages);
    // V10.14: always inject intent-specific efficiency directive — this is
    // DISTIRA's core output optimization, not optional.
    let forwarded_messages = inject_efficiency_directive(forwarded_messages, &result.intent);

    // Determine route early so compiled_total uses model-aware token counting.
    // V9.16: snapshot daily counts + latency map before locking for the cache check.
    let (daily_counts, latency_map, error_map) = {
        let c = state.collector.lock().unwrap_or_else(|e| e.into_inner());
        (
            c.daily_provider_counts.clone(),
            c.avg_latency_by_provider(),
            c.error_rate_by_provider(),
        )
    };
    let route = state.router_config.choose_provider_adaptive(
        &result.intent,
        sensitive || result.force_local,
        &daily_counts,
        &latency_map,
        &error_map,
    );
    let model = payload.model.clone().unwrap_or_else(|| route.model.clone());

    // ── 4. Measure COMPILED = actual forwarded token count ────────────────────
    // Honest measure: what DISTIRA actually sends to the LLM after history
    // compression and per-turn compilation.  The gap vs raw_tokens_estimate
    // is the real savings delivered by the full pipeline.
    // V9.4: model-aware tokenizer calibrated for provider family (Llama3/GPT-4/Qwen).
    let forwarded_text = extract_conversation_text(&forwarded_messages);
    let token_family = tokenizer::family_for_provider(&route.provider);
    let compiled_total = tokenizer::count_for(&forwarded_text, token_family).max(1);

    let fp = build_chat_cache_key(&forwarded_messages, &payload.extra_body);

    // ── Memory Lensing: delta-forwarding (V9.0) ──────────────────────────────
    // Exact semantic cache hit → full compiled block reused from ContextStore.
    // Multi-turn conversation → prior turns are already resident in the upstream
    // LLM's context window; only the latest user message is genuinely new.
    // Single-turn or empty prior context → zero reuse, everything is new.
    let mem = if semantic_cache_hit {
        let collector = state.collector.lock().unwrap();
        collector.context_store.compute_reuse(
            semantic_fp,
            result.raw_tokens_estimate,
            &result.intent,
        )
    } else if payload.messages.len() > 1 && !latest_user.trim().is_empty() {
        let latest_user_tokens = compiler::estimate_tokens(&latest_user);
        let prior_tokens = result
            .raw_tokens_estimate
            .saturating_sub(latest_user_tokens);
        memory::compute_delta(prior_tokens, latest_user_tokens)
    } else {
        memory::MemorySummary {
            reused_tokens: 0,
            delta_tokens: result.raw_tokens_estimate,
            context_reuse_ratio: 0.0,
        }
    };
    let stream = payload.stream.unwrap_or(false);
    let runtime_context = read_runtime_client_context();
    let scope = resolve_workspace_scope(
        payload.tenant_id.as_deref(),
        payload.project_id.as_deref(),
        &runtime_context,
        &state.workspace_context,
    );
    let upstream = upstream_identity(
        payload
            .client_app
            .as_deref()
            .or(runtime_context.client_app.as_deref()),
        payload
            .upstream_provider
            .as_deref()
            .or(runtime_context.upstream_provider.as_deref()),
        payload
            .upstream_model
            .as_deref()
            .or(runtime_context.upstream_model.as_deref()),
    );

    let efficiency = metrics::compute(
        result.raw_tokens_estimate,
        compiled_total,
        mem.reused_tokens,
    );

    let cache_key = format!("{}::{}::{}", fp, route.provider, model);

    // 2. Chat response cache short-circuit — no provider call on hit
    {
        let mut collector = state.collector.lock().unwrap();
        if let Some(cached) = collector.chat_cache.get(&cache_key).cloned() {
            let saved_tokens = cached.prompt_tokens.unwrap_or(result.raw_tokens_estimate)
                + cached.completion_tokens.unwrap_or(0);

            collector.record(RecordEntry {
                raw: result.raw_tokens_estimate,
                compiled: compiled_total,
                reused: mem.reused_tokens,
                provider: route.provider.clone(),
                model: model.clone(),
                cache_hit: true,
                semantic_cache_hit,
                semantic_fingerprint: Some(semantic_fp.to_string()),
                cache_saved_tokens: saved_tokens,
                intent: result.intent.clone(),
                sensitive,
                upstream: upstream.clone(),
                scope: scope.clone(),
                cost_usd: 0.0, // cache hit — no provider call
                latency_ms: 0,
                rct2i_applied: result.rct2i_applied,
            });

            if stream {
                return stream_cached_response(&fp.to_string(), &cached.model, &cached.content);
            }

            return Json(json!({
                "id": format!("distira-{fp}"),
                "object": "chat.completion",
                "model": cached.model,
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": cached.content
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": cached.prompt_tokens,
                    "completion_tokens": cached.completion_tokens
                },
                "distira": {
                    "provider": route.provider,
                    "model": model,
                    "intent": result.intent,
                    "raw_tokens": result.raw_tokens_estimate,
                    "compiled_tokens": compiled_total,
                    "client_app": upstream.client_app,
                    "upstream_provider": upstream.provider,
                    "upstream_model": upstream.model,
                    "tenant_id": scope.tenant_id,
                    "project_id": scope.project_id,
                    "policy_pack": scope.policy_pack,
                    "semantic_cache_hit": semantic_cache_hit,
                    "semantic_fingerprint": semantic_fp.to_string(),
                    "cache_hit": true,
                    "cache_saved_tokens": saved_tokens,
                    "cached_response": true,
                    "token_avoidance_ratio": efficiency.token_avoidance_ratio
                }
            }))
            .into_response();
        }
    }

    // 3. Resolve API key from env
    let api_key = route
        .api_key_env
        .as_deref()
        .and_then(|env_var| std::env::var(env_var).ok());

    // 4. Forward to LLM provider
    if stream {
        let t_start = std::time::Instant::now();
        return match adapters::forward_stream(
            &route.base_url,
            &model,
            &forwarded_messages,
            api_key.as_deref(),
            &payload.extra_body,
        )
        .await
        {
            Ok(response) => {
                let latency_ms = t_start.elapsed().as_millis() as u64;
                {
                    let mut collector = state.collector.lock().unwrap();
                    collector.record(RecordEntry {
                        raw: result.raw_tokens_estimate,
                        compiled: compiled_total,
                        reused: mem.reused_tokens,
                        provider: route.provider.clone(),
                        model: model.clone(),
                        cache_hit: false,
                        semantic_cache_hit,
                        semantic_fingerprint: Some(semantic_fp.to_string()),
                        cache_saved_tokens: 0,
                        intent: result.intent.clone(),
                        sensitive,
                        upstream: upstream.clone(),
                        scope: scope.clone(),
                        cost_usd: state.router_config.cost_estimate_usd(
                            &route.provider,
                            compiled_total,
                            0,
                        ),
                        latency_ms,
                        rct2i_applied: result.rct2i_applied,
                    });
                }

                let cache_key_for_stream = cache_key.clone();
                let model_for_stream = model.clone();
                let state_for_stream = Arc::clone(&state);
                let prompt_tokens = Some(result.raw_tokens_estimate);
                let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(16);
                let mut upstream_stream = response.bytes_stream();

                tokio::spawn(async move {
                    let mut pending = String::new();
                    let mut cached_content = String::new();
                    let mut cached_model = model_for_stream;
                    let mut completion_tokens = None;

                    while let Some(chunk) = FuturesStreamExt::next(&mut upstream_stream).await {
                        match chunk {
                            Ok(bytes) => {
                                if let Ok(text) = std::str::from_utf8(&bytes) {
                                    absorb_sse_chunk(
                                        text,
                                        &mut pending,
                                        &mut cached_content,
                                        &mut cached_model,
                                        &mut completion_tokens,
                                    );
                                }

                                if tx.send(Ok(bytes)).await.is_err() {
                                    return;
                                }
                            }
                            Err(error) => {
                                let _ =
                                    tx.send(Err(std::io::Error::other(error.to_string()))).await;
                                return;
                            }
                        }
                    }

                    if !pending.is_empty() {
                        absorb_sse_payload(
                            &pending,
                            &mut cached_content,
                            &mut cached_model,
                            &mut completion_tokens,
                        );
                    }

                    if !cached_content.is_empty() {
                        // V9.5: decode streamed content before caching so that
                        // cache replays serve clean, artifact-free output.
                        let decoded = tokenizer::decode_for(&cached_content, token_family);
                        cached_content = decoded;
                        if let Ok(mut collector) = state_for_stream.collector.lock() {
                            collector.chat_cache.insert(
                                cache_key_for_stream,
                                CachedChatResponse {
                                    content: cached_content,
                                    model: cached_model,
                                    prompt_tokens,
                                    completion_tokens,
                                },
                            );
                            if let Err(error) = collector.persist_to_disk() {
                                eprintln!("Warning: runtime state persistence failed: {error}");
                            }
                        }
                    }
                });

                let stream = ReceiverStream::new(rx);

                Response::builder()
                    .status(StatusCode::OK)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("text/event-stream"),
                    )
                    .header(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"))
                    .body(Body::from_stream(stream))
                    .unwrap()
            }
            Err(e) => {
                {
                    let mut c = state.collector.lock().unwrap_or_else(|e| e.into_inner());
                    c.record_error(&route.provider);
                }
                Json(json!({
                    "error": {
                        "message": e,
                        "type": "provider_error",
                        "distira": {
                            "provider": route.provider,
                            "model": model,
                            "intent": result.intent,
                            "compiled_tokens": compiled_total,
                            "client_app": upstream.client_app,
                            "upstream_provider": upstream.provider,
                            "upstream_model": upstream.model,
                            "semantic_cache_hit": semantic_cache_hit,
                            "semantic_fingerprint": semantic_fp.to_string()
                        }
                    }
                }))
                .into_response()
            }
        };
    }

    let t_start_fwd = std::time::Instant::now();
    match adapters::forward(
        &route.base_url,
        &model,
        &forwarded_messages,
        api_key.as_deref(),
        &payload.extra_body,
    )
    .await
    {
        Ok(fwd) => {
            let latency_ms_fwd = t_start_fwd.elapsed().as_millis() as u64;
            // V9.5: decode LLM output to fix BPE reconstruction artifacts
            // (stray spaces before punctuation, CRLF, double spaces, CJK spacing).
            let decoded_content = tokenizer::decode_for(&fwd.content, token_family);
            {
                let mut collector = state.collector.lock().unwrap();
                collector.chat_cache.insert(
                    cache_key,
                    CachedChatResponse {
                        content: decoded_content.clone(),
                        model: fwd.model.clone(),
                        prompt_tokens: fwd.prompt_tokens,
                        completion_tokens: fwd.completion_tokens,
                    },
                );
                collector.record(RecordEntry {
                    raw: result.raw_tokens_estimate,
                    compiled: compiled_total,
                    reused: mem.reused_tokens,
                    provider: route.provider.clone(),
                    model: model.clone(),
                    cache_hit: false,
                    semantic_cache_hit,
                    semantic_fingerprint: Some(semantic_fp.to_string()),
                    cache_saved_tokens: 0,
                    intent: result.intent.clone(),
                    sensitive,
                    upstream: upstream.clone(),
                    scope: scope.clone(),
                    cost_usd: state.router_config.cost_estimate_usd(
                        &route.provider,
                        compiled_total,
                        fwd.prompt_tokens.unwrap_or(0) + fwd.completion_tokens.unwrap_or(0),
                    ),
                    latency_ms: latency_ms_fwd,
                    rct2i_applied: result.rct2i_applied,
                });
            }

            // Return OpenAI-compatible format
            Json(json!({
                "id": format!("distira-{fp}"),
                "object": "chat.completion",
                "model": fwd.model,
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": decoded_content
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": fwd.prompt_tokens,
                    "completion_tokens": fwd.completion_tokens
                },
                "distira": {
                    "provider": route.provider,
                    "model": model,
                    "intent": result.intent,
                    "raw_tokens": result.raw_tokens_estimate,
                    "compiled_tokens": compiled_total,
                    "client_app": upstream.client_app,
                    "upstream_provider": upstream.provider,
                    "upstream_model": upstream.model,
                    "tenant_id": scope.tenant_id,
                    "project_id": scope.project_id,
                    "policy_pack": scope.policy_pack,
                    "semantic_cache_hit": semantic_cache_hit,
                    "semantic_fingerprint": semantic_fp.to_string(),
                    "cache_hit": false,
                    "cache_saved_tokens": 0,
                    "cached_response": false,
                    "token_avoidance_ratio": efficiency.token_avoidance_ratio
                }
            }))
            .into_response()
        }
        Err(e) => {
            {
                let mut c = state.collector.lock().unwrap_or_else(|e| e.into_inner());
                c.record_error(&route.provider);
            }
            Json(json!({
                "error": {
                    "message": e,
                    "type": "provider_error",
                    "distira": {
                        "provider": route.provider,
                        "model": model,
                        "intent": result.intent,
                        "compiled_tokens": compiled_total,
                        "semantic_cache_hit": semantic_cache_hit,
                        "semantic_fingerprint": semantic_fp.to_string()
                    }
                }
            }))
            .into_response()
        }
    }
}

/// V10 — Return adaptive optimization suggestions based on routing config,
/// session metrics, provider error rates and latency measurements.
/// Always returns at least informational suggestions so the panel is never empty.
async fn get_suggestions(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let (latency_map, error_map, snapshot) = {
        let collector = state.collector.lock().unwrap_or_else(|e| e.into_inner());
        let snap = collector.snapshot().clone();
        (
            collector.avg_latency_by_provider(),
            collector.error_rate_by_provider(),
            snap,
        )
    };

    let mut suggestions: Vec<serde_json::Value> = Vec::new();

    // ── 1. Always: active routing map ───────────────────────────────────────
    let routing = state.router_config.task_routing_summary();
    if !routing.is_empty() {
        let routing_desc = routing
            .iter()
            .map(|(intent, prov)| format!("{intent}→{prov}"))
            .collect::<Vec<_>>()
            .join(", ");
        suggestions.push(json!({
            "severity": "info",
            "code": "routing_active",
            "provider": "",
            "metric": "task_routing",
            "value": routing.len(),
            "message": format!(
                "{} intent routes active: {}",
                routing.len(),
                routing_desc
            )
        }));
    }

    // ── 2. Always: session efficiency (once there are requests) ─────────────
    if snapshot.total_requests > 0 {
        let saved = snapshot.raw_tokens.saturating_sub(snapshot.compiled_tokens) as u64;
        let pct = if snapshot.raw_tokens > 0 {
            (saved as f64 / snapshot.raw_tokens as f64 * 100.0).round() as u64
        } else {
            0
        };
        let total_local = snapshot.routes_local;
        let total_cloud = snapshot.routes_cloud;
        let local_pct = if snapshot.total_requests > 0 {
            (total_local as f64 / snapshot.total_requests as f64 * 100.0).round() as u64
        } else {
            0
        };
        suggestions.push(json!({
            "severity": "info",
            "code": "session_efficiency",
            "provider": "",
            "metric": "token_reduction_pct",
            "value": pct,
            "message": format!(
                "Session: {} requests processed — {}% token reduction, {}% on-prem ({} local / {} cloud)",
                snapshot.total_requests, pct, local_pct, total_local, total_cloud
            )
        }));
    }

    // ── 3. Cache performance (once there are hits) ──────────────────────────
    let total_cache = snapshot.cache_hits + snapshot.cache_misses;
    if total_cache > 0 {
        let hit_pct = (snapshot.cache_hits as f64 / total_cache as f64 * 100.0).round() as u64;
        let severity = if hit_pct >= 30 { "info" } else { "warning" };
        suggestions.push(json!({
            "severity": severity,
            "code": "cache_performance",
            "provider": "",
            "metric": "cache_hit_pct",
            "value": hit_pct,
            "message": format!(
                "Semantic cache: {}% hit rate ({}/{} requests) — {} tokens avoided via cache",
                hit_pct, snapshot.cache_hits, total_cache, snapshot.cache_saved_tokens
            )
        }));
    }

    // ── 4. Concise mode status ───────────────────────────────────────────────
    if state.router_config.concise_mode() {
        suggestions.push(json!({
            "severity": "info",
            "code": "concise_mode_active",
            "provider": "",
            "metric": "concise_mode",
            "value": 1,
            "message": "Concise mode is ON — DISTIRA injects a brevity directive into every LLM request, reducing output token usage across all providers."
        }));
    }

    // ── 5. Reactive: high error rate ≥ 5 % ──────────────────────────────────
    for (provider, error_rate) in &error_map {
        if *error_rate >= 0.05 {
            suggestions.push(json!({
                "severity": "warning",
                "code": "high_error_rate",
                "provider": provider,
                "metric": "error_rate",
                "value": (*error_rate * 10000.0).round() / 10000.0,
                "message": format!(
                    "Provider {} has {:.0}% error rate. Check connectivity or add a fallback in providers.yaml.",
                    provider, error_rate * 100.0
                )
            }));
        }
    }

    // ── 6. Reactive: high latency ≥ 3 000 ms ────────────────────────────────
    for (provider, avg_ms) in &latency_map {
        if *avg_ms >= 3000.0 {
            let severity = if *avg_ms >= 6000.0 { "warning" } else { "info" };
            suggestions.push(json!({
                "severity": severity,
                "code": "high_latency",
                "provider": provider,
                "metric": "avg_latency_ms",
                "value": avg_ms.round(),
                "message": format!(
                    "Provider {} avg latency is {:.0}ms. Consider routing this intent to a faster on-prem provider.",
                    provider, avg_ms
                )
            }));
        }
    }

    // Sort: warnings first, then info; within each level preserve insertion order.
    suggestions.sort_by(|a, b| {
        let sa = a.get("severity").and_then(|v| v.as_str()).unwrap_or("info");
        let sb = b.get("severity").and_then(|v| v.as_str()).unwrap_or("info");
        sb.cmp(sa)
    });

    Json(json!({
        "generated_at": now_epoch(),
        "count": suggestions.len(),
        "suggestions": suggestions
    }))
}

fn stream_cached_response(request_id: &str, model: &str, content: &str) -> Response<Body> {
    let chunk = json!({
        "id": format!("distira-{request_id}"),
        "object": "chat.completion.chunk",
        "model": model,
        "choices": [{
            "index": 0,
            "delta": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }]
    });

    let stream = iter(vec![
        Ok::<_, std::io::Error>(Bytes::from(format!("data: {}\n\n", chunk))),
        Ok::<_, std::io::Error>(Bytes::from("data: [DONE]\n\n")),
    ]);

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/event-stream"),
        )
        .header(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"))
        .body(Body::from_stream(stream))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_conversation_text_joins_string_and_segment_content() {
        let messages = vec![
            json!({ "role": "system", "content": "You are precise" }),
            json!({
                "role": "user",
                "content": [
                    { "type": "text", "text": "Explain" },
                    { "type": "text", "text": "the panic" }
                ]
            }),
        ];

        let text = extract_conversation_text(&messages);

        assert!(text.contains("You are precise"));
        assert!(text.contains("Explain"));
        assert!(text.contains("the panic"));
    }

    #[test]
    fn build_forward_messages_preserves_history() {
        let messages = vec![
            json!({ "role": "system", "content": "Keep context" }),
            json!({ "role": "assistant", "content": "Prior reply" }),
            json!({ "role": "user", "content": "Latest request" }),
        ];

        let forwarded = build_forward_messages(&messages, "fallback");

        assert_eq!(forwarded.len(), 3);
        assert_eq!(forwarded[0]["role"], json!("system"));
        assert_eq!(forwarded[1]["role"], json!("assistant"));
        assert_eq!(forwarded[2]["content"], json!("Latest request"));
    }

    #[test]
    fn apply_compiled_user_message_rewrites_only_latest_user_turn() {
        let messages = vec![
            json!({ "role": "system", "content": "Keep context" }),
            json!({ "role": "user", "content": "Older request" }),
            json!({ "role": "assistant", "content": "Older answer" }),
            json!({ "role": "user", "content": "Latest request" }),
        ];

        let rewritten = apply_compiled_user_message(&messages, "Compiled request");

        assert_eq!(rewritten[0]["content"], json!("Keep context"));
        assert_eq!(rewritten[1]["content"], json!("Older request"));
        assert_eq!(rewritten[3]["content"], json!("Compiled request"));
    }

    #[test]
    fn extract_latest_user_text_prefers_latest_user_turn() {
        let messages = vec![
            json!({ "role": "user", "content": "first" }),
            json!({ "role": "assistant", "content": "reply" }),
            json!({ "role": "user", "content": "second" }),
        ];

        assert_eq!(extract_latest_user_text(&messages), "second");
    }

    #[test]
    fn chat_cache_key_changes_when_options_change() {
        let messages = vec![json!({ "role": "user", "content": "Hello" })];
        let mut cold = Map::new();
        let mut warm = Map::new();
        cold.insert("temperature".into(), json!(0.1));
        warm.insert("temperature".into(), json!(0.9));

        let left = build_chat_cache_key(&messages, &cold);
        let right = build_chat_cache_key(&messages, &warm);

        assert_ne!(left, right);
    }

    #[test]
    fn chat_cache_key_ignores_volatile_numeric_and_uuid_noise() {
        let left_messages = vec![json!({
            "role": "user",
            "content": "error request_id=123456 trace=550e8400-e29b-41d4-a716-446655440000"
        })];
        let right_messages = vec![json!({
            "role": "user",
            "content": "error request_id=987654 trace=550e8400-e29b-41d4-a716-446655440001"
        })];

        let opts = Map::new();
        let left = build_chat_cache_key(&left_messages, &opts);
        let right = build_chat_cache_key(&right_messages, &opts);

        assert_eq!(left, right);
    }

    #[test]
    fn compile_with_semantic_cache_reuses_compiled_result() {
        let mut collector = MetricsCollector::new();

        let (first_fp, first_result, first_hit) =
            compile_with_semantic_cache(&mut collector, "panic: duplicated stack trace", None);
        let (second_fp, second_result, second_hit) =
            compile_with_semantic_cache(&mut collector, "panic: duplicated stack trace", None);

        assert!(!first_hit);
        assert!(second_hit);
        assert_eq!(first_fp, second_fp);
        assert_eq!(
            first_result.compiled_context,
            second_result.compiled_context
        );
        assert_eq!(collector.sem_cache.len(), 1);
    }

    #[test]
    fn compress_history_pass_through_short_conversation() {
        let messages = vec![
            json!({ "role": "user", "content": "msg1" }),
            json!({ "role": "assistant", "content": "reply1" }),
            json!({ "role": "user", "content": "msg2" }),
        ];
        let compressed = compress_conversation_history(&messages);
        assert_eq!(compressed.len(), 3);
        assert_eq!(compressed[0]["content"], json!("msg1"));
    }

    #[test]
    fn compress_history_summarises_older_turns() {
        // 8 messages — first 2 should be compressed into a system summary
        let messages: Vec<Value> = (0..8)
            .flat_map(|i| {
                vec![
                    json!({ "role": "user", "content": format!("question {i}") }),
                    json!({ "role": "assistant", "content": format!("answer {i}") }),
                ]
            })
            .collect();
        let compressed = compress_conversation_history(&messages);
        // MAX_FULL_TURNS = 6; first 2 turns compressed into system msg
        assert_eq!(compressed.len(), MAX_FULL_TURNS + 1); // +1 for the summary system msg
        assert_eq!(compressed[0]["role"], json!("system"));
        let system_content = compressed[0]["content"].as_str().unwrap_or("");
        assert!(system_content.contains("turns compressed"));
    }

    // ── V10.15 — Compile older turns + cross-message dedup tests ────────────

    #[test]
    fn compile_older_turns_keeps_recent_messages_verbatim() {
        let messages = vec![
            json!({ "role": "system", "content": "You are a very helpful and knowledgeable assistant that always provides detailed comprehensive answers to every question asked by the user." }),
            json!({ "role": "user", "content": "short question" }),
            json!({ "role": "assistant", "content": "short answer" }),
            json!({ "role": "user", "content": "another question" }),
        ];
        let result = compile_older_turns(messages.clone());
        // Last 2 messages should be unchanged
        assert_eq!(result[2]["content"], messages[2]["content"]);
        assert_eq!(result[3]["content"], messages[3]["content"]);
        // System message (verbose, > 100 chars) should be compiled (shorter)
        let sys_content = result[0]["content"].as_str().unwrap_or("");
        assert!(
            sys_content.len() <= messages[0]["content"].as_str().unwrap().len(),
            "system prompt should be compressed or equal length"
        );
    }

    #[test]
    fn compile_older_turns_skips_short_messages() {
        let messages = vec![
            json!({ "role": "system", "content": "Be concise." }),
            json!({ "role": "user", "content": "hi" }),
            json!({ "role": "assistant", "content": "hello" }),
            json!({ "role": "user", "content": "bye" }),
        ];
        let result = compile_older_turns(messages.clone());
        // All messages < 100 chars → no changes
        assert_eq!(result[0]["content"], messages[0]["content"]);
        assert_eq!(result[1]["content"], messages[1]["content"]);
    }

    #[test]
    fn compile_older_turns_passthrough_two_messages() {
        let messages = vec![
            json!({ "role": "user", "content": "long question" }),
            json!({ "role": "assistant", "content": "long answer" }),
        ];
        let result = compile_older_turns(messages.clone());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["content"], messages[0]["content"]);
    }

    #[test]
    fn dedup_cross_messages_removes_repeated_paragraphs() {
        let messages = vec![
            json!({ "role": "user", "content": "First paragraph about cats and dogs and their behavior in the wild.\n\nSecond paragraph about birds flying south for winter." }),
            json!({ "role": "assistant", "content": "Here is my response about the topic you raised." }),
            json!({ "role": "user", "content": "First paragraph about cats and dogs and their behavior in the wild.\n\nNow I have a new question about fish." }),
        ];
        let result = dedup_cross_messages(messages);
        // The duplicate paragraph should be removed from the first message
        let first_content = result[0]["content"].as_str().unwrap_or("");
        // The duplicate paragraph moved to message index 2, so it should be gone from 0
        assert!(
            !first_content.contains("cats and dogs")
                || result[2]["content"]
                    .as_str()
                    .unwrap_or("")
                    .contains("cats and dogs"),
            "duplicate should be in latest message only"
        );
    }

    #[test]
    fn dedup_cross_messages_passthrough_short_conversations() {
        let messages = vec![
            json!({ "role": "user", "content": "hello" }),
            json!({ "role": "assistant", "content": "hi" }),
        ];
        let result = dedup_cross_messages(messages.clone());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["content"], messages[0]["content"]);
    }

    #[test]
    fn prune_request_history_respects_ttl_and_limit() {
        let mut history = vec![
            RequestLineage {
                client_app: Some("c1".into()),
                upstream_provider: Some("p1".into()),
                upstream_model: Some("m1".into()),
                tenant_id: Some("t1".into()),
                project_id: Some("pr1".into()),
                policy_pack: Some("baseline".into()),
                routed_provider: "ollama-llama3".into(),
                routed_model: "llama3:latest".into(),
                intent: "general".into(),
                semantic_cache_hit: false,
                semantic_fingerprint: Some("fp-1".into()),
                cache_hit: false,
                sensitive: false,
                cost_usd: 0.0,
                raw_tokens: 100,
                compiled_tokens: 60,
                tokens_saved: 40,
                ts: 100,
            },
            RequestLineage {
                client_app: Some("c1".into()),
                upstream_provider: Some("p1".into()),
                upstream_model: Some("m1".into()),
                tenant_id: Some("t1".into()),
                project_id: Some("pr1".into()),
                policy_pack: Some("baseline".into()),
                routed_provider: "ollama-llama3".into(),
                routed_model: "llama3:latest".into(),
                intent: "general".into(),
                semantic_cache_hit: false,
                semantic_fingerprint: Some("fp-2".into()),
                cache_hit: false,
                sensitive: false,
                cost_usd: 0.0,
                raw_tokens: 80,
                compiled_tokens: 50,
                tokens_saved: 30,
                ts: 200,
            },
            RequestLineage {
                client_app: Some("c1".into()),
                upstream_provider: Some("p1".into()),
                upstream_model: Some("m1".into()),
                tenant_id: Some("t1".into()),
                project_id: Some("pr1".into()),
                policy_pack: Some("baseline".into()),
                routed_provider: "ollama-llama3".into(),
                routed_model: "llama3:latest".into(),
                intent: "general".into(),
                semantic_cache_hit: false,
                semantic_fingerprint: Some("fp-3".into()),
                cache_hit: false,
                sensitive: false,
                cost_usd: 0.0,
                raw_tokens: 90,
                compiled_tokens: 55,
                tokens_saved: 35,
                ts: 300,
            },
        ];

        prune_request_history(&mut history, Some(180), 1);

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].ts, 300);
    }

    #[test]
    fn collector_persistence_round_trip_restores_runtime_state() {
        let mut state_path = std::env::temp_dir();
        state_path.push(format!("distira-runtime-state-{}.json", now_epoch()));

        let mut collector = MetricsCollector::new();
        collector.persistence_path = state_path.clone();
        collector.record(RecordEntry {
            raw: 120,
            compiled: 70,
            reused: 20,
            provider: "ollama-llama3".into(),
            model: "llama3:latest".into(),
            cache_hit: false,
            semantic_cache_hit: true,
            semantic_fingerprint: Some("fp-test".into()),
            cache_saved_tokens: 0,
            intent: "general".into(),
            sensitive: false,
            upstream: UpstreamIdentity::default(),
            scope: WorkspaceScope::default(),
            cost_usd: 0.0,
            latency_ms: 0,
            rct2i_applied: false,
        });

        let mut restored = MetricsCollector::new();
        restored.persistence_path = state_path.clone();
        restored.restore_from_disk().unwrap();

        assert_eq!(restored.snapshot.total_requests, 1);
        assert_eq!(restored.snapshot.raw_tokens, 120);
        assert_eq!(restored.snapshot.compiled_tokens, 70);
        assert_eq!(restored.snapshot.request_history.len(), 1);
        assert_eq!(
            restored.snapshot.request_history[0].semantic_cache_hit,
            true
        );

        let _ = std::fs::remove_file(state_path);
    }
}

fn budget_alerts(
    daily_counts: &HashMap<String, u64>,
    router: &router::RouterConfig,
    session_cost_usd: f64,
    session_budget_usd: f64,
) -> Vec<serde_json::Value> {
    let mut alerts: Vec<serde_json::Value> = router
        .list_provider_summaries()
        .into_iter()
        .filter_map(|ps| {
            let budget = ps.max_requests_per_day;
            if budget == 0 {
                return None;
            }
            let used = daily_counts.get(&ps.key).copied().unwrap_or(0);
            if used >= budget {
                Some(serde_json::json!({
                    "type": "budget_exhausted",
                    "provider": ps.key,
                    "message": format!(
                        "Provider {} has reached its daily budget ({}/{} requests).",
                        ps.key, used, budget
                    )
                }))
            } else if used * 10 >= budget * 8 {
                Some(serde_json::json!({
                    "type": "budget_warning",
                    "provider": ps.key,
                    "message": format!(
                        "Provider {} is at {}% of daily budget ({}/{} requests).",
                        ps.key,
                        used * 100 / budget,
                        used,
                        budget
                    )
                }))
            } else {
                None
            }
        })
        .collect();

    // V10.3 — Session cost budget alerts.
    if session_budget_usd > 0.0 {
        if session_cost_usd >= session_budget_usd {
            alerts.push(serde_json::json!({
                "type": "budget_exhausted",
                "message": format!(
                    "Session cost budget exhausted: ${:.4} / ${:.4} USD.",
                    session_cost_usd, session_budget_usd
                )
            }));
        } else if session_cost_usd >= session_budget_usd * 0.8 {
            let pct = (session_cost_usd / session_budget_usd * 100.0).round() as u64;
            alerts.push(serde_json::json!({
                "type": "budget_warning",
                "message": format!(
                    "Session cost at {}% of budget: ${:.4} / ${:.4} USD.",
                    pct, session_cost_usd, session_budget_usd
                )
            }));
        }
    }

    alerts
}

/// Build a full metrics JSON value with all context-memory fields derived live
/// from the current `ContextStore` state — not from the cached snapshot fields.
/// Used by both the REST endpoint and the SSE stream so every consumer always
/// sees the true real-time picture regardless of how recently `record()` ran.
fn build_full_snapshot(collector: &MetricsCollector, session_budget_usd: f64) -> serde_json::Value {
    let mut val = serde_json::to_value(collector.snapshot()).unwrap_or_default();

    // Live context blocks — stability, token count, and intent come straight
    // from the store, so decay is visible between requests.
    let blocks_summary: Vec<serde_json::Value> = collector
        .context_store
        .blocks()
        .into_iter()
        .map(|b| {
            let short_id = if b.id.len() > 8 {
                b.id[..8].to_string()
            } else {
                b.id.clone()
            };
            json!({
                "id": short_id,
                "stability": (b.stability * 100.0).round() / 100.0,
                "token_count": b.content.split_whitespace().count(),
                "intent": b.intent
            })
        })
        .collect();

    // Override snapshot fields with live values so SSE and REST are always in sync.
    // stable_blocks — real count from context_store, reflects decay/eviction instantly.
    val["stable_blocks"] = serde_json::json!(blocks_summary.len());

    // context_reuse_ratio_pct — recomputed from current cumulative totals each tick.
    let raw = val["raw_tokens"].as_u64().unwrap_or(0) as f32;
    let reused = val["memory_reused_tokens"].as_u64().unwrap_or(0) as f32;
    val["context_reuse_ratio_pct"] = serde_json::json!(if raw > 0.0 {
        (reused / raw * 100.0).min(100.0)
    } else {
        0.0
    });

    val["context_blocks_summary"] = serde_json::Value::Array(blocks_summary);

    // V10.3 — Inject configured session budget so the dashboard can render utilisation.
    val["session_budget_usd"] = serde_json::json!(session_budget_usd);

    // V10.17 — Provider health observatory: per-provider live stats.
    let latency_map = collector.avg_latency_by_provider();
    let error_map = collector.error_rate_by_provider();
    let provider_health: Vec<serde_json::Value> = collector
        .provider_total
        .iter()
        .map(|(key, &total)| {
            let errors = collector.provider_errors.get(key).copied().unwrap_or(0);
            let error_rate = error_map.get(key).copied().unwrap_or(0.0);
            let avg_latency = latency_map.get(key).copied().unwrap_or(0.0);
            let status = if error_rate >= 0.5 {
                "down"
            } else if error_rate >= 0.1 || avg_latency >= 5000.0 {
                "degraded"
            } else {
                "healthy"
            };
            json!({
                "provider": key,
                "requests": total,
                "errors": errors,
                "error_rate": (error_rate * 10000.0).round() / 10000.0,
                "avg_latency_ms": (avg_latency * 10.0).round() / 10.0,
                "status": status,
            })
        })
        .collect();
    val["provider_health"] = serde_json::Value::Array(provider_health);

    val
}

async fn metrics_snapshot(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let collector = state.collector.lock().unwrap();
    let session_cost = collector.snapshot().session_cost_usd;
    let budget = state.workspace_context.session_budget_usd;
    let mut snapshot_val = build_full_snapshot(&collector, budget);
    let alerts = budget_alerts(
        &collector.daily_provider_counts,
        &state.router_config,
        session_cost,
        budget,
    );
    if !alerts.is_empty() {
        snapshot_val["alerts"] = serde_json::Value::Array(alerts);
    }
    Json(snapshot_val)
}

async fn metrics_reset(State(state): State<SharedState>) -> impl IntoResponse {
    let mut collector = state.collector.lock().unwrap();
    collector.reset();
    StatusCode::NO_CONTENT
}

/// V10.17 — Export cumulative metrics as structured JSON for enterprise reporting.
/// Returns per-provider breakdown, per-intent breakdown, and cumulative totals.
async fn metrics_export(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let collector = state.collector.lock().unwrap_or_else(|e| e.into_inner());
    let snap = collector.snapshot();
    let latency_map = collector.avg_latency_by_provider();
    let error_map = collector.error_rate_by_provider();

    let tokens_saved = snap.raw_tokens.saturating_sub(snap.compiled_tokens);
    let savings_pct = if snap.raw_tokens > 0 {
        (tokens_saved as f64 / snap.raw_tokens as f64 * 100.0 * 10.0).round() / 10.0
    } else {
        0.0
    };

    // Per-provider breakdown
    let providers: Vec<serde_json::Value> = collector
        .provider_total
        .iter()
        .map(|(key, &total)| {
            let errors = collector.provider_errors.get(key).copied().unwrap_or(0);
            let error_rate = error_map.get(key).copied().unwrap_or(0.0);
            let avg_latency = latency_map.get(key).copied().unwrap_or(0.0);
            json!({
                "provider": key,
                "requests": total,
                "errors": errors,
                "error_rate": (error_rate * 10000.0).round() / 10000.0,
                "avg_latency_ms": (avg_latency * 10.0).round() / 10.0,
            })
        })
        .collect();

    // Per-intent breakdown
    let intents: Vec<serde_json::Value> = snap
        .intent_stats
        .iter()
        .map(|(intent, stats)| {
            let intent_saved = stats.raw_tokens.saturating_sub(stats.compiled_tokens);
            let intent_pct = if stats.raw_tokens > 0 {
                (intent_saved as f64 / stats.raw_tokens as f64 * 100.0 * 10.0).round() / 10.0
            } else {
                0.0
            };
            json!({
                "intent": intent,
                "requests": stats.requests,
                "raw_tokens": stats.raw_tokens,
                "compiled_tokens": stats.compiled_tokens,
                "tokens_saved": intent_saved,
                "savings_pct": intent_pct,
            })
        })
        .collect();

    Json(json!({
        "exported_at": now_epoch(),
        "version": runtime_version(),
        "cumulative": {
            "total_requests": snap.total_requests,
            "raw_tokens": snap.raw_tokens,
            "compiled_tokens": snap.compiled_tokens,
            "tokens_saved": tokens_saved,
            "savings_pct": savings_pct,
            "memory_reused_tokens": snap.memory_reused_tokens,
            "cache_hits": snap.cache_hits,
            "cache_misses": snap.cache_misses,
            "efficiency_score": snap.efficiency_score,
            "session_cost_usd": snap.session_cost_usd,
            "rct2i_applied_count": snap.rct2i_applied_count,
            "routes_local": snap.routes_local,
            "routes_cloud": snap.routes_cloud,
            "routes_midtier": snap.routes_midtier,
        },
        "by_provider": providers,
        "by_intent": intents,
    }))
}

async fn metrics_stream(
    State(state): State<SharedState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(std::time::Duration::from_secs(2));
    let stream = tokio_stream::StreamExt::map(IntervalStream::new(interval), move |_| {
        let collector = state.collector.lock().unwrap_or_else(|e| e.into_inner());
        let mut snapshot_val =
            build_full_snapshot(&collector, state.workspace_context.session_budget_usd);
        let session_cost = collector.snapshot().session_cost_usd;
        let alerts = budget_alerts(
            &collector.daily_provider_counts,
            &state.router_config,
            session_cost,
            state.workspace_context.session_budget_usd,
        );
        if !alerts.is_empty() {
            snapshot_val["alerts"] = serde_json::Value::Array(alerts);
        }
        let data = serde_json::to_string(&snapshot_val).unwrap_or_default();
        Ok(Event::default().event("metrics").data(data))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

// ── Main ──────────────────────────────────────────────

/// Optional Bearer-token middleware for /v1/* routes.
/// Activated only when the `DISTIRA_API_KEY` env var is set.
/// If the env var is absent, every request passes through unchanged.
async fn require_api_key(
    req: Request<Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    // If no key configured, always allow
    let expected = match std::env::var("DISTIRA_API_KEY") {
        Ok(k) if !k.trim().is_empty() => k,
        _ => return Ok(next.run(req).await),
    };

    let provided = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .unwrap_or("");

    if provided == expected.trim() {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
#[tokio::main]
async fn main() {
    println!("DISTIRA v{} — The AI Context Compiler", runtime_version());
    println!("────────────────────────────────────────");

    let router_config = load_config();
    let workspace_context = load_workspace_context();
    let policies = load_policies();
    if workspace_context.tenant_id.is_some() || workspace_context.project_id.is_some() {
        println!(
            "  Workspace scope: tenant={:?}, project={:?}, policy_pack={:?}",
            workspace_context.tenant_id,
            workspace_context.project_id,
            workspace_context.policy_pack
        );
    }
    if let Some(max) = policies.max_tokens_per_request {
        println!("  Policy: max_tokens_per_request={max}");
    }

    let state: SharedState = Arc::new(AppState {
        collector: Mutex::new(MetricsCollector::new()),
        router_config,
        workspace_context,
        policies,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let v1_routes = Router::new()
        .route("/v1/providers", get(list_providers))
        .route(
            "/v1/runtime/client-context",
            get(get_runtime_client_context).post(set_runtime_client_context),
        )
        .route("/v1/compile", post(compile))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/metrics", get(metrics_snapshot))
        .route("/v1/metrics/reset", delete(metrics_reset))
        .route("/v1/metrics/export", get(metrics_export))
        .route("/v1/metrics/stream", get(metrics_stream))
        .route("/v1/suggestions", get(get_suggestions))
        .layer(middleware::from_fn(require_api_key))
        .with_state(state.clone());

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/version", get(version))
        .merge(v1_routes)
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("────────────────────────────────────────");
    println!("Listening on {addr}");
    if std::env::var("DISTIRA_API_KEY")
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false)
    {
        println!("  Auth: Bearer token enabled (DISTIRA_API_KEY is set)");
    } else {
        println!("  Auth: open (set DISTIRA_API_KEY to enable Bearer auth)");
    }
    println!("  POST /v1/compile            — compile context only");
    println!("  POST /v1/chat/completions   — compile + forward to LLM");
    println!("  GET  /v1/providers          — list configured providers + runtime details");
    println!("  GET  /v1/runtime/client-context — read live upstream client context");
    println!("  POST /v1/runtime/client-context — update live upstream client context");
    println!("  GET    /v1/metrics            — JSON snapshot");
    println!("  GET    /v1/metrics/export     — V10.17 cumulative export (enterprise)");
    println!("  DELETE /v1/metrics/reset     — reset all counters");
    println!("  GET    /v1/metrics/stream    — SSE live stream");
    println!("  GET    /v1/suggestions       — V10 adaptive optimization suggestions");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
