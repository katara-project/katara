use axum::{routing::get, routing::post, Json, Router};
use serde_json::json;
use std::net::SocketAddr;

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "katara-core",
        "version": "7.0.0"
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "version": "7.0.0",
        "product": "KATARA"
    }))
}

async fn compile(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let raw = payload
        .get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let result = compiler::compile_context(raw);
    let mem = memory::summarize_memory(result.raw_tokens_estimate);
    let sensitive = payload
        .get("sensitive")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let route = router::choose_provider(&result.intent, sensitive);
    let efficiency = metrics::compute(
        result.raw_tokens_estimate,
        result.compiled_tokens_estimate,
        mem.reused_tokens,
    );
    Json(json!({
        "intent": result.intent,
        "raw_tokens": result.raw_tokens_estimate,
        "compiled_tokens": result.compiled_tokens_estimate,
        "memory_reused_tokens": mem.reused_tokens,
        "context_reuse_ratio": mem.context_reuse_ratio,
        "provider": route.provider,
        "routing_reason": route.reason,
        "token_avoidance_ratio": efficiency.token_avoidance_ratio
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/healthz", get(health))
        .route("/version", get(version))
        .route("/v1/compile", post(compile));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("KATARA core listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
