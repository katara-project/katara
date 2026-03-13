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

### V7.0.2 — Model Lineage Visibility + Provider Catalog (2026-03-12)

**Status:** Delivered.

- Upstream request lineage added to runtime contracts: `client_app`, `upstream_provider`, `upstream_model`
- Live metrics snapshot now distinguishes upstream client model vs KATARA-routed model
- Overview dashboard clarifies model scope to avoid confusing assistant/client models with routed provider models
- `/v1/providers` now exposes both provider keys and rich runtime details (type, deployment, base URL, model, description)
- Foundation laid for per-client and per-tenant LLM observability

## Next iterations

### V7.1 — Provider Runtime (2026-03-12)

**Status:** Delivered.

- Rich provider catalog delivered through `/v1/providers`
- Upstream model recognition contract delivered for client integrations
- MCP server now forwards upstream client/model/provider lineage automatically for VS Code agent usage
- MCP server now supports per-request runtime resolution for upstream model/provider/client context
- Initial `/v1/chat/completions` streaming delivered via SSE proxy for OpenAI-compatible providers
- Streamed responses are now cached after completion for subsequent cache hits
- Runtime Audit view now exposes rolling routed request history in the dashboard
- `/v1/chat/completions` now preserves full OpenAI-compatible message history and forwards extra request options like `temperature`
- Compiler runtime now emits a concrete `compiled_context` and the chat runtime injects it into the latest user turn before provider forwarding
- Semantic cache now persists full compiler results so repeated compile/chat requests can reuse `compiled_context` directly
- Intent detection priority fixed: `review` > `summarize` to correctly route pull-request analysis to `ollama-qwen2.5-coder`

## Next iterations

### V7.2 — Compiler Runtime (2026-03-12)

**Status:** Delivered.

- `ocr` intent added to `detect_intent()` — routes to `mistral-ocr-cloud` (cloud quality)
- Enhanced `reduce_debug_context` — tiered signal extraction: error/panic headlines (priority 1) + file:line references `.rs:`, `.py:`, `.go:` etc. (priority 2)
- New `reduce_ocr_context` — normalises and deduplicates, preserves full raw content (OCR tasks require complete input)
- Intent detection extended with OCR keyword set: `ocr`, `scan image`, `extract text from`, `image to text`, `read this image`
- Multi-turn conversation history compression (`compress_conversation_history`): conversations > 6 turns have older turns collapsed into a compact system-message summary before forwarding, reducing provider token spend for long sessions

### V7.3 — Memory Runtime (2026-03-12)

**Status:** Delivered.

- `ContextStore` — real in-memory context block store replacing the mock `summarize_memory()` stub
- `ContextStore::register(fingerprint, compiled)` — stores compiled context keyed by semantic fingerprint
- `ContextStore::compute_reuse(fingerprint, raw_tokens)` — returns real reuse ratio derived from previously compiled blocks (not a synthetic 50% estimate)
- `ContextStore` is wired into `MetricsCollector` and replaces `memory::summarize_memory()` in both compile and chat handlers
- `context_reuse_ratio` in compile responses now reflects actual token reuse: 100% on cache hit, 0% on first-seen context
- 3 new memory tests: `context_store_registers_and_computes_reuse`, `context_store_miss_returns_zero`, `context_store_caps_reuse_at_raw`

### V7.4 — Visual Intelligence Console (2026-03-12)

**Status:** Partially delivered.

- **Intent Distribution panel** in `OverviewView.vue` — per-intent tile showing request count, % share bar, and average token reduction rate
- Intent tiles colour-coded by intent type: debug, review, summarize, ocr, general
- `intentRows` computed property in dashboard aggregates `intentStats` from live SSE metrics stream
- Remaining V7.4 items (animated flow visualiser, request latency timeline, optimization advisor panel) carried forward

### V7.5 — Codegen-specialized routing (2026-03-12)

**Status:** Delivered.

- New `codegen` intent in compiler runtime to distinguish code-generation prompts from generic queries
- `codegen` intent routed to `ollama-qwen2.5-coder` in routing policy for best-in-class local code assistance
- Intent detection tuned for English/French codegen phrasing ("write a Rust function", "écris du code en Rust", "generate code example in Python", etc.)
- Keeps `review` on `ollama-qwen2.5-coder` and `general` on `openrouter-step-3.5-flash-cloud` so reasoning-centric queries still benefit from Step while code tasks go to Qwen

### V7.6 — Visual Intelligence Console II (2026-03-12)

**Status:** Partially delivered.

- Animated flow visualiser enhanced with live edges and dynamic highlighting of the active routing branch (Local LLM / Cloud / Mid-tier) in the FlowVisualizer
- Dedicated "Codegen vs Review" slice in the Overview to track how much traffic is pure generation vs review/refactor, with per-intent token reduction stats
- Per-intent efficiency deltas surfaced via the Intent Distribution panel and the Codegen vs Review card (e.g., how much `codegen` and `review` benefit from compiler and memory reductions)
- Optimization advisor panel implemented in the Insights view, generating live recommendations from efficiency, cache and sovereignty metrics
- Agent governance reinforced in `.github/agents/katara.agent.md` with an essentials-first workflow so users no longer need to repeatedly request baseline actions (review/validation/docs/version updates)
- Runtime Audit now includes automatic retention guardrails (default 7-day TTL + bounded history size) to avoid unnecessary data accumulation
- Token Trends chart readability improved with hour-based X-axis labels and sparse tick rendering for better 24h visibility
- Token Trends now supports true hour-by-hour granularity over the last 24h using backend hourly buckets
- Token Trends now includes interactive window controls (`1h` / `6h` / `24h`) for faster operational readability
- Token Trends header now displays a live last-update timestamp based on metrics stream (`lastTs`) for better operational traceability
- Agent instructions now enforce an automatic to-do list setup before substantial tasks
- Remaining: request latency timeline (end-to-end and/or per-segment) to visualise performance trends over time

### V7.7 — Governance & Multi-tenant foundations (2026-03-12)

**Status:** Delivered.

- Workspace-level tenant/project foundation added via `configs/workspace/workspace.yaml` (with optional `policy_pack`) for future per-tenant routing and policy layers
- Runtime context API (`/v1/runtime/client-context`) now supports `tenant_id` and `project_id`, and compile/chat requests can override or inherit those values
- Runtime Audit view now exposes scope information (`tenant_id` / `project_id`) alongside who/when/intent/provider lineage
- Audit lineage schema extended with `tenant_id`, `project_id`, and `policy_pack`, establishing groundwork for future RBAC/policy-pack enforcement

### V7.7.1 — Audit usability (2026-03-12)

**Status:** Delivered.

- Runtime Audit now supports tenant/project filtering directly in the dashboard
- Runtime Audit now supports scoped CSV export (`tenant/project`) for lightweight operational reporting
- Runtime Audit now supports temporal filtering (`24h` / `7d` / `custom`) and applies the same scope to CSV exports
- Runtime Audit filter state is now persisted in browser localStorage (tenant/project/time/custom range) and auto-restored after refresh/restart
- Version consistency audit completed: runtime, MCP fallback, npm manifests, and operator docs aligned on `7.7.1`

## Future iterations

- Role-based access control (RBAC)
- Audit trail and compliance logging
- Multi-tenant org / team / project model
- Policy packs (GDPR, SOC2, HIPAA templates)

### V9 — Adaptive AI Optimization Network

- Learning routing loop (feedback from response quality)
- Provider capability graph
- Automated optimization recommendations
