# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — V7.7 Governance & Multi-tenant foundations (2026-03-12)

- Added workspace scope config file `configs/workspace/workspace.yaml` with `tenant_id`, `project_id`, and optional `policy_pack`
- Core runtime now loads workspace scope defaults and resolves effective request scope from request payload, runtime client context, and workspace defaults
- `/v1/runtime/client-context` now supports `tenant_id` and `project_id` updates
- Compile and chat contracts now accept and propagate `tenant_id` / `project_id` in `katara` metadata
- Runtime audit lineage now includes `tenant_id`, `project_id`, and `policy_pack`
- Dashboard Runtime Audit view now displays tenant/project scope per request

### Added — V7.7.1 Audit usability (2026-03-12)

- Runtime Audit dashboard now includes tenant/project filters for scoped incident and compliance triage
- Runtime Audit dashboard can export filtered request history to CSV for quick reporting workflows
- Runtime Audit dashboard now includes time-window filters (`24h` / `7d` / `custom`) and CSV export filenames include filter scope
- Runtime Audit filter state is now persisted in localStorage and restored on reload (`tenant`, `project`, `time`, `custom range`)

### Changed — V7.7.1 consistency audit (2026-03-12)

- Runtime/version surfaces aligned to `7.7.1` across `VERSION`, MCP fallback, dashboard and MCP package manifests
- Operator docs (`README.md`, `INSTALL.md`, `TESTING.md`) now reference the current runtime/version examples (`7.7.1`)

### Changed — Version alignment (2026-03-12)

- Root `VERSION` bumped to `7.7.1`
- MCP fallback version in `mcp/katara-server.mjs` updated to `7.7.1`

### Changed — Agent governance + version alignment (2026-03-12)

- **Agent essentials-first policy** in `.github/agents/katara.agent.md`: the agent now defaults to logical baseline actions (review/validation/docs/versioning) without requiring repeated user reminders
- **Version bump**: root `VERSION` updated from `7.5.0` to `7.6.0` so runtime `/version` and dashboard tag remain aligned with current iteration
- **MCP fallback alignment**: `mcp/katara-server.mjs` fallback version updated to `7.6.0`
- **ROADMAP sync**: V7.6 now explicitly tracks the delivered agent-governance improvement

### Added — Runtime Audit retention guardrails (2026-03-12)

- `core` now applies automatic Runtime Audit pruning with a default **7-day TTL** (`KATARA_AUDIT_RETENTION_DAYS`, default `7`)
- Runtime Audit history now also enforces a max entry cap (`KATARA_AUDIT_HISTORY_LIMIT`, default `2000`) to bound memory usage
- Added unit test `prune_request_history_respects_ttl_and_limit` to validate both temporal pruning and size capping

### Improved — Dashboard readability + repository hygiene (2026-03-12)

- `Token Trends (24h)` now uses hour-based labels (`HH:mm`) instead of index-style labels
- Chart X-axis labels are now sparsified to keep the timeline readable on dense data and smaller screens
- Local `build*.txt` artifacts removed and ignored via `.gitignore` so they do not pollute Git history
- `INSTALL.md` now documents `KATARA_AUDIT_RETENTION_DAYS` and `KATARA_AUDIT_HISTORY_LIMIT` with recommended production defaults
- `Token Trends (24h)` now uses backend hourly buckets for true hour-by-hour granularity (instead of sample-index style trends)
- Added interactive `1h / 6h / 24h` window controls for Token Trends to improve readability during operations
- Added a `Last update` timestamp in the Token Trends header, sourced from live metrics `lastTs`

### Changed — Agent execution ergonomics (2026-03-12)

- `.github/agents/katara.agent.md` now explicitly requires automatic to-do list setup and updates before substantial tasks, reducing repeated user prompting and improving execution flow

### Added — V7.2 Compiler Runtime (2026-03-12)

- **OCR intent** — `detect_intent()` now recognises `ocr`, `scan image`, `extract text from`, `image to text`, `read this image` and routes to `mistral-ocr-cloud`
- **`reduce_ocr_context()`** — new OCR-specific reducer: normalises and deduplicates content while preserving full input (OCR tasks cannot lose data)
- **Enhanced `reduce_debug_context()`** — tiered signal extraction: Tier 1 = error/panic/exception headlines, Tier 2 = stack-trace + file:line references (`.rs:`, `.py:`, `.js:`, `.go:`, etc.)
- **`compress_conversation_history()`** — conversations exceeding 6 turns have older turns collapsed into a compact system-message summary before provider forwarding, preventing unbounded token growth in long sessions
- 2 new compiler tests: `detect_ocr_intent`, `debug_compiler_keeps_file_line_references`
- 2 new core tests: `compress_history_pass_through_short_conversation`, `compress_history_summarises_older_turns`

### Added — V7.3 Memory Runtime (2026-03-12)

- **`memory::ContextStore`** — real in-memory context block store replacing the synthetic `summarize_memory()` stub
  - `register(fingerprint, compiled)` stores compiled context keyed by semantic fingerprint
  - `compute_reuse(fingerprint, raw_tokens)` returns actual reuse ratio (100% on cache hit, 0% on miss)
- **`ContextStore` integrated into `MetricsCollector`** — wired through both compile and chat handlers
- `context_reuse_ratio` in API responses now reflects real token reuse rather than a fixed 50% estimate
- 3 new memory tests: `context_store_registers_and_computes_reuse`, `context_store_miss_returns_zero`, `context_store_caps_reuse_at_raw`

### Added — V7.4 Visual Intelligence Console (partial) (2026-03-12)

- **Intent Distribution panel** in dashboard `OverviewView.vue`: per-intent coloured tiles showing request count, % share bar, average token reduction, and cumulative total badge
- Intent tiles colour-coded: debug (red), review (blue), summarize (green), ocr (purple), general (neutral)

### Fixed (2026-03-12)

- Intent detection priority reordered: `review` now checked before `summarize` so pull-request/diff prompts route correctly to `ollama-qwen2.5-coder` instead of `openai-cloud`

- **`scripts/test-api.ps1`**: PowerShell quick-test script (7 assertions covering health, providers, intents, sensitive mode, metrics)
- **`mcp/test-mcp.mjs`**: standalone MCP handshake test harness (spawns server, sends `initialize` + `tools/list`, validates responses without VS Code)
- **Dynamic upstream lineage resolution** in `mcp/katara-server.mjs`: request-by-request client/model/provider detection via MCP metadata or runtime resolver command
- **`scripts/resolve-upstream-context.ps1`**: example runtime resolver for dynamic upstream model context
- **`scripts/set-upstream-context.ps1`**: helper to update the live upstream client context without restarting the backend
- **Overview Last Request panel** in `dashboard/ui-vue/src/views/OverviewView.vue` showing upstream model, routed model, cache state, and sensitivity flag
- **Initial SSE streaming support** for `POST /v1/chat/completions` when `stream=true`
- **Runtime Audit view** in `dashboard/ui-vue/src/views/AuditView.vue` backed by rolling `request_history`
- **Live client-context API**: `GET/POST /v1/runtime/client-context`
- **Dynamic runtime version display**: backend version now served from root `VERSION`, dashboard sidebar now reads it live from `/version`
- **OpenAI-compatible chat passthrough**: `/v1/chat/completions` now preserves full message history and forwards extra request options such as `temperature`
- **Compiler runtime output**: `CompileResult` now includes `compiled_context`, and the chat runtime uses it to rewrite the latest user turn before forwarding
- **Semantic cache reuse**: semantic cache entries now persist full compiler outputs so repeated compile/chat requests can skip recompilation and reuse the same reduced payload
- **Dashboard scope clarity**: Overview now has a dedicated upstream client models table, and routed-model efficiency is labeled explicitly so `GPT-5.4` upstream is not confused with the Katara-routed target
- **Best-effort Copilot model detection**: the MCP bridge now scans generic metadata fields for upstream model/provider information when clients expose it without `katara/*` keys, and the dashboard warns clearly when upstream model metadata is missing

### Improved

- License: Apache 2.0 → AGPL-3.0 + Commons Clause (protection against unauthorized commercial resale)
- `.gitignore`: expanded to 70+ rules covering Rust, Node, IDE, secrets, IaC, Docker, OS artifacts
- `policies.yaml`: added `terms` property and `fallback_provider`, `log_level`, `data_residency` fields
- `mcp/katara-server.mjs`: migrated from custom Buffer-based stdio transport to official `@modelcontextprotocol/sdk` v1.27.1 (`McpServer` + `StdioServerTransport` + Zod tool schemas) — eliminates Windows stdin hang
- `.vscode/mcp.json`: added `"cwd": "${workspaceFolder}/mcp"` so Node resolves SDK imports from `mcp/node_modules/`
- `scripts/start-win.ps1`: replaced `Get-NetTCPConnection` (unreliable) with `netstat -ano` for port 8080 process detection; passes `$cargoPath` explicitly to `Start-Job` to avoid PATH inheritance failures
- `scripts/bootstrap-win.ps1`: adds `~\.cargo\bin` to PATH before `Get-Command cargo` check to detect freshly installed Rust
- Documentation alignment across `README.md`, `INSTALL.md`, `TESTING.md`, `ROADMAP.md`, and `docs/architecture.md` for dynamic upstream lineage and streaming behavior
- Streamed chat responses are now cached after stream completion and can produce later cache hits
- Chat cache keys now account for the full forwarded message payload and request options rather than only the last user message

### Fixed

- `compiler::compile_context`: compiled tokens now capped at raw token count (`.min(raw_tokens_estimate)`)
- Added `compile_floor_applies_above_threshold` test to compiler
- Moved `workflows/` to `.github/workflows/` for GitHub Actions compatibility
- Fixed README bootstrap script reference (`bootstrap-win.ps1`)
- Hardened `.gitignore` (added `.env`, `*.zip`, `Thumbs.db`)
- Added `---` document start markers to all YAML configs
- Expanded all documentation to canonical standards
- Added unit tests to every Rust crate
- Replaced `|| true` guards in CI with proper error handling
- Added Dockerfile `EXPOSE`, healthcheck, and non-root user
- Converted bootstrap scripts from echo-only to functional installers

## [7.0.1] — 2026-03-11

### Added

- **MCP Server** (`mcp/katara-server.mjs`): stdio-based Model Context Protocol server exposing 4 tools — `katara_compile`, `katara_chat`, `katara_providers`, `katara_metrics`
- **VS Code Agent** (`.github/agents/katara.agent.md`): custom Copilot agent invoking KATARA tools via `@katara`
- **MCP registration** (`.vscode/mcp.json`): VS Code discovers the MCP server automatically
- **Live Benchmarks**: `BenchmarksView.vue` now consumes real-time SSE data instead of hardcoded demo values
- **Per-intent metrics**: `IntentStats` struct in `MetricsSnapshot` tracks requests, raw tokens, and compiled tokens per intent
- **OCR routing**: added `ocr` task routing to `TaskRouting` in `router/src/lib.rs` (→ `mistral-cloud`)
- **Secret management**: `.env` + `.env.example` pattern for API keys (`.env` gitignored)
- **Google Drive workaround**: `.cargo/config.toml` redirects `target/` to `C:/katara-target`

### Changed

- `core/src/main.rs`: `MetricsSnapshot` now includes `intent_stats: HashMap<String, IntentStats>`; `record()` accepts `intent` parameter
- `router/src/lib.rs`: `TaskRouting` struct gains `ocr: Option<String>` field
- `dashboard/ui-vue/src/store/metrics.ts`: added `IntentStat` interface and `intentStats` ref from SSE
- `dashboard/ui-vue/src/views/BenchmarksView.vue`: complete rewrite — computed properties from SSE, 6-column grid, Live/Offline badge
- `configs/routing/routing.yaml`: optimized per-intent routing (review → qwen2.5-coder, ocr → mistral-cloud)
- `configs/providers/providers.yaml`: 5 providers configured (3 Ollama + 1 local OCR + 1 Mistral cloud)

### Fixed

- Port conflict on 8080 from stale `core.exe` process
- Google Drive File Stream locking `target/debug/core.exe`

## [7.0.0] — 2026-03-09

### Added

- Advanced monorepo structure with 8 Rust workspace crates
- Rust crate scaffolding: compiler, memory, router, metrics, cache, adapters, fingerprint
- Vue 3 + Vite dark dashboard with Pinia state and Vue Router
- Branding assets (`brand/`) and dark design system tokens
- GitHub Actions CI and release scaffold workflows
- Community files: CONTRIBUTING, SECURITY, GOVERNANCE, CODE_OF_CONDUCT
- Benchmark fixtures: token-reduction and request-latency
- Windows (`bootstrap-win.ps1`) and Unix (`bootstrap.sh`) bootstrap scripts
- Docker multi-stage build, Kubernetes deployment, and Helm chart
- Provider, routing, and policy YAML configuration
- Documentation: architecture, compiler, memory lensing, flow visualizer, benchmarking, branding

### Changed

- Roadmap coherence across V1–V6.7 historical iterations
- Stronger product positioning around context compilation and memory lensing
- Clearer separation between gateway logic and visualization layer

> **Note:** This release is an advanced scaffold optimized for implementation speed,
> not a finished production runtime.
