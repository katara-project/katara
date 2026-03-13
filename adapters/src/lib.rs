use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Known adapter styles.
pub const ADAPTER_STYLES: &[&str] = &["openai-compatible", "google"];

/// Response from a forwarded LLM call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardResponse {
    pub provider: String,
    pub model: String,
    pub content: String,
    pub prompt_tokens: Option<usize>,
    pub completion_tokens: Option<usize>,
}

/// Forward compiled context to an LLM provider.
///
/// Supports any OpenAI-compatible endpoint (Ollama, OpenAI, Mistral, etc.).
/// The `base_url` should end in `/v1`.
pub async fn forward(
    base_url: &str,
    model: &str,
    messages: &[Value],
    api_key: Option<&str>,
    extra_body: &Map<String, Value>,
) -> Result<ForwardResponse, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = build_body(model, messages, extra_body, false);

    let client = reqwest::Client::new();
    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body);

    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {key}"));
    }

    let resp = req.send().await.map_err(|e| format!("HTTP error: {e}"))?;
    let status = resp.status();

    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Provider returned {status}: {text}"));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid JSON from provider: {e}"))?;

    // Parse OpenAI-compatible response.
    // Some reasoning models (Step, DeepSeek-R1 via OpenRouter) may return
    // the final answer in `message.content` and thinking tokens in
    // `message.reasoning`. When `content` is null/empty we fall back to
    // `message.reasoning` so the caller always gets something useful.
    let msg = json["choices"]
        .get(0)
        .and_then(|c| c["message"].as_object());
    let content = msg
        .and_then(|m| m.get("content").and_then(|v| v.as_str()))
        .filter(|s| !s.is_empty())
        .or_else(|| {
            msg.and_then(|m| m.get("reasoning").and_then(|v| v.as_str()))
                .filter(|s| !s.is_empty())
        })
        .unwrap_or("")
        .to_string();

    let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().map(|v| v as usize);
    let completion_tokens = json["usage"]["completion_tokens"]
        .as_u64()
        .map(|v| v as usize);

    Ok(ForwardResponse {
        provider: base_url.to_string(),
        model: model.to_string(),
        content,
        prompt_tokens,
        completion_tokens,
    })
}

/// Forward compiled context to an LLM provider with streaming enabled.
///
/// Returns the upstream HTTP response so the caller can proxy the SSE body.
pub async fn forward_stream(
    base_url: &str,
    model: &str,
    messages: &[Value],
    api_key: Option<&str>,
    extra_body: &Map<String, Value>,
) -> Result<reqwest::Response, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = build_body(model, messages, extra_body, true);

    let client = reqwest::Client::new();
    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .json(&body);

    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {key}"));
    }

    let resp = req.send().await.map_err(|e| format!("HTTP error: {e}"))?;
    let status = resp.status();

    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Provider returned {status}: {text}"));
    }

    Ok(resp)
}

/// List known adapter styles.
pub fn supported_adapters() -> Vec<&'static str> {
    ADAPTER_STYLES.to_vec()
}

fn build_body(
    model: &str,
    messages: &[Value],
    extra_body: &Map<String, Value>,
    stream: bool,
) -> Value {
    let mut body = extra_body.clone();
    body.insert("model".into(), Value::String(model.to_string()));
    body.insert("messages".into(), Value::Array(messages.to_vec()));
    body.insert("stream".into(), Value::Bool(stream));
    if !body.contains_key("max_tokens") {
        // 4096 accommodates reasoning models (Step, DeepSeek-R1, etc.) that
        // consume part of the budget for chain-of-thought before the reply.
        body.insert("max_tokens".into(), json!(4096));
    }
    Value::Object(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_adapters() {
        let adapters = supported_adapters();
        assert!(adapters.contains(&"openai-compatible"));
        assert!(adapters.contains(&"google"));
    }

    #[test]
    fn forward_response_serializes() {
        let resp = ForwardResponse {
            provider: "http://localhost:11434/v1".into(),
            model: "llama3.1".into(),
            content: "Hello!".into(),
            prompt_tokens: Some(10),
            completion_tokens: Some(5),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("llama3.1"));
        assert!(json.contains("Hello!"));
    }

    #[test]
    fn build_body_preserves_messages_and_options() {
        let mut extra = Map::new();
        extra.insert("temperature".into(), json!(0.2));

        let body = build_body(
            "llama3:latest",
            &[json!({ "role": "system", "content": "Be brief" })],
            &extra,
            true,
        );

        assert_eq!(body["model"], json!("llama3:latest"));
        assert_eq!(body["messages"][0]["role"], json!("system"));
        assert_eq!(body["temperature"], json!(0.2));
        assert_eq!(body["stream"], json!(true));
    }
}
