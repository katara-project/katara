# KATARA Roadmap

## Completed iterations

| Version | Milestone |
| --- | --- |
| V1 | Concept prototype |
| V2 | Multi-OS foundation |
| V3 | Gateway MVP |
| V4 | Enterprise architecture |
| V5 | Predictive AI gateway concepts |
| V6.1 | Functional gateway baseline |
| V6.2 | Hybrid routing engine |
| V6.3 | KATARA rebrand |
| V6.4 | Context Budget Compiler framing |
| V6.5 | Optimization layer + Vue dashboard direction |
| V6.6 | AI Flow Visualizer + Context Memory Lensing scaffold |
| V6.7 | Win11 monorepo bootstrap scaffold |

## Current iteration

### V7.0.0 — Advanced Sovereign AI Context OS

**Status:** Delivered as advanced scaffold.

- Stronger monorepo structure with 8 Rust workspace crates
- Vue 3 + Vite dashboard with dark design system
- OpenAI-compatible API contract
- Hybrid provider configuration (Ollama, OpenAI-compatible, Mistral)
- Prompt fingerprint graph scaffolding
- Context Memory Lensing modules
- AI Flow Visualizer view
- Benchmark fixtures and methodology
- GitHub Actions CI + release workflows
- Community files (CONTRIBUTING, SECURITY, GOVERNANCE, CODE_OF_CONDUCT)

### V7.0.1 — Runtime Hardening + MCP Agent (2026-03-11)

**Status:** Delivered.

- **MCP Server** (`mcp/katara-server.mjs`) — stdio-based, 4 tools: compile, chat, providers, metrics
- **VS Code Agent** (`.github/agents/katara.agent.md`) — `@katara` in Copilot Chat
- **Live Benchmarks** — `BenchmarksView.vue` rewritten with real-time SSE data (no more demo data)
- **Per-intent metrics** — `IntentStats` in `MetricsSnapshot` (requests, raw_tokens, compiled_tokens per intent)
- **OCR routing** — `ocr` intent routes to `mistral-cloud` via `TaskRouting`
- `.env` + `.env.example` secret management (keys never in Git)
- `.cargo/config.toml` — build redirect for Google Drive workspaces
- Mistral OCR cloud provider configured (`mistral-ocr-2512`)
- Optimized hybrid routing:
  - `review` → `ollama-qwen2.5-coder` (best local code model)
  - `debug` → `ollama-mistral`
  - `summarize` / `general` → `ollama-llama3`
  - `ocr` → `mistral-cloud` (cloud quality)
  - `sensitive` → always local (sovereign override)
- Updated documentation: README, INSTALL, ROADMAP, CHANGELOG, architecture

## Next iterations

### V7.1 — Provider Runtime

- Real OpenAI-compatible HTTP endpoints (`/v1/chat/completions`)
- Ollama adapter fully wired with streaming
- OpenAI-compatible generic adapter
- Response streaming support (SSE)

### V7.2 — Compiler Runtime

- Real reducer pipeline with configurable stages
- Log trimming (keep last N lines, deduplicate repeated frames)
- Diff compaction (collapse unchanged hunks)
- Transcript summarization strategies

### V7.3 — Memory Runtime

- Stable context block persistence (file-backed + optional Redis)
- Delta extraction engine
- Context reuse metrics based on real state

### V7.4 — Visual Intelligence Console

- Animated flow visualizer with real-time request paths
- Request timeline and latency breakdown
- Top prompt families by token cost
- Optimization advisor panel

### V8 — Enterprise Control Plane

- Role-based access control (RBAC)
- Audit trail and compliance logging
- Multi-tenant org / team / project model
- Policy packs (GDPR, SOC2, HIPAA templates)

### V9 — Adaptive AI Optimization Network

- Learning routing loop (feedback from response quality)
- Provider capability graph
- Automated optimization recommendations
