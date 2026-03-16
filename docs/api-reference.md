# DISTIRA REST API Reference

> **Version 10.13** — The AI Context Compiler
>
> Base URL: `http://localhost:8080`

---

## Table of Contents

1. [Authentication](#authentication)
2. [Endpoints Overview](#endpoints-overview)
3. [Health & Version](#health--version)
   - [GET /healthz](#get-healthz)
   - [GET /version](#get-version)
4. [Context Compilation](#context-compilation)
   - [POST /v1/compile](#post-v1compile)
5. [Chat Completions](#chat-completions)
   - [POST /v1/chat/completions](#post-v1chatcompletions)
6. [Providers](#providers)
   - [GET /v1/providers](#get-v1providers)
7. [Runtime Client Context](#runtime-client-context)
   - [GET /v1/runtime/client-context](#get-v1runtimeclient-context)
   - [POST /v1/runtime/client-context](#post-v1runtimeclient-context)
8. [Metrics](#metrics)
   - [GET /v1/metrics](#get-v1metrics)
   - [GET /v1/metrics/stream](#get-v1metricsstream)
   - [DELETE /v1/metrics/reset](#delete-v1metricsreset)
- [GET /v1/metrics/export](#get-v1metricsexport)
9. [Suggestions](#suggestions)
   - [GET /v1/suggestions](#get-v1suggestions)
10. [Slash Commands](#slash-commands)
11. [Intent Detection & Routing](#intent-detection--routing)
12. [Understanding Metrics](#understanding-metrics)
13. [Recipes & Examples](#recipes--examples)

---

## Authentication

Authentication is **optional** and controlled by the `DISTIRA_API_KEY` environment variable.

| Mode | Configuration | Behavior |
|---|---|---|
| **Open** (default) | `DISTIRA_API_KEY` not set or empty | All requests are accepted |
| **Bearer token** | `DISTIRA_API_KEY=your-secret-key` | All `/v1/*` routes require `Authorization: Bearer your-secret-key` |

> `/healthz` and `/version` are **always public**, even when auth is enabled.

### Example with auth

```bash
curl -H "Authorization: Bearer your-secret-key" \
     http://localhost:8080/v1/metrics
```

---

## Endpoints Overview

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/healthz` | No | Health check |
| `GET` | `/version` | No | Version info |
| `POST` | `/v1/compile` | If enabled | Compile context (no LLM call) |
| `POST` | `/v1/chat/completions` | If enabled | Compile + forward to LLM |
| `GET` | `/v1/providers` | If enabled | List configured providers |
| `GET` | `/v1/runtime/client-context` | If enabled | Read live upstream client context |
| `POST` | `/v1/runtime/client-context` | If enabled | Update live upstream client context |
| `GET` | `/v1/metrics` | If enabled | Metrics JSON snapshot |
| `GET` | `/v1/metrics/stream` | If enabled | SSE live metrics stream |
| `DELETE` | `/v1/metrics/reset` | If enabled | Reset all counters |
| `GET` | `/v1/suggestions` | If enabled | Optimization suggestions |

---

## Health & Version

### GET /healthz

Returns the service health status. Always public.

**Request:**

```bash
curl http://localhost:8080/healthz
```

**Response:**

```json
{
  "status": "ok",
  "service": "distira-core",
  "version": "10.13.0"
}
```

---

### GET /version

Returns the product version.

**Request:**

```bash
curl http://localhost:8080/version
```

**Response:**

```json
{
  "version": "10.13.0",
  "product": "DISTIRA"
}
```

---

## Context Compilation

### POST /v1/compile

The **core endpoint**. Sends raw context through the full DISTIRA pipeline (fingerprint → cache → compiler → memory → router) **without** calling any LLM. Returns the optimized context, intent detection, routing decision, and efficiency metrics.

Use this endpoint to:
- See how DISTIRA compresses your prompts
- Test intent detection
- Evaluate token savings before sending to an LLM
- Integrate DISTIRA as a middleware in your own AI pipeline

#### Request Body

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `context` | string | Yes | — | The raw prompt/context to compile |
| `sensitive` | boolean | No | `false` | Force local-only routing (sovereign mode) |
| `client_app` | string | No | — | Client application label (e.g. `"VS Code Copilot Chat"`) |
| `upstream_provider` | string | No | — | Upstream provider label (e.g. `"GitHub Copilot"`) |
| `upstream_model` | string | No | — | Upstream model label (e.g. `"Claude Opus 4.6"`) |
| `tenant_id` | string | No | `"default-tenant"` | Multi-tenant isolation key |
| `project_id` | string | No | `"distira-platform"` | Project identifier |

#### Example Request

```bash
curl -X POST http://localhost:8080/v1/compile \
  -H "Content-Type: application/json" \
  -d '{
    "context": "Debug this auth function: error at line 42 in auth.rs, the unwrap panics on None",
    "sensitive": false
  }'
```

#### Example Response

```json
{
  "fingerprint": "6916858722145209385",
  "cache_hit": false,
  "intent": "debug",
  "intent_confidence": 0.57,
  "raw_tokens": 38,
  "compiled_tokens": 27,
  "optimizer_savings": 3,
  "compiled_context": "[k:debug]|auth function: error line 42 auth.rs unwrap panics None",
  "summary": "Intent: debug. Reduced estimated context from 38 to 27 tokens.",
  "slash_command": null,
  "force_local": false,
  "efficiency_directive": "Be a precise debugging assistant. Return ONLY: 1) root cause (1 sentence), 2) fix (code patch or command). No explanations, no background, no alternatives unless asked. If the fix is a code change, show only the minimal diff.",
  "RCT2I_applied": false,
  "RCT2I_sections": 0,
  "memory_reused_tokens": 12,
  "context_reuse_ratio": 0.31,
  "provider": "ollama-mistral-7b-instruct",
  "model": "mistral:7b-instruct",
  "client_app": null,
  "upstream_provider": null,
  "upstream_model": null,
  "tenant_id": "default-tenant",
  "project_id": "distira-platform",
  "policy_pack": "baseline",
  "routing_reason": "Intent [debug] → routed to ollama-mistral-7b-instruct.",
  "token_avoidance_ratio": 0.59,
  "cost_usd": 0.0
}
```

#### Response Fields

| Field | Type | Description |
|---|---|---|
| `fingerprint` | string | Unique hash for semantic caching (same prompt = same fingerprint) |
| `cache_hit` | boolean | `true` if result was served from semantic cache |
| `intent` | string | Detected intent: `debug`, `review`, `codegen`, `summarize`, `translate`, `ocr`, `general` |
| `intent_confidence` | float | Confidence score ∈ [0.0, 1.0] |
| `raw_tokens` | integer | Original token count before compilation |
| `compiled_tokens` | integer | Token count after DISTIRA optimization |
| `optimizer_savings` | integer | Tokens saved by the BPE optimizer (lossless passes) |
| `compiled_context` | string | The optimized context text (prefixed with intent marker) |
| `summary` | string | Human-readable summary of the compilation |
| `slash_command` | string \| null | Detected slash command (e.g. `"/debug"`) or null |
| `force_local` | boolean | `true` if routing is forced to local providers |
| `efficiency_directive` | string | Auto-injected LLM instruction selected transparently by DISTIRA from intent + prompt signals (reduces output tokens) |
| `RCT2I_applied` | boolean | `true` if the prompt was restructured using RCT2I (Role/Context/Task/Intent/Audience) sections |
| `RCT2I_sections` | integer | Number of RCT2I sections found in the restructured prompt (0–5) |
| `memory_reused_tokens` | integer | Tokens reused from session memory (prior stable context blocks) |
| `context_reuse_ratio` | float | Ratio of reused tokens to raw tokens ∈ [0.0, 1.0] |
| `provider` | string | Provider key the request would be routed to |
| `model` | string | Model name at the routed provider |
| `client_app` | string \| null | Client app identity |
| `upstream_provider` | string \| null | Upstream provider label |
| `upstream_model` | string \| null | Upstream model label |
| `tenant_id` | string | Resolved tenant ID |
| `project_id` | string | Resolved project ID |
| `policy_pack` | string | Active policy pack |
| `routing_reason` | string | Human-readable explanation of the routing decision |
| `token_avoidance_ratio` | float | Overall efficiency ratio ∈ [0.0, 1.0] — higher = more savings |
| `cost_usd` | float | Estimated cost in USD for this compilation (0 for on-prem) |

---

## Chat Completions

### POST /v1/chat/completions

**OpenAI-compatible** chat endpoint. Compiles context through the full pipeline, then forwards the optimized request to the routed LLM provider. Supports streaming via SSE.

Use this endpoint as a **drop-in replacement** for OpenAI's `/v1/chat/completions` — any OpenAI-compatible client works out of the box.

#### Request Body

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `messages` | array | Yes | — | OpenAI-format message array (see below) |
| `model` | string | No | auto-routed | Override the model (otherwise DISTIRA routes automatically) |
| `sensitive` | boolean | No | `false` | Force local-only routing |
| `stream` | boolean | No | `false` | Enable SSE streaming |
| `client_app` | string | No | — | Client application label |
| `upstream_provider` | string | No | — | Upstream provider label |
| `upstream_model` | string | No | — | Upstream model label |
| `tenant_id` | string | No | `"default-tenant"` | Multi-tenant isolation key |
| `project_id` | string | No | — | Project identifier |
| `temperature` | float | No | — | Forwarded to LLM provider |
| `max_tokens` | integer | No | — | Forwarded to LLM provider |
| *(any extra)* | — | No | — | All extra fields are forwarded to the LLM provider |

##### Message Format

```json
{
  "messages": [
    { "role": "system", "content": "You are a helpful assistant." },
    { "role": "user", "content": "Explain the Rust borrow checker." }
  ]
}
```

#### Example: Non-Streaming

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      { "role": "user", "content": "Write a Python function to sort a list" }
    ]
  }'
```

**Response** (OpenAI-compatible format with DISTIRA metadata):

```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "def sort_list(lst):\n    return sorted(lst)"
      },
      "finish_reason": "stop"
    }
  ],
  "distira": {
    "intent": "codegen",
    "provider": "ollama-qwen2.5-coder",
    "model": "qwen2.5-coder:7b",
    "raw_tokens": 18,
    "compiled_tokens": 12,
    "cache_hit": false,
    "token_avoidance_ratio": 0.55
  }
}
```

#### Example: Streaming

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      { "role": "user", "content": "Explain async/await in Rust" }
    ],
    "stream": true
  }'
```

**Response** (Server-Sent Events):

```md
data: {"id":"chatcmpl-abc","object":"chat.completion.chunk","choices":[{"delta":{"content":"Async"},"index":0}]}

data: {"id":"chatcmpl-abc","object":"chat.completion.chunk","choices":[{"delta":{"content":"/await"},"index":0}]}

data: [DONE]
```

#### Example: Sensitive Mode (Force Local)

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      { "role": "user", "content": "Analyze this patient medical record" }
    ],
    "sensitive": true
  }'
```

When `sensitive: true`, DISTIRA **always** routes to `ollama-llama3` (on-prem) regardless of intent.

---

## Providers

### GET /v1/providers

Lists all configured LLM providers with runtime statistics.

**Request:**

```bash
curl http://localhost:8080/v1/providers
```

**Response:**

```json
{
  "providers": [
    "ollama-llama3",
    "ollama-qwen2.5-coder",
    "ollama-mistral-7b-instruct",
    "ollama-deepseek-ocr",
    "mistral-ocr-2512-cloud",
    "openrouter-step-3.5-flash-cloud",
    "openrouter-mistral-small-3.1-24b-instruct-cloud"
  ],
  "provider_details": [
    {
      "key": "ollama-llama3",
      "model": "llama3:latest",
      "base_url": "http://localhost:11434",
      "deployment": "on-prem",
      "intents": ["general", "summarize", "sensitive"],
      "avg_latency_ms": 1250.5,
      "error_rate": 0.0
    }
  ]
}
```

#### Provider Details Fields

| Field | Type | Description |
|---|---|---|
| `key` | string | Unique provider identifier |
| `model` | string | Model name at this provider |
| `base_url` | string | Provider API base URL |
| `deployment` | string | `"on-prem"` or `"cloud"` |
| `intents` | array | List of intents this provider serves |
| `avg_latency_ms` | float | Rolling average latency in milliseconds |
| `error_rate` | float | Rolling error rate ∈ [0.0, 1.0] |

---

## Runtime Client Context

### GET /v1/runtime/client-context

Read the current live upstream client context (client app, provider, model).

```bash
curl http://localhost:8080/v1/runtime/client-context
```

### POST /v1/runtime/client-context

Update the live upstream client context. Useful when the user switches models or providers in their IDE.

**Request Body:**

| Field | Type | Required | Description |
|---|---|---|---|
| `client_app` | string | No | Client application label |
| `upstream_provider` | string | No | Upstream provider name |
| `upstream_model` | string | No | Upstream model name |
| `tenant_id` | string | No | Tenant identifier |
| `project_id` | string | No | Project identifier |

```bash
curl -X POST http://localhost:8080/v1/runtime/client-context \
  -H "Content-Type: application/json" \
  -d '{
    "client_app": "VS Code Copilot Chat",
    "upstream_provider": "Anthropic",
    "upstream_model": "Claude Opus 4.6"
  }'
```

---

## Metrics

### GET /v1/metrics

Returns a full snapshot of all DISTIRA metrics since the last reset (or server start).

**Request:**

```bash
curl http://localhost:8080/v1/metrics
```

**Response:**

```json
{
  "ts": 1742054400,
  "total_requests": 150,
  "raw_tokens": 45000,
  "compiled_tokens": 28500,
  "memory_reused_tokens": 8200,
  "efficiency_score": 0.72,
  "local_ratio": 85.3,
  "cache_hits": 32,
  "cache_misses": 118,
  "cache_saved_tokens": 4800,
  "routes_local": 128,
  "routes_cloud": 17,
  "routes_midtier": 5,
  "session_cost_usd": 0.0023,
  "last_request_cost_usd": 0.00001,
  "stable_blocks": 12,
  "context_reuse_ratio_pct": 18.2,
  "session_budget_usd": 1.0,
  "RCT2I_applied_count": 42,
  "intent_stats": {
    "debug": { "requests": 37, "raw_tokens": 12000, "compiled_tokens": 7200 },
    "codegen": { "requests": 45, "raw_tokens": 15000, "compiled_tokens": 9000 },
    "general": { "requests": 50, "raw_tokens": 12000, "compiled_tokens": 8400 },
    "review": { "requests": 10, "raw_tokens": 4000, "compiled_tokens": 2400 },
    "summarize": { "requests": 8, "raw_tokens": 2000, "compiled_tokens": 1500 }
  },
  "model_stats": { "...": "..." },
  "upstream_stats": { "...": "..." },
  "last_request": { "...": "..." },
  "request_history": [],
  "history_raw": [1200, 1400, 1100],
  "history_compiled": [800, 900, 700],
  "history_reused": [200, 300, 250],
  "history_hour_epochs": [],
  "history_hour_raw": [],
  "history_hour_compiled": [],
  "history_hour_reused": [],
  "context_blocks_summary": [],
  "alerts": []
}
```

#### Key Metrics Explained

| Field | Type | Description |
|---|---|---|
| `ts` | integer | Unix timestamp of the snapshot |
| `total_requests` | integer | Total compile/chat requests since last reset |
| `raw_tokens` | integer | Cumulative tokens **before** DISTIRA optimization |
| `compiled_tokens` | integer | Cumulative tokens **after** optimization |
| `memory_reused_tokens` | integer | Tokens recovered from session memory (prior stable blocks) |
| `efficiency_score` | float | Overall pipeline efficiency ∈ [0.0, 1.0] (higher = better) |
| `local_ratio` | float | Percentage of requests routed to on-prem providers |
| `cache_hits` | integer | Requests served from semantic cache (zero-cost) |
| `cache_misses` | integer | Requests that required full compilation |
| `cache_saved_tokens` | integer | Tokens saved by cache hits |
| `routes_local` | integer | Requests routed to on-prem (sovereign) providers |
| `routes_cloud` | integer | Requests routed to cloud providers |
| `routes_midtier` | integer | Requests routed to mid-tier providers |
| `session_cost_usd` | float | Cumulative estimated cost in USD for this session |
| `last_request_cost_usd` | float | Cost of the most recent request in USD |
| `stable_blocks` | integer | Number of stable context blocks in memory |
| `context_reuse_ratio_pct` | float | Session-level context reuse as a percentage |
| `session_budget_usd` | float | Configured session cost budget (0 = disabled) |
| `RCT2I_applied_count` | integer | Number of requests where RCT2I prompt structuring was applied |
| `intent_stats` | object | Per-intent breakdown (see below) |
| `model_stats` | object | Per-model/provider breakdown |
| `upstream_stats` | object | Per-upstream-client breakdown |
| `last_request` | object \| null | Lineage of the most recent request |
| `request_history` | array | Rolling history of recent requests (audit trail) |
| `alerts` | array | Budget alerts (if session cost exceeds thresholds) |

#### Intent Stats (per intent)

| Field | Type | Description |
|---|---|---|
| `requests` | integer | Total requests for this intent |
| `raw_tokens` | integer | Cumulative raw tokens for this intent |
| `compiled_tokens` | integer | Cumulative compiled tokens for this intent |

**Computing reduction rate per intent:**

```md
reduction_pct = (raw_tokens - compiled_tokens) / raw_tokens × 100
```

#### Model Stats (per model)

| Field | Type | Description |
|---|---|---|
| `model` | string | Model name |
| `provider` | string | Provider key |
| `requests` | integer | Requests routed to this model |
| `raw_tokens` | integer | Raw tokens for requests to this model |
| `compiled_tokens` | integer | Compiled tokens for requests to this model |
| `memory_reused_tokens` | integer | Tokens reused from memory |
| `efficiency_score` | float | Efficiency score for this model |
| `sovereign_requests` | integer | Sovereign (on-prem) requests |
| `non_sovereign_requests` | integer | Cloud requests |
| `sovereign_ratio` | float | On-prem ratio for this model |

#### History Arrays

The `history_raw`, `history_compiled`, and `history_reused` arrays are **rolling 24-point time series** (one point per tick), useful for rendering trend charts in a dashboard.

---

### GET /v1/metrics/stream

Server-Sent Events (SSE) endpoint that pushes metrics snapshots every **2 seconds**. Perfect for live dashboards.

**Request:**

```bash
curl -N http://localhost:8080/v1/metrics/stream
```

**Response** (continuous stream):

```md
event: metrics
data: {"ts":1742054400,"total_requests":150,"raw_tokens":45000,...}

event: metrics
data: {"ts":1742054402,"total_requests":151,"raw_tokens":45200,...}
```

#### JavaScript Example

```javascript
const evtSource = new EventSource("http://localhost:8080/v1/metrics/stream");

evtSource.addEventListener("metrics", (event) => {
  const metrics = JSON.parse(event.data);
  console.log(`Requests: ${metrics.total_requests}`);
  console.log(`Efficiency: ${(metrics.efficiency_score * 100).toFixed(1)}%`);
  console.log(`Tokens saved: ${metrics.raw_tokens - metrics.compiled_tokens}`);
});
```

---

### DELETE /v1/metrics/reset

Resets all counters to zero. Returns `204 No Content`.

```bash
curl -X DELETE http://localhost:8080/v1/metrics/reset
```

---

### GET /v1/metrics/export

*V10.17* — Export cumulative metrics as structured JSON for enterprise reporting. Returns per-provider breakdown, per-intent breakdown, and cumulative totals.

```bash
curl http://localhost:8080/v1/metrics/export | jq .
```

**Response Fields:**

| Field | Type | Description |
|---|---|---|
| `exported_at` | integer | Unix epoch timestamp of export |
| `version` | string | DISTIRA version |
| `cumulative` | object | Total requests, raw/compiled tokens, savings %, cache stats, routing distribution |
| `by_provider` | array | Per-provider: requests, errors, error rate, avg latency |
| `by_intent` | array | Per-intent: requests, raw/compiled tokens, tokens saved, savings % |

---

## Suggestions

### GET /v1/suggestions

Returns proactive optimization suggestions based on routing config, session metrics, provider error rates, and latency measurements.

**Request:**

```bash
curl http://localhost:8080/v1/suggestions
```

**Response:**

```json
{
  "suggestions": [
    {
      "severity": "info",
      "code": "routing_active",
      "provider": "",
      "metric": "task_routing",
      "value": 5,
      "message": "5 intent routes active: codegen→ollama-qwen2.5-coder, debug→ollama-mistral-7b-instruct, ..."
    },
    {
      "severity": "info",
      "code": "session_efficiency",
      "provider": "",
      "metric": "token_savings",
      "value": 36,
      "message": "Session: 150 requests, 16500 tokens saved (36%), 85% on-prem"
    }
  ]
}
```

---

## Slash Commands

Prefix any `context` string with a slash command to **override** automatic intent detection with 100% confidence.

| Command | Intent | Description |
|---|---|---|
| `/debug` | debug | Force debug intent |
| `/code` | codegen | Force codegen intent |
| `/review` | review | Force code review intent |
| `/summarize` | summarize | Force summarize intent |
| `/translate` | translate | Force translate intent |
| `/ocr` | ocr | Force OCR intent |
| `/fast` | fast | Force fast/summarize mode |
| `/quality` | quality | Force high-quality mode |
| `/general` | general | Force general intent |
| `/dtlr` | *(any)* | **Data to Local Routing** — forces on-prem routing (`force_local: true`) |

### Example

```bash
curl -X POST http://localhost:8080/v1/compile \
  -H "Content-Type: application/json" \
  -d '{"context": "/debug panic at line 42 in auth.rs"}'
```

The `/debug` prefix is stripped from the context, intent is set to `debug` with confidence `1.0`, and the slash_command field in the response will be `"/debug"`.

---

## Intent Detection & Routing

DISTIRA automatically detects the intent of each request and routes to the best provider.

### Intent Keywords

| Intent | Trigger Keywords | Default Provider |
|---|---|---|
| **debug** | error, trace, panic, exception, fatal, crash | `ollama-mistral-7b-instruct` (Mistral 7B, local) |
| **review** | diff, pull request, refactor, review | `ollama-qwen2.5-coder` (Qwen 2.5, local) |
| **codegen** | function, implement, write, typescript, javascript, go, kotlin | `ollama-qwen2.5-coder` (Qwen 2.5, local) |
| **summarize** | summarize, explain, recap | `openrouter-mistral-small-3.1-24b-instruct-cloud` |
| **translate** | translate, traduire, french, german, japanese, chinese | `openrouter-mistral-small-3.1-24b-instruct-cloud` |
| **ocr** | ocr, image, scan, extract text | `mistral-ocr-2512-cloud` |
| **general** | *(anything else)* | `openrouter-step-3.5-flash-cloud` |

### Routing Overrides

- **`sensitive: true`** → always routes to `ollama-llama3` (on-prem), regardless of intent
- **`/dtlr` slash command** → forces `force_local: true`, routes to nearest on-prem provider
- **Adaptive routing** → DISTIRA considers provider latency and error rates and may re-route to fallback providers

---

## Understanding Metrics

### Token Reduction

DISTIRA's primary value is **reducing tokens** before they reach your LLM:

```md
Tokens saved     = raw_tokens - compiled_tokens
Reduction %      = tokens_saved / raw_tokens × 100
Token avoidance  = (tokens_saved + memory_reused) / raw_tokens
```

**V10.13 targets:** ≥ 30% reduction across all intents (debug, review, codegen, summarize, general).

### Efficiency Score

The `efficiency_score` combines:
- **Token reduction** — how much DISTIRA compresses
- **Memory reuse** — how much context is recovered from prior stable blocks
- **Cache hits** — how often the semantic cache avoids recompilation

Formula: `efficiency = (tokens_saved + memory_reused) / raw_tokens`

### Semantic Cache

Identical prompts produce the same `fingerprint`. On cache hit:
- Zero compilation cost
- Instant response
- `cache_hit: true` in the response

### Sovereign Ratio

```md
local_ratio = routes_local / total_requests × 100
```

Measures what percentage of your AI traffic stays **on-prem** (sovereign). Higher = more data sovereignty.

### Session Cost

```md
session_cost_usd = Σ cost_usd for all requests
```

On-prem providers (Ollama) cost `$0.00`. Cloud providers are estimated at `$0.006 / 1K tokens`.

---

## Recipes & Examples

### Recipe 1: Measure Token Savings for a Prompt

```bash
# Compile without sending to LLM
RESULT=$(curl -s -X POST http://localhost:8080/v1/compile \
  -H "Content-Type: application/json" \
  -d '{"context": "Your long prompt here..."}')

# Extract savings
echo $RESULT | jq '{
  raw: .raw_tokens,
  compiled: .compiled_tokens,
  saved: (.raw_tokens - .compiled_tokens),
  reduction_pct: ((.raw_tokens - .compiled_tokens) / .raw_tokens * 100),
  provider: .provider,
  intent: .intent
}'
```

### Recipe 2: Batch Test All Intents

```powershell
$intents = @(
    @{ context = "error: panic at thread main"; expected = "debug" },
    @{ context = "implement a sort function in Rust"; expected = "codegen" },
    @{ context = "review this pull request diff"; expected = "review" },
    @{ context = "summarize the meeting notes"; expected = "summarize" },
    @{ context = "translate this to French"; expected = "translate" }
)

foreach ($test in $intents) {
    $body = @{ context = $test.context; sensitive = $false } | ConvertTo-Json
    $r = Invoke-RestMethod -Uri "http://localhost:8080/v1/compile" `
         -Method Post -ContentType "application/json" -Body $body
    $status = if ($r.intent -eq $test.expected) { "PASS" } else { "FAIL" }
    Write-Host "[$status] $($test.expected): intent=$($r.intent), " `
               "reduction=$([math]::Round((1 - $r.compiled_tokens/$r.raw_tokens) * 100))%"
}
```

### Recipe 3: Monitor Metrics in Real-Time (Python)

```python
import json
import sseclient  # pip install sseclient-py
import requests

response = requests.get("http://localhost:8080/v1/metrics/stream", stream=True)
client = sseclient.SSEClient(response)

for event in client.events():
    if event.event == "metrics":
        m = json.loads(event.data)
        saved = m["raw_tokens"] - m["compiled_tokens"]
        pct = saved / m["raw_tokens"] * 100 if m["raw_tokens"] > 0 else 0
        print(f"Requests: {m['total_requests']} | "
              f"Saved: {saved} tokens ({pct:.1f}%) | "
              f"Cache hits: {m['cache_hits']} | "
              f"Local: {m['local_ratio']:.0f}%")
```

### Recipe 4: Use as OpenAI Drop-In (Python)

```python
from openai import OpenAI

# Point to DISTIRA instead of OpenAI
client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="your-distira-api-key"  # or "unused" if auth is disabled
)

response = client.chat.completions.create(
    model="auto",  # DISTIRA routes automatically
    messages=[
        {"role": "user", "content": "Write a Fibonacci function in Go"}
    ]
)

print(response.choices[0].message.content)
```

### Recipe 5: Sensitive Data — Force Local Routing

```bash
# PII/medical/financial data → always on-prem
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      { "role": "user", "content": "Summarize this patient chart: ..." }
    ],
    "sensitive": true
  }'
```

### Recipe 6: Per-Intent Reduction Stats

```bash
curl -s http://localhost:8080/v1/metrics | jq '.intent_stats | to_entries[] | {
  intent: .key,
  requests: .value.requests,
  raw: .value.raw_tokens,
  compiled: .value.compiled_tokens,
  reduction_pct: (if .value.raw_tokens > 0
    then ((.value.raw_tokens - .value.compiled_tokens) / .value.raw_tokens * 100 | round)
    else 0 end)
}'
```

### Recipe 7: Check Provider Health

```bash
curl -s http://localhost:8080/v1/providers | jq '.provider_details[] | {
  provider: .key,
  model: .model,
  deployment: .deployment,
  latency_ms: .avg_latency_ms,
  error_rate: .error_rate
}'
```

---

## Error Handling

| Status | Meaning |
|---|---|
| `200` | Success |
| `204` | Success, no content (metrics reset) |
| `401` | Unauthorized — invalid or missing Bearer token |
| `422` | Unprocessable — invalid request body |
| `502` | Bad Gateway — upstream LLM provider is unreachable |

All error responses follow the format:

```json
{
  "error": "description of the error"
}
```

---

## CORS

CORS is enabled by default for all origins, supporting `GET`, `POST`, `DELETE` methods and `Content-Type`, `Authorization` headers.

---

*DISTIRA — The AI Context Compiler*
*License: AGPL-3.0 — Copyright 2024-2026 Christophe Freijanes*
