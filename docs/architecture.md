# Architecture

Distira is **The AI Context Compiler**.
It sits between clients and LLM providers, compiling the smallest useful context before every LLM call.

## Request lifecycle

```text
┌────────────────────────┐
│ 1. Request ingestion   │  Accept OpenAI-compatible API call
├────────────────────────┤
│ 2. Intent detection    │  Classify: debug, summarize, review, general
├────────────────────────┤
│ 3. Context Budget      │  Reduce raw tokens: trim logs, compact diffs
│    Compiler            │
├────────────────────────┤
│ 4. Context Memory      │  Reuse stable blocks, send only deltas
│    Lensing             │
├────────────────────────┤
│ 5. Semantic cache      │  Return cached response if fingerprint matches
├────────────────────────┤
│ 6. Hybrid routing      │  Select provider by policy, intent, sensitivity
├────────────────────────┤
│ 7. Lineage resolution  │  Attach upstream client/model/provider metadata
├────────────────────────┤
│ 8. Provider adapter    │  Forward compiled context to chosen LLM
│                        │  JSON or SSE stream proxy depending on request
├────────────────────────┤
│ 9. Telemetry           │  Emit efficiency score, lineage, and route metrics
└────────────────────────┘
```

## Live data flow

```text
┌─────────────────────┐         POST /v1/compile            ┌──────────────────────────┐
│   Client (curl,     │ ──────────────────────────────────► │   DISTIRA Rust Backend    │
│   VS Code ext,      │                                     │                          │
│   any AI tool)      │ ◄────── JSON response ───────────── │  compile() → fingerprint │
└─────────────────────┘                                     │   → cache → compiler     │
                                                            │   → memory → router      │
┌─────────────────────┐                                     │                          │
│   Vue Dashboard     │ ◄── SSE /v1/metrics/stream ──────── │  MetricsCollector (Arc)  │
│   (Pinia store)     │     text/event-stream               │   - cumulative totals    │
│                     │     { raw, compiled, reused, ... }  │   - 24-point history     │
│   EventSource API   │     every 2 seconds                 │   - per-route counters   │
│                     │                                     │   - upstream lineage     │
│                     │                                     │   - last request trace   │
└─────────────────────┘                                     └──────────────────────────┘
```

**How it works:**

1. Any client sends a `POST /v1/compile` or `POST /v1/chat/completions` with raw context.
2. The backend resolves upstream lineage from request fields such as `client_app`, `upstream_provider`, and `upstream_model`.
3. The pipeline runs: fingerprint → semantic cache → compiler → memory lensing → hybrid router → provider adapter.
    The compiler now emits a concrete `compiled_context`, and the chat runtime rewrites the latest user message with that reduced payload while preserving prior system and assistant turns for compatibility.
    On a semantic cache hit, DISTIRA reuses the stored compiler result directly instead of recomputing it.
4. When `stream=true`, DISTIRA proxies the provider SSE stream; otherwise it returns standard OpenAI-compatible JSON.
5. A `MetricsCollector` (shared via `Arc<Mutex>`) accumulates totals, cache hits, provider counters, upstream lineage stats, last-request trace, and a rolling 50-request history.
6. The backend also serves `GET /version`, reading the root `VERSION` file at runtime rather than relying on a hardcoded string.
7. The Vue dashboard connects once via `EventSource` to `GET /v1/metrics/stream` (SSE), and polls `GET /version` periodically so the sidebar version tag follows the live backend version.

## Crate responsibilities

| Crate | Role |
| --- | --- |
| `core` | Axum HTTP server, health endpoint, orchestrates pipeline |
| `compiler` | Intent detection, context reduction |
| `memory` | Stable block tracking, delta extraction |
| `router` | Policy-aware provider selection |
| `adapters` | HTTP clients per provider (Ollama, OpenAI, Mistral, Gemini) |
| `metrics` | Efficiency score computation |
| `cache` | Prompt fingerprint lookup, semantic caching |
| `fingerprint` | Stable hashing for prompt deduplication |

## MCP Integration Layer

DISTIRA exposes itself as an MCP (Model Context Protocol) tool server for IDE agents like VS Code Copilot Chat.

```text
┌──────────────────────┐     stdio (JSON-RPC 2.0)      ┌─────────────────────────┐
│  VS Code Copilot     │ ────────────────────────────► │  mcp/distira-server.mjs  │
│  Chat (@distira)      │ ◄──────────────────────────── │  Content-Length framing │
└──────────────────────┘                               └──────────┬──────────────┘
                                                                  │ HTTP
                                                       ┌──────────▼──────────────┐
                                                       │  DISTIRA Backend :8080   │
                                                       │  /v1/compile            │
                                                       │  /v1/chat/completions   │
                                                       │  /v1/providers          │
                                                       │  /v1/metrics            │
                                                       └─────────────────────────┘
```

| MCP Tool | Backend Endpoint | Description |
| --- | --- | --- |
| `distira_compile` | `POST /v1/compile` | Compile context through the full pipeline |
| `distira_chat` | `POST /v1/chat/completions` | Compile + forward to routed LLM, with optional SSE streaming |
| `distira_providers` | `GET /v1/providers` | List configured providers |
| `distira_metrics` | `GET /v1/metrics` | Fetch live efficiency metrics |

## Observability model

DISTIRA now exposes two distinct model scopes in telemetry and dashboard views:

- **Upstream model**: what the client or assistant says it is using
- **Routed model**: what DISTIRA actually sends traffic to after routing

This distinction is critical when:

- an IDE or chat client changes selected models dynamically
- DISTIRA rewrites or overrides the routed target for sovereignty or policy reasons
- the dashboard needs to explain why a user-facing `Claude Sonnet` request was actually routed to a local sovereign model

The runtime stores this in `upstream_stats` and `last_request`, which feed the Overview dashboard sections for model scope clarity and last-request lineage.

For deeper operational visibility, `request_history` powers a dedicated Runtime Audit view in the dashboard.
To prevent unnecessary data accumulation, Runtime Audit history is automatically pruned with a time-based retention window (default 7 days) and a bounded entry limit.

If the upstream application cannot propagate model switches directly, DISTIRA can ingest a live runtime client context through `POST /v1/runtime/client-context`; the MCP layer and helper scripts can update this state without restarting the backend.

Runtime version visibility follows the same principle: the dashboard does not hardcode a release number, it asks the backend for the current runtime version.

## Design principles

- **Compile before routing** — never send raw context to an LLM.
- **Local-first** — default to on-prem providers, fall back to cloud.
- **Observable** — every optimization step is visible in the dashboard.
