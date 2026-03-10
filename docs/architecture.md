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
