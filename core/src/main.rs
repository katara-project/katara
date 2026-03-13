use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{header, HeaderValue, Response, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::StreamExt as FuturesStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio_stream::iter;
use tokio_stream::wrappers::{IntervalStream, ReceiverStream};
use tower_http::cors::{Any, CorsLayer};

/// ── Shared application state ──────────────────────────
#[derive(Debug, Clone, Serialize)]
struct IntentStats {
    requests: u64,
    raw_tokens: usize,
    compiled_tokens: usize,
}

#[derive(Debug, Clone, Serialize)]
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
}

#[derive(Debug, Clone, Serialize)]
struct UpstreamStats {
    client_app: String,
    upstream_provider: String,
    upstream_model: String,
    requests: u64,
    last_seen_ts: u64,
}

#[derive(Debug, Clone, Serialize)]
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
    cache_hit: bool,
    sensitive: bool,
    ts: u64,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceContextFile {
    workspace: WorkspaceContext,
}

#[derive(Debug, Clone, Serialize)]
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
}

#[derive(Debug, Clone)]
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
    cache_saved_tokens: usize,
    intent: String,
    sensitive: bool,
    upstream: UpstreamIdentity,
    scope: WorkspaceScope,
}

#[derive(Debug, Clone, Default)]
struct WorkspaceScope {
    tenant_id: Option<String>,
    project_id: Option<String>,
    policy_pack: Option<String>,
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
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
            },
            sem_cache: cache::SemanticCache::new(),
            chat_cache: HashMap::new(),
            context_store: memory::ContextStore::new(),
            audit_retention_secs: read_u64_env("KATARA_AUDIT_RETENTION_DAYS", 7)
                .saturating_mul(24 * 60 * 60),
            audit_history_limit: read_usize_env("KATARA_AUDIT_HISTORY_LIMIT", 2000),
            hour_buckets: HashMap::new(),
        }
    }

    fn record(&mut self, e: RecordEntry) {
        let RecordEntry {
            raw,
            compiled,
            reused,
            provider,
            model,
            cache_hit,
            cache_saved_tokens,
            intent,
            sensitive,
            upstream,
            scope,
        } = e;
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

        // Classify deployment type from provider name
        if is_sovereign_provider(&provider) {
            s.routes_local += 1;
        } else if is_midtier_provider(&provider) {
            s.routes_midtier += 1;
        } else {
            s.routes_cloud += 1;
        }

        let avoided = s.raw_tokens.saturating_sub(s.compiled_tokens);
        s.efficiency_score = if s.raw_tokens == 0 {
            0.0
        } else {
            (avoided as f32 / s.raw_tokens as f32) * 100.0
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
            (model_avoided as f32 / model_entry.raw_tokens as f32) * 100.0
        };
        let model_total_routes =
            model_entry.sovereign_requests + model_entry.non_sovereign_requests;
        model_entry.sovereign_ratio = if model_total_routes == 0 {
            0.0
        } else {
            (model_entry.sovereign_requests as f32 / model_total_routes as f32) * 100.0
        };

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
            cache_hit,
            sensitive,
            ts,
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
    }

    fn snapshot(&self) -> &MetricsSnapshot {
        &self.snapshot
    }
}

/// Combined shared state
struct AppState {
    collector: Mutex<MetricsCollector>,
    router_config: router::RouterConfig,
    workspace_context: WorkspaceContext,
}

type SharedState = Arc<AppState>;

fn compile_result_from_cache(entry: &cache::CacheEntry) -> compiler::CompileResult {
    compiler::CompileResult {
        intent: entry.intent.clone(),
        raw_tokens_estimate: entry.raw_tokens_estimate,
        compiled_tokens_estimate: entry.compiled_tokens_estimate,
        summary: entry.summary.clone(),
        compiled_context: entry.compiled_context.clone(),
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
    }
}

fn compile_with_semantic_cache(
    collector: &mut MetricsCollector,
    raw: &str,
) -> (u64, compiler::CompileResult, bool) {
    let fingerprint = fingerprint::fingerprint(raw);
    if let Some(entry) = collector.sem_cache.get(fingerprint) {
        return (fingerprint, compile_result_from_cache(entry), true);
    }

    let result = compiler::compile_context(raw);
    collector
        .sem_cache
        .insert(cache_entry_from_compile_result(fingerprint, &result));
    // Register compiled context in real memory store for future reuse tracking
    collector
        .context_store
        .register(fingerprint, &result.compiled_context);
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
            if let Ok(context) = serde_yaml::from_str::<WorkspaceContext>(&raw) {
                return context;
            }
            if let Ok(wrapped) = serde_yaml::from_str::<WorkspaceContextFile>(&raw) {
                return wrapped.workspace;
            }
        }
    }

    WorkspaceContext {
        tenant_id: None,
        project_id: None,
        policy_pack: None,
    }
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
const MAX_FULL_TURNS: usize = 6;

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
            let short: String = if content.split_whitespace().count() > 20 {
                content
                    .split_whitespace()
                    .take(20)
                    .collect::<Vec<_>>()
                    .join(" ")
                    + " [...]"
            } else {
                content
            };
            Some(format!("[{role}]: {short}"))
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
    let serialized = serde_json::to_string(&json!({
        "messages": messages,
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
    Json(json!({ "status": "ok", "service": "katara-core", "version": runtime_version() }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({ "version": runtime_version(), "product": "KATARA" }))
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
    Json(json!({
        "providers": state.router_config.list_providers(),
        "provider_details": state.router_config.list_provider_summaries()
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

    let mut collector = state.collector.lock().unwrap();
    let (fp, result, cache_hit) = compile_with_semantic_cache(&mut collector, raw);
    let mem = collector
        .context_store
        .compute_reuse(fp, result.raw_tokens_estimate);
    let runtime_context = read_runtime_client_context();
    let scope = resolve_workspace_scope(
        payload.tenant_id.as_deref(),
        payload.project_id.as_deref(),
        &runtime_context,
        &state.workspace_context,
    );
    let route = state
        .router_config
        .choose_provider(&result.intent, sensitive);
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
        cache_saved_tokens: 0,
        intent: result.intent.clone(),
        sensitive,
        upstream: upstream.clone(),
        scope: scope.clone(),
    });
    drop(collector);

    Json(json!({
        "fingerprint": fp.to_string(),
        "cache_hit": cache_hit,
        "intent": result.intent,
        "raw_tokens": result.raw_tokens_estimate,
        "compiled_tokens": result.compiled_tokens_estimate,
        "compiled_context": result.compiled_context,
        "summary": result.summary,
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
        "token_avoidance_ratio": efficiency.token_avoidance_ratio
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
    let raw = extract_latest_user_text(&payload.messages);
    let sensitive = payload.sensitive.unwrap_or(false);

    // 1. Full KATARA pipeline
    let compile_input = if raw.trim().is_empty() {
        extract_conversation_text(&payload.messages)
    } else {
        raw.clone()
    };
    let (semantic_fp, result, semantic_cache_hit) = {
        let mut collector = state.collector.lock().unwrap();
        compile_with_semantic_cache(&mut collector, &compile_input)
    };
    let compiled_prompt = if result.compiled_context.trim().is_empty() {
        compile_input.clone()
    } else {
        result.compiled_context.clone()
    };
    let compiled_messages = if payload.messages.is_empty() {
        build_forward_messages(&payload.messages, &compiled_prompt)
    } else {
        apply_compiled_user_message(&payload.messages, &compiled_prompt)
    };
    // Compress history when conversation has grown beyond MAX_FULL_TURNS
    let forwarded_messages = compress_conversation_history(&compiled_messages);
    let fp = build_chat_cache_key(&forwarded_messages, &payload.extra_body);
    let mem = {
        let collector = state.collector.lock().unwrap();
        collector
            .context_store
            .compute_reuse(semantic_fp, result.raw_tokens_estimate)
    };
    let route = state
        .router_config
        .choose_provider(&result.intent, sensitive);
    let model = payload.model.clone().unwrap_or_else(|| route.model.clone());
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
        result.compiled_tokens_estimate,
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
                compiled: result.compiled_tokens_estimate,
                reused: mem.reused_tokens,
                provider: route.provider.clone(),
                model: model.clone(),
                cache_hit: true,
                cache_saved_tokens: saved_tokens,
                intent: result.intent.clone(),
                sensitive,
                upstream: upstream.clone(),
                scope: scope.clone(),
            });

            if stream {
                return stream_cached_response(&fp.to_string(), &cached.model, &cached.content);
            }

            return Json(json!({
                "id": format!("katara-{fp}"),
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
                "katara": {
                    "provider": route.provider,
                    "model": model,
                    "intent": result.intent,
                    "raw_tokens": result.raw_tokens_estimate,
                    "compiled_tokens": result.compiled_tokens_estimate,
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
                {
                    let mut collector = state.collector.lock().unwrap();
                    collector.record(RecordEntry {
                        raw: result.raw_tokens_estimate,
                        compiled: result.compiled_tokens_estimate,
                        reused: mem.reused_tokens,
                        provider: route.provider.clone(),
                        model: model.clone(),
                        cache_hit: false,
                        cache_saved_tokens: 0,
                        intent: result.intent.clone(),
                        sensitive,
                        upstream: upstream.clone(),
                        scope: scope.clone(),
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
            Err(e) => Json(json!({
                "error": {
                    "message": e,
                    "type": "provider_error",
                    "katara": {
                        "provider": route.provider,
                        "model": model,
                        "intent": result.intent,
                        "compiled_tokens": result.compiled_tokens_estimate,
                        "client_app": upstream.client_app,
                        "upstream_provider": upstream.provider,
                        "upstream_model": upstream.model,
                        "semantic_cache_hit": semantic_cache_hit,
                        "semantic_fingerprint": semantic_fp.to_string()
                    }
                }
            }))
            .into_response(),
        };
    }

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
            {
                let mut collector = state.collector.lock().unwrap();
                collector.chat_cache.insert(
                    cache_key,
                    CachedChatResponse {
                        content: fwd.content.clone(),
                        model: fwd.model.clone(),
                        prompt_tokens: fwd.prompt_tokens,
                        completion_tokens: fwd.completion_tokens,
                    },
                );
                collector.record(RecordEntry {
                    raw: result.raw_tokens_estimate,
                    compiled: result.compiled_tokens_estimate,
                    reused: mem.reused_tokens,
                    provider: route.provider.clone(),
                    model: model.clone(),
                    cache_hit: false,
                    cache_saved_tokens: 0,
                    intent: result.intent.clone(),
                    sensitive,
                    upstream: upstream.clone(),
                    scope: scope.clone(),
                });
            }

            // Return OpenAI-compatible format
            Json(json!({
                "id": format!("katara-{fp}"),
                "object": "chat.completion",
                "model": fwd.model,
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": fwd.content
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": fwd.prompt_tokens,
                    "completion_tokens": fwd.completion_tokens
                },
                "katara": {
                    "provider": route.provider,
                    "model": model,
                    "intent": result.intent,
                    "raw_tokens": result.raw_tokens_estimate,
                    "compiled_tokens": result.compiled_tokens_estimate,
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
        Err(e) => Json(json!({
            "error": {
                "message": e,
                "type": "provider_error",
                "katara": {
                    "provider": route.provider,
                    "model": model,
                    "intent": result.intent,
                    "compiled_tokens": result.compiled_tokens_estimate,
                    "semantic_cache_hit": semantic_cache_hit,
                    "semantic_fingerprint": semantic_fp.to_string()
                }
            }
        }))
        .into_response(),
    }
}

fn stream_cached_response(request_id: &str, model: &str, content: &str) -> Response<Body> {
    let chunk = json!({
        "id": format!("katara-{request_id}"),
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
    fn compile_with_semantic_cache_reuses_compiled_result() {
        let mut collector = MetricsCollector::new();

        let (first_fp, first_result, first_hit) =
            compile_with_semantic_cache(&mut collector, "panic: duplicated stack trace");
        let (second_fp, second_result, second_hit) =
            compile_with_semantic_cache(&mut collector, "panic: duplicated stack trace");

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
                cache_hit: false,
                sensitive: false,
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
                cache_hit: false,
                sensitive: false,
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
                cache_hit: false,
                sensitive: false,
                ts: 300,
            },
        ];

        prune_request_history(&mut history, Some(180), 1);

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].ts, 300);
    }
}

async fn metrics_snapshot(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let collector = state.collector.lock().unwrap();
    Json(serde_json::to_value(collector.snapshot()).unwrap_or_default())
}

async fn metrics_stream(
    State(state): State<SharedState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(std::time::Duration::from_secs(2));
    let stream = tokio_stream::StreamExt::map(IntervalStream::new(interval), move |_| {
        let collector = state.collector.lock().unwrap();
        let data = serde_json::to_string(collector.snapshot()).unwrap_or_default();
        Ok(Event::default().event("metrics").data(data))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// ── Main ──────────────────────────────────────────────

#[tokio::main]
async fn main() {
    println!("KATARA v{} — Sovereign AI Context OS", runtime_version());
    println!("────────────────────────────────────────");

    let router_config = load_config();
    let workspace_context = load_workspace_context();
    if workspace_context.tenant_id.is_some() || workspace_context.project_id.is_some() {
        println!(
            "  Workspace scope: tenant={:?}, project={:?}, policy_pack={:?}",
            workspace_context.tenant_id,
            workspace_context.project_id,
            workspace_context.policy_pack
        );
    }

    let state: SharedState = Arc::new(AppState {
        collector: Mutex::new(MetricsCollector::new()),
        router_config,
        workspace_context,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/version", get(version))
        .route("/v1/providers", get(list_providers))
        .route(
            "/v1/runtime/client-context",
            get(get_runtime_client_context).post(set_runtime_client_context),
        )
        .route("/v1/compile", post(compile))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/metrics", get(metrics_snapshot))
        .route("/v1/metrics/stream", get(metrics_stream))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("────────────────────────────────────────");
    println!("Listening on {addr}");
    println!("  POST /v1/compile            — compile context only");
    println!("  POST /v1/chat/completions   — compile + forward to LLM");
    println!("  GET  /v1/providers          — list configured providers + runtime details");
    println!("  GET  /v1/runtime/client-context — read live upstream client context");
    println!("  POST /v1/runtime/client-context — update live upstream client context");
    println!("  GET  /v1/metrics            — JSON snapshot");
    println!("  GET  /v1/metrics/stream     — SSE live stream");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
