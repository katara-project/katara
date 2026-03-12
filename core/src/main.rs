use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};

/// ── Shared application state ──────────────────────────
#[derive(Debug, Clone, Serialize)]
struct IntentStats {
    requests: u64,
    raw_tokens: usize,
    compiled_tokens: usize,
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
    history_raw: Vec<usize>,
    history_compiled: Vec<usize>,
    history_reused: Vec<usize>,
    routes_local: u64,
    routes_cloud: u64,
    routes_midtier: u64,
    intent_stats: std::collections::HashMap<String, IntentStats>,
}

#[derive(Debug)]
struct MetricsCollector {
    snapshot: MetricsSnapshot,
    sem_cache: cache::SemanticCache,
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
                history_raw: Vec::with_capacity(24),
                history_compiled: Vec::with_capacity(24),
                history_reused: Vec::with_capacity(24),
                routes_local: 0,
                routes_cloud: 0,
                routes_midtier: 0,
                intent_stats: std::collections::HashMap::new(),
            },
            sem_cache: cache::SemanticCache::new(),
        }
    }

    fn record(
        &mut self,
        raw: usize,
        compiled: usize,
        reused: usize,
        provider: &str,
        cache_hit: bool,
        intent: &str,
    ) {
        let s = &mut self.snapshot;
        s.total_requests += 1;
        s.raw_tokens += raw;
        s.compiled_tokens += compiled;
        s.memory_reused_tokens += reused;

        if cache_hit {
            s.cache_hits += 1;
        } else {
            s.cache_misses += 1;
        }

        // Classify deployment type from provider name
        if provider.contains("local") || provider.contains("ollama") {
            s.routes_local += 1;
        } else if provider.contains("mistral") {
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

        s.ts = now_epoch();
    }

    fn snapshot(&self) -> &MetricsSnapshot {
        &self.snapshot
    }
}

/// Combined shared state
struct AppState {
    collector: Mutex<MetricsCollector>,
    router_config: router::RouterConfig,
}

type SharedState = Arc<AppState>;

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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
    Json(json!({ "status": "ok", "service": "katara-core", "version": "7.0.0" }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({ "version": "7.0.0", "product": "KATARA" }))
}

async fn list_providers(State(state): State<SharedState>) -> Json<serde_json::Value> {
    Json(json!({ "providers": state.router_config.list_providers() }))
}

#[derive(Deserialize)]
struct CompileRequest {
    context: Option<String>,
    sensitive: Option<bool>,
}

async fn compile(
    State(state): State<SharedState>,
    Json(payload): Json<CompileRequest>,
) -> Json<serde_json::Value> {
    let raw = payload.context.as_deref().unwrap_or("");
    let sensitive = payload.sensitive.unwrap_or(false);

    let fp = fingerprint::fingerprint(raw);

    let mut collector = state.collector.lock().unwrap();
    let cache_hit = collector.sem_cache.get(fp).is_some();

    let result = compiler::compile_context(raw);
    let mem = memory::summarize_memory(result.raw_tokens_estimate);
    let route = state
        .router_config
        .choose_provider(&result.intent, sensitive);

    let efficiency = metrics::compute(
        result.raw_tokens_estimate,
        result.compiled_tokens_estimate,
        mem.reused_tokens,
    );

    if !cache_hit {
        collector.sem_cache.insert(fp, result.summary.clone());
    }
    collector.record(
        result.raw_tokens_estimate,
        result.compiled_tokens_estimate,
        mem.reused_tokens,
        &route.provider,
        cache_hit,
        &result.intent,
    );
    drop(collector);

    Json(json!({
        "fingerprint": fp.to_string(),
        "cache_hit": cache_hit,
        "intent": result.intent,
        "raw_tokens": result.raw_tokens_estimate,
        "compiled_tokens": result.compiled_tokens_estimate,
        "memory_reused_tokens": mem.reused_tokens,
        "context_reuse_ratio": mem.context_reuse_ratio,
        "provider": route.provider,
        "model": route.model,
        "routing_reason": route.reason,
        "token_avoidance_ratio": efficiency.token_avoidance_ratio
    }))
}

/// OpenAI-compatible chat endpoint.
/// Compiles context, routes, then forwards to the chosen LLM.
#[derive(Deserialize)]
struct ChatRequest {
    messages: Option<Vec<serde_json::Value>>,
    model: Option<String>,
    sensitive: Option<bool>,
}

async fn chat_completions(
    State(state): State<SharedState>,
    Json(payload): Json<ChatRequest>,
) -> Json<serde_json::Value> {
    // Extract last user message as the raw context
    let raw: String = payload
        .messages
        .as_ref()
        .and_then(|msgs| msgs.iter().rev().find(|m| m["role"] == "user"))
        .and_then(|m| m["content"].as_str())
        .unwrap_or("")
        .to_string();
    let sensitive = payload.sensitive.unwrap_or(false);

    // 1. Full KATARA pipeline
    let fp = fingerprint::fingerprint(&raw);
    let result = compiler::compile_context(&raw);
    let mem = memory::summarize_memory(result.raw_tokens_estimate);
    let route = state
        .router_config
        .choose_provider(&result.intent, sensitive);

    let efficiency = metrics::compute(
        result.raw_tokens_estimate,
        result.compiled_tokens_estimate,
        mem.reused_tokens,
    );

    // 2. Cache check — lock, update, drop before any .await
    let cache_hit;
    {
        let mut collector = state.collector.lock().unwrap();
        cache_hit = collector.sem_cache.get(fp).is_some();
        if !cache_hit {
            collector.sem_cache.insert(fp, result.summary.clone());
        }
        collector.record(
            result.raw_tokens_estimate,
            result.compiled_tokens_estimate,
            mem.reused_tokens,
            &route.provider,
            cache_hit,
            &result.intent,
        );
    } // MutexGuard dropped here, before .await

    // 3. Resolve API key from env
    let api_key = route
        .api_key_env
        .as_deref()
        .and_then(|env_var| std::env::var(env_var).ok());

    // 4. Forward to LLM provider
    let model = payload.model.clone().unwrap_or_else(|| route.model.clone());

    match adapters::forward(&route.base_url, &model, &raw, api_key.as_deref()).await {
        Ok(fwd) => {
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
                    "intent": result.intent,
                    "raw_tokens": result.raw_tokens_estimate,
                    "compiled_tokens": result.compiled_tokens_estimate,
                    "cache_hit": cache_hit,
                    "token_avoidance_ratio": efficiency.token_avoidance_ratio
                }
            }))
        }
        Err(e) => Json(json!({
            "error": {
                "message": e,
                "type": "provider_error",
                "katara": {
                    "provider": route.provider,
                    "model": model,
                    "intent": result.intent,
                    "compiled_tokens": result.compiled_tokens_estimate
                }
            }
        })),
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
    let stream = IntervalStream::new(interval).map(move |_| {
        let collector = state.collector.lock().unwrap();
        let data = serde_json::to_string(collector.snapshot()).unwrap_or_default();
        Ok(Event::default().event("metrics").data(data))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// ── Main ──────────────────────────────────────────────

#[tokio::main]
async fn main() {
    println!("KATARA v7.0.0 — Sovereign AI Context OS");
    println!("────────────────────────────────────────");

    let router_config = load_config();

    let state: SharedState = Arc::new(AppState {
        collector: Mutex::new(MetricsCollector::new()),
        router_config,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/version", get(version))
        .route("/v1/providers", get(list_providers))
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
    println!("  GET  /v1/providers          — list configured providers");
    println!("  GET  /v1/metrics            — JSON snapshot");
    println!("  GET  /v1/metrics/stream     — SSE live stream");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
