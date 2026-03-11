# Architecture

KATARA is designed as a sovereign AI context operating layer.
It sits between clients and LLM providers, optimizing every request.

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
│ 7. Provider adapter    │  Forward compiled context to chosen LLM
├────────────────────────┤
│ 8. Telemetry           │  Emit efficiency score and token metrics
└────────────────────────┘
```

## Live data flow

```text
┌─────────────────────┐         POST /v1/compile           ┌──────────────────────────┐
│   Client (curl,     │ ──────────────────────────────────► │   KATARA Rust Backend    │
│   VS Code ext,      │                                     │                          │
│   any AI tool)      │ ◄────── JSON response ───────────── │  compile() → fingerprint │
└─────────────────────┘                                     │   → cache → compiler     │
                                                            │   → memory → router      │
┌─────────────────────┐                                     │                          │
│   Vue Dashboard     │ ◄── SSE /v1/metrics/stream ──────── │  MetricsCollector (Arc)  │
│   (Pinia store)     │     text/event-stream               │   - totals cumulés       │
│                     │     { raw, compiled, reused, ... }   │   - historique 24 pts    │
│   EventSource API   │     toutes les 2 secondes           │   - compteurs par route  │
└─────────────────────┘                                     └──────────────────────────┘
```

**How it works:**

1. Any client sends a `POST /v1/compile` with raw context.
2. The backend runs the full pipeline: fingerprint → semantic cache → compiler → memory lensing → hybrid router → metrics.
3. A `MetricsCollector` (shared via `Arc<Mutex>`) accumulates totals, cache hits, provider counters, and a rolling 24-point history.
4. The Vue dashboard connects once via `EventSource` to `GET /v1/metrics/stream` (SSE) and receives a JSON snapshot every 2 seconds — zero polling, zero WebSocket overhead.

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

## Design principles

- **Compile before routing** — never send raw context to an LLM.
- **Local-first** — default to on-prem providers, fall back to cloud.
- **Observable** — every optimization step is visible in the dashboard.
