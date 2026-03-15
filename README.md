# DISTIRA

![DISTIRA](brand/distira_banner_1600x900.svg)

> **Distira is a Sovereign AI Context Operating System.**
> It compiles, minimizes, and governs context before every LLM call.

[![CI](https://github.com/distira-project/distira/actions/workflows/ci.yml/badge.svg?branch=wip-chf)](https://github.com/distira-project/distira/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-10.8.0-brightgreen.svg)](VERSION)

> **Distira reduces token waste before the model call — not after it.**

---

## The problem

Every LLM call carries too much context: logs, traces, diffs, histories, noise.
You pay for tokens that don't contribute to the answer.
You route blindly between local and cloud models.
You have no visibility into what is actually sent, why, and at what cost.

Existing tools are proxies, routers, and dashboards.
None of them touch the context itself.

---

## What Distira does differently

Distira is not a proxy. It is the layer **between your intent and the model call**.

```md
Raw context (12 000 tokens)
        │
   [ Context Budget Compiler ]   →  compiled:  2 400 tokens  (−80%)
        │
   [ Context Memory Lensing ]    →  reused:    5 000 tokens  (avoided)
        │
   [ Hybrid Sovereign Router ]   →  cloud call avoided: yes / no
        │
   [ AI Flow Visualizer ]        →  every step visible, every gain measurable
        │
Model call (2 400 tokens instead of 17 000)
```

Every saving is **provable**:

| Signal | Example value |
| --- | --- |
| Raw context | 12 000 tokens |
| Compiled context | 2 400 tokens (−80%) |
| Memory reuse | 5 000 tokens avoided |
| Cloud call avoided | yes |
| Cost: raw vs optimized | €0.036 → €0.007 |
| AI Efficiency Score | 81% |

---

## The 4 differentiating building blocks

### A. Context Budget Compiler

Reduces logs, stack traces, diffs, conversation histories, and transcripts.
Extracts signal. Removes noise. Cuts tokens before they reach a model.

### B. Context Memory Lensing

Builds a structured memory of stable context blocks.
Sends only what is new, changed, or still relevant.
This is the core innovation: **delta-first forwarding**.

### C. Hybrid Sovereign Routing

Chooses the right provider — local, private, or cloud — based on:
confidentiality, cost, latency, policy, and model capability.
Sensitive data never leaves local. Always.

### D. AI Flow Visualizer

Makes every optimization step visible in a live dark dashboard.
Shows before/after, cloud vs local, reused context, and real efficiency gains.
Demonstrable. Memorable. Trustworthy.

---

## Not a gateway. Not a proxy

The market already has proxies, routers, fallback logic, cost dashboards, and guardrails.
If Distira stopped there, it would be compared to Kong AI, LiteLLM, or PortKey.

Distira's position is **context-first**:

| Capability | Proxy / Gateway | Distira |
| --- | --- | --- |
| Multi-provider routing | ✅ | ✅ |
| Semantic cache | ✅ partial | ✅ |
| **Context compilation** | ❌ | ✅ |
| **Memory lensing / delta forwarding** | ❌ | ✅ |
| **Sovereignty enforcement (local-first)** | ❌ | ✅ |
| **Per-request proof of savings** | ❌ | ✅ |
| **AI Flow Visualizer** | ❌ | ✅ |

## Architecture

```text
Clients / IDE / Agents
        │
  OpenAI-compatible API
        │
      DISTIRA
        │
  ┌─────┴─────┐
  │  Intent   │
  │  Detector │
  └─────┬─────┘
        │
  ┌─────┴──────────┐
  │ Context Budget │
  │ Compiler       │
  └─────┬──────────┘
        │
  ┌─────┴──────────┐
  │ Context Memory │
  │ Lensing        │
  └─────┬──────────┘
        │
  ┌─────┴──────────┐
  │ Semantic Cache │
  └─────┬──────────┘
        │
  ┌─────┴──────────┐
  │ Hybrid Router  │
  └─────┬──────────┘
        │
  Local / Private / Cloud Providers
```

## Live data flow

```text
┌─────────────────────┐         POST /v1/compile            ┌──────────────────────────┐
│   Client (curl,     │ ──────────────────────────────────► │   DISTIRA Rust Backend   │
│   VS Code ext,      │                                     │                          │
│   any AI tool)      │ ◄────── JSON response ───────────── │  compile() → fingerprint │
└─────────────────────┘                                     │   → cache → compiler     │
                                                            │   → memory → router      │
┌─────────────────────┐                                     │                          │
│   Vue Dashboard     │ ◄── SSE /v1/metrics/stream ──────── │  MetricsCollector (Arc)  │
│   (Pinia store)     │     text/event-stream               │   - cumulative totals    │
│                     │     { raw, compiled, reused, ... }  │   - 24-point history     │
│   EventSource API   │     every 2 seconds                 │   - per-provider counts  │
└─────────────────────┘                                     └──────────────────────────┘
```

Every `POST /v1/compile` runs the full pipeline (fingerprint → cache → compiler → memory → router → metrics) and feeds a shared `MetricsCollector`. The Vue dashboard auto-connects via SSE and updates in real time — no polling, no WebSocket.

Runtime operational data is persisted automatically to `cache/runtime-state.json` and restored on backend startup. This keeps metrics/audit/cache history transparent across backend restarts.

## Compatibility

DISTIRA connects to any **OpenAI-compatible** endpoint — no code changes required, only a `providers.yaml` entry.

| Runtime / Frontend | Default port | Type in providers.yaml | Notes |
|---|---|---|---|
| **Ollama** | `:11434/v1` | `openai-compatible` | `ollama pull <model>` |
| **vLLM** | `:8000/v1` | `openai-compatible` | `python -m vllm.entrypoints.openai.api_server` |
| **LM Studio** | `:1234/v1` | `openai-compatible` | Enable local server in the UI |
| **OpenWebUI** | `:3000/api` | `openai-compatible` | Proxies Ollama or any backend |
| **OpenAI** | `api.openai.com/v1` | `openai-compatible` | Requires `OPENAI_API_KEY` |
| **Anthropic Claude** | `api.anthropic.com/v1` | `openai-compatible` | Requires `ANTHROPIC_API_KEY` |
| **Google Gemini** | `generativelanguage.googleapis.com/v1beta/openai` | `openai-compatible` | Requires `GOOGLE_API_KEY` |
| **Mistral AI** | `api.mistral.ai/v1` | `mistral` | Requires `MISTRAL_API_KEY` |
| **OpenRouter** | `openrouter.ai/api/v1` | `openai-compatible` | Requires `OPENROUTER_API_KEY` |
| **ZhipuAI GLM** | `open.bigmodel.cn/api/paas/v4` | `openai-compatible` | Requires `ZHIPU_API_KEY` |
| **DashScope (Qwen)** | `dashscope.aliyuncs.com/compatible-mode/v1` | `openai-compatible` | Requires `DASHSCOPE_API_KEY` |

All providers support **streaming** (`stream: true`). Sensitive requests are **always** forced to on-prem regardless of routing config.  
Ready-to-use commented entries for every provider above are in [`configs/providers/providers.yaml`](configs/providers/providers.yaml).

## Monorepo layout

| Directory | Purpose |
| --- | --- |
| `core/` | Axum HTTP server — API entry point and pipeline orchestration |
| `compiler/` | Context Budget Compiler — intent detection and context reduction |
| `memory/` | Context Memory Lensing — stable block store and delta engine |
| `router/` | Hybrid Sovereign Router — provider selection by policy and intent |
| `adapters/` | Provider-specific HTTP clients (Ollama, OpenAI, Mistral, OpenRouter) |
| `metrics/` | AI Efficiency Score computation and telemetry |
| `cache/` | Semantic cache — fingerprint lookup and compiled context store |
| `fingerprint/` | Prompt fingerprint graph for deduplication |
| `dashboard/ui-vue/` | AI Flow Visualizer — Vue 3 + Vite dark dashboard |
| `configs/` | Provider, routing, policy, and workspace configuration |
| `deployments/` | Docker, Kubernetes, and Helm manifests |
| `docs/` | Architecture and implementation notes |
| `examples/` | Quick integration examples |
| `mcp/` | MCP server for VS Code Copilot integration |
| `benchmarks/` | Reproducible token-reduction fixtures |

## Quick start

### Windows — usage quotidien

```powershell
# First-time only (installs Rust, Node.js, builds crates, npm deps)
.\scripts\bootstrap-win.ps1

# Every day — starts Ollama + backend (release binary) + Vue dashboard
.\scripts\start-win.ps1
```

> **MCP** — lancé automatiquement par VS Code dès l'ouverture du dossier (`mcp.json`). Rien à faire manuellement.

`bootstrap-win.ps1` n'est nécessaire qu'une seule fois (ou après `git clone` / mise à jour des dépendances).  
`start-win.ps1` détecte si les sources ont changé et recompile uniquement si nécessaire — le démarrage habituel prend quelques secondes.

### Linux / macOS

```bash
# First-time setup
./scripts/bootstrap.sh

# Daily start
cargo build --release -p core && ./target/release/core &
cd dashboard/ui-vue && npm run dev
```

### Manual

```bash
# Rust backend (debug)
cargo run -p core

# Vue dashboard (separate terminal)
cd dashboard/ui-vue && npm run dev

# MCP server (VS Code manages it automatically via .vscode/mcp.json)
# Manual test only: cd mcp && node distira-server.mjs
```

### Secrets management

API keys are stored in a `.env` file at the project root.
This file is **excluded from Git** (listed in `.gitignore`).

```bash
cp .env.example .env
# Edit .env with your real keys
```

See `.env.example` for the expected variables.

### Optional API key authentication

To restrict access to `/v1/*` routes, set the `DISTIRA_API_KEY` environment variable before starting the server.

```bash
export DISTIRA_API_KEY=my-secret-key
./scripts/start.sh
```

When set, every `/v1` request must include the header:

```md
Authorization: Bearer my-secret-key
```

If `DISTIRA_API_KEY` is not set, all routes remain open (default for local development).
`/healthz` and `/version` are always public.

### Cache TTL

The semantic cache evicts entries older than 24 hours by default.
Override with:

```bash
export DISTIRA_CACHE_TTL_SECS=3600  # 1 hour
```

## VS Code Agent Integration

DISTIRA ships with a built-in MCP (Model Context Protocol) server.
Once configured, type `@distira` in VS Code Copilot Chat to invoke DISTIRA tools directly.

```text
Copilot Chat  →  @distira  →  MCP stdio  →  distira-server.mjs  →  localhost:8080
```

| Tool | Description |
| --- | --- |
| `distira_compile` | Compile raw context through the full pipeline |
| `distira_chat` | Compile + forward to routed LLM |
| `distira_set_client_context` | Update the live upstream client model/provider context |
| `distira_providers` | List configured providers |
| `distira_metrics` | Fetch live metrics snapshot |

The chat endpoint also supports `stream=true` and proxies OpenAI-compatible SSE responses from the routed provider.
It preserves full message history (`system`, `assistant`, `user`) and forwards extra OpenAI-compatible request options like `temperature` to the routed provider.
The semantic cache now stores the full compiler result, so repeated prompts can reuse the same `compiled_context` without recompiling before routing.

The MCP server uses `@modelcontextprotocol/sdk` v1.27.1 with stdio transport.
Dependencies are installed in `mcp/node_modules/` — run `npm install` inside `mcp/` if pulling fresh.

The MCP layer now forwards client lineage metadata automatically to DISTIRA:

- `client_app`: defaults to `VS Code Copilot Chat`
- `upstream_model`: resolved per request from the tool's `model`, MCP `_meta`, or an optional runtime resolver command
- `upstream_provider`: resolved per request from MCP `_meta`, a runtime resolver command, or inferred from the upstream model family

This is what lets the dashboard distinguish the user-facing assistant/client model from the model actually routed by DISTIRA.

DISTIRA now also performs a best-effort scan of MCP request metadata for generic model/provider fields when clients expose them without the custom `distira/*` keys. This improves automatic detection of Copilot-selected models such as `GPT-5.4` when that information is actually present.

The Overview now also exposes a live `Last Request` panel showing:

- upstream client app, provider, and model
- routed provider and routed model
- cache hit vs miss
- sensitive override vs standard routing

The Overview also has a dedicated `Upstream Client Models` table so a model such as `GPT-5.4` selected in VS Code Copilot is visible separately from the routed model efficiency table.
When the client does not expose its selected model, the Overview now shows a prominent warning banner instead of silently implying that the upstream model is known.

The dashboard also includes a `Runtime Audit` view backed by the rolling `request_history` snapshot so operators can inspect the latest routed requests without opening raw JSON metrics.

When the upstream client cannot send its selected model directly, update the live context with:

```powershell
.\scripts\set-upstream-context.ps1 -UpstreamProvider Anthropic -UpstreamModel "Claude Sonnet 4.6"
```

or through the MCP tool `distira_set_client_context`.

See [INSTALL.md](INSTALL.md#vs-code-agent-mcp) for setup instructions and [TESTING.md](TESTING.md#mcp-agent-tests-vs-code) for validation steps.

## Workflow Schema (MCP -> Distira Agent -> Distira App)

```text
┌──────────────────────────────┐
│ VS Code Copilot Chat         │
│ (user prompt: @distira ...)   │
└──────────────┬───────────────┘
               │
               │ MCP stdio (JSON-RPC 2.0)
               ▼
┌──────────────────────────────┐
│ Distira MCP Server           │
│ mcp/distira-server.mjs       │
│ - distira_compile            │
│ - distira_chat               │
│ - distira_metrics            │
│ - distira_providers          │
└──────────────┬───────────────┘
               │
               │ HTTP (localhost:8080)
               ▼
┌──────────────────────────────┐
│ Distira App (Rust backend)   │
│                              │
│ core + compiler + memory     │
│ cache + router + metrics     │
│ /v1/compile                  │
│ /v1/chat/completions         │
│ /v1/metrics                  │
│ /v1/providers                │
└──────────────┬───────────────┘
               │
               │ Routed request (policy + intent)
               ▼
┌──────────────────────────────┐
│ LLM Providers                │
│ local / private / cloud      │
└──────────────────────────────┘
```

## Version

Current runtime version: **10.2.0** — served from [VERSION](VERSION) and exposed live via `GET /version`.

See [CHANGELOG.md](CHANGELOG.md) for release history and [ROADMAP.md](ROADMAP.md) for planned iterations.

## Testing

See [TESTING.md](TESTING.md) for the complete verification guide:
curl smoke tests, intent routing matrix, MCP agent tests, and a PowerShell quick-test script.

## Status

This is a **V8.0 runtime**: production-hardened, GitHub-ready, and implementation-oriented.

What's solid:
- Deterministic semantic cache fingerprinting (FnvHasher, stable across restarts)
- Cache TTL eviction (default 24h, configurable)
- Accurate BPE token estimation (chars ÷ 4, ±10% vs GPT tokenizers)
- Optional Bearer API key auth on all `/v1/*` routes
- Provider pricing table with `cost_per_1k_input_tokens` / `cost_per_1k_output_tokens`
- Real benchmark fixtures (77–88% token reduction measured across 6 scenarios)
- 62 unit tests, 0 failures

Live: benchmarks, MCP agent integration, per-intent metrics, multi-tenant scoping, transparent runtime persistence.
Provider adapters (`/v1/chat/completions`) forward to real Ollama and Mistral cloud endpoints.
Work in progress: provider latency instrumentation, adaptive quality guardrails (V8.x).

---

## Author

Distira is designed and built by **Christophe Freijanes**.

This project is my public exploration of a question I find genuinely important:
> *What if the intelligence layer was not the model, but the context you give it?*

If Distira resonates with you — whether you use it, fork it, benchmark it, or just think the idea is worth pursuing — I'd love to connect.

- GitHub: [@christophefreijanes](https://github.com/christophefreijanes)
- LinkedIn: [Christophe Freijanes](https://www.linkedin.com/in/christophefreijanes)

⭐ Star the repo if you find it useful. It helps the project get discovered.

## License

[AGPL-3.0](LICENSE) — Free and open-source. Copyright 2024–2026 Christophe Freijanes and DISTIRA contributors.

Use it, fork it, build on it. If you distribute a modified version or run it as a service, publish your source under the same license.
