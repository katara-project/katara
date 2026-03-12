use serde::{Deserialize, Serialize};
use serde_json::json;

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
    compiled_context: &str,
    api_key: Option<&str>,
) -> Result<ForwardResponse, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": compiled_context
            }
        ],
        "max_tokens": 1024
    });

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

    // Parse OpenAI-compatible response
    let content = json["choices"]
        .get(0)
        .and_then(|c| c["message"]["content"].as_str())
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

/// List known adapter styles.
pub fn supported_adapters() -> Vec<&'static str> {
    ADAPTER_STYLES.to_vec()
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
}
