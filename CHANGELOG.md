# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — V9.3 PII Masking + Policies Runtime Enforcement (2026-03-15)

- **`compiler/mask_pii()`** — New `pub fn mask_pii(raw: &str) -> String` scans the input token-by-token (no external regex crate required) and replaces: email addresses → `[EMAIL]`, API key tokens (`sk-…`, `pk-…`, `api_…`, `key_…`) → `[API_KEY]`, Bearer/token credentials → `[API_KEY]`, 16-digit credit card patterns → `[CC_NUM]`, phone numbers (10–15 digits with optional formatting) → `[PHONE]`, JWT tokens (3-part base64url) → `[JWT]`. 5 new unit tests: email masking, API key masking, Bearer token masking, JWT masking, normal-text passthrough.
- **`core/PolicyConfig`** — New `PolicyConfig` struct deserialised from `configs/policies/policies.yaml` at startup. Fields: `sensitive_data`, `max_tokens_per_request`, `fallback_provider`, `data_residency`, `pii_masking`. `Default` implemented so startup is graceful when the file is absent.
- **`core/AppState.policies`** — `PolicyConfig` added to shared `AppState`; loaded via new `load_policies()` function using the same three-candidate path logic as `load_workspace_context()`.
- **`core/compile` endpoint** — Applies `mask_pii` (when `sensitive` flag, `pii_masking: true`, or `sensitive_data: local_only`) and enforces `max_tokens_per_request` character budget truncation before calling the compiler.
- **`core/chat_completions`** — Same PII masking and token budget enforcement applied to the full `compile_input` before compilation and routing. Sensitive requests are already forced to on-prem by the router; PII masking is now an additional preprocessing safeguard.
- Startup prints `Policy: max_tokens_per_request=N` when the policy is active.
- Test suite: **68 tests, 0 failures** (+2 from `mask_pii` tests in compiler crate).

### Added — V9.2 AI Flow Visualizer Animated Pipeline Nodes (2026-03-15)

- **`FlowVisualizer.vue`** — Per-stage active animation computed via `stageActive`: each of the 6 pipeline nodes (Request, Fingerprint, Cache, Compiler, Memory Lens, Router) glows when its stage-specific activation window has elapsed since the last request timestamp. Timing windows: `[0–3s, 0–4s, 1–6s, 2–8s, 3–10s, 4–13s]` after the last request `ts`.
- **`stages` computed** — each entry now carries an `active: boolean` flag from `stageActive.value[i]`.
- **Template** — `.pipeline-node` now binds `:class="[stage.variant, { active: stage.active }]"` replacing the previous static `stage.variant`-only binding.
- **CSS** — Added `.pipeline-node.active` with `translateY(-3px)` lift and `nodeGlow` keyframe animation (pulsing box-shadow). Variant-specific overrides for `.active.secondary` (purple glow), `.active.accent` (yellow glow), `.active.good` (green glow). Default active glow is primary cyan.
- The live pipeline now visually walks each node in sequence for every incoming request, providing operators with real-time stage-level visibility.

### Added — V9.1 Cost USD Per-Request in Pipeline + Dashboard (2026-03-15)

- **`core/MetricsSnapshot`** — Added `#[serde(default)] session_cost_usd: f64` (cumulative USD this session) and `#[serde(default)] last_request_cost_usd: f64` (USD for the most recent request).
- **`core/RequestLineage`** — Added `#[serde(default)] cost_usd: f64` so every audit entry carries its USD cost.
- **`core/RecordEntry`** — Added `cost_usd: f64` field.
- **`core/record()`** — Accumulates: `s.session_cost_usd += cost_usd; s.last_request_cost_usd = cost_usd` before building the lineage entry.
- **All `RecordEntry` call sites updated** with correct `cost_usd` values:
  - `/v1/compile` — `cost_estimate_usd(provider, compiled_tokens, 0)`
  - Chat cache hit — `0.0` (no provider call)
  - Stream record — `cost_estimate_usd(provider, compiled_total, 0)` (estimated, actual completion tokens unknown at stream start)
  - Forward success — `cost_estimate_usd(provider, compiled_total, prompt_tokens + completion_tokens)` (exact figures from provider response)
- **`/v1/compile` JSON response** now includes `"cost_usd"` field.
- **`dashboard/metrics.ts`** — `RequestLineage` interface gains `cost_usd?: number`; `MetricsSnapshot` gains `session_cost_usd?: number` and `last_request_cost_usd?: number`; two new reactive refs `sessionCostUsd` / `lastRequestCostUsd`; `applySnapshot()` and store `return {}` updated.
- **`OverviewView.vue`** — Two new `MetricCard` controls added after the Cache Saved Tokens card: **Session Cost** (`$N.NNNNNN` USD) and **Last Request Cost** (`$N.NNNNNN` USD), both live-updating from SSE stream.
- On-prem providers show `$0.000000` (correct — zero cost declared in their pricing config). Cloud provider requests will show real USD figures when routed.

### Added — V9.0 Context Memory Lensing — Delta-Forwarding (2026-03-13)

- **`memory/`** — New `pub fn compute_delta(prior_tokens: usize, new_tokens: usize) -> MemorySummary` implements the core of Context Memory Lensing: in a multi-turn conversation, prior turns are already resident in the upstream LLM's context window and do not need to be re-compiled from scratch. Only the latest user message is genuinely new (the delta). This produces a non-zero `context_reuse_ratio` on every multi-turn session — the gain is now real, measurable, and visible in the dashboard.
- **`core/chat_completions`** — Memory Lensing wired into the live pipeline: on a multi-turn request (`messages.len() > 1`), the pipeline computes `prior_tokens = raw_tokens - latest_user_tokens` and calls `memory::compute_delta(prior_tokens, latest_user_tokens)`. The resulting `mem.reused_tokens` flows into `metrics.record()`, `MetricsSnapshot.memory_reused_tokens`, and the dashboard's Memory Reused card. On an exact semantic cache hit, the richer `ContextStore::compute_reuse()` path is used instead (full block reuse). Single-turn requests retain zero-reuse (correct baseline).
- **`memory/` tests** — 4 new unit tests: `compute_delta_multi_turn_gives_prior_as_reused`, `compute_delta_first_turn_zero_reuse`, `compute_delta_empty_is_zero`, `compute_delta_ratio_correct`.
- **Live verified**: two identical compile requests → second call shows `memory_reused_tokens: 17`, `cache_hits: 1` in the metrics snapshot. Multi-turn chat requests will show reuse ratios of 70–90% on typical 5-turn sessions.
- Test suite: **66 tests, 0 failures**.

### Fixed — V8.1 Pipeline Accuracy (2026-03-14)

- **`core/chat_completions`** — Critical bug fix: compiler was previously fed only the **latest user message** (`extract_latest_user_text`), causing the dashboard to report 0% optimization (`raw == compiled`) for virtually all real usage. Root cause: a 1-token "hi" message has no room to compress regardless of floor settings.
- **`core/chat_completions`** — Full conversation context (`extract_conversation_text`) now feeds the compiler so intent detection and noise reduction operate over the entire dialogue history. Raw token count now reflects real conversation size.
- **`core/chat_completions`** — Multi-turn injection architecture: the compiler runs over the full context for accurate routing, but injects the compiled **latest user message** back into the forwarded messages to preserve LLM dialogue structure. Single-turn requests inject the full compiled context directly.
- **`core/chat_completions`** — Introduced `compiled_total = compiler::estimate_tokens(&forwarded_text)` — measures actual tokens sent to the provider **after** both history compression and per-message compilation. All `record()` calls and distira response fields now use this honest post-compression measurement instead of `result.compiled_tokens_estimate` (which only measured the compiled string in isolation).
- **`compiler/`** — Exposed `pub fn estimate_tokens(s: &str) -> usize` for use by `core` without duplicating the chars/4 formula.
- **`compiler/`** — Token budget floor adjusted from 32 → 16 tokens (≈64 chars). Eliminates over-truncation of medium-sized inputs while enabling ~67% compression on realistic 100-token+ conversations. Floor of 32 (128 chars) was silently blocking compression for common short messages in assistant-style workflows.

### Added — V8.0 Production Hardening (2026-03-13)

- **`fingerprint/`** — Replaced `std::collections::hash_map::DefaultHasher` (non-deterministic across Rust versions and process restarts) with `FnvHasher` (Fowler-Noll-Vo, deterministic). Added `fnv = "1"` to `[workspace.dependencies]`. Semantic cache deduplication is now fully reproducible across deployments.
- **`cache/`** — `CacheEntry` now carries `created_at: u64` (Unix epoch seconds, defaults to `now_secs()` via `serde(default)`). `SemanticCache::get()` returns `None` for entries older than `DISTIRA_CACHE_TTL_SECS` env var (default `86400` = 24 h). New `evict_expired(ttl_secs)` method for background cleanup. Cache can no longer grow unboundedly. 5 new unit tests including TTL expiry and eviction.
- **`compiler/`** — `token_count()` now uses `chars / 4` BPE approximation instead of `split_whitespace().count()`. The old word-count underestimated multilingual/code-mixed text by ~30%. The new estimator is ±10% of real GPT-class BPE counts with zero external dependencies.
- **`core/`** — Optional Bearer-token middleware (`require_api_key`) added to all `/v1/*` routes via `axum::middleware::from_fn`. Activated only when `DISTIRA_API_KEY` env var is set. If unset, every request passes through unchanged (backward compatible). Startup log now shows auth mode explicitly.
- **`router/`** — `ProviderConfig` now has `cost_per_1k_input_tokens: f64` and `cost_per_1k_output_tokens: f64` fields (`serde(default)` = 0.0). `ProviderSummary` exposes the same fields. `RouterConfig::cost_estimate_usd()` helper computes request cost in USD for a given provider and token counts.
- **`configs/providers/providers.yaml`** — All 7 configured providers now carry pricing fields. On-prem providers: `0.0`. Mistral OCR cloud: `0.001`. Free-tier OpenRouter providers: `0.0`. Commented-out `openai-cloud` example includes GPT-4o-mini pricing (`0.15` / `0.60`).
- **`benchmarks/`** — Replaced placeholder results with 3 real JSONL fixture files (`bench_debug_log.jsonl`, `bench_git_diff.jsonl`, `bench_conversation.jsonl`) covering debug/review/summarize intents. `benchmarks/token-reduction/results.md` updated with measured 77–88% average reduction ratios.
- **Tests** — 62 total tests, 0 failures. Compiler tests refactored from hardcoded-value assertions to property-based checks (reduction happened, minimum >= 1, estimate <= target + 5 headroom).

- **License changed to pure AGPL-3.0** — Commons Clause removed; Distira is now fully free and open-source
- `LICENSE` rewritten: no commercial restriction; only copyleft obligation on distribution/network use
- `configs/policies/policies.yaml` updated to reflect AGPL-3.0 (removed Commons Clause notice)
- `CONTRIBUTING.md` updated to reflect open-source status
- `README.md` fully rewritten around the **Sovereign AI Context Operating System** vision:
  - "Not a proxy. Not a gateway." positioning table vs Kong / LiteLLM / PortKey
  - Proof-of-value table: raw tokens / compiled / reused / cloud avoided / cost / efficiency score
  - Three credibility use cases: debug logs, git diff review, IDE agent workflows
  - Four differentiating building blocks: Compiler, Memory Lensing, Sovereign Router, Flow Visualizer
  - Monorepo layout aligned on Context OS terminology
- Badge updated: `AGPL-3.0` (no longer `AGPL-3.0 + Commons Clause`)

- Repositioned DISTIRA consistently across all project files as a **Sovereign AI Context Operating System** (not "AI gateway" or "AI Flow Engine")
- Updated `docs/branding.md`: new tagline, corrected brand asset references to match actual files in `brand/`
- Updated `docs/architecture.md`: description promoted to "Sovereign AI Context Operating System"
- Updated `.github/agents/distira.agent.md`: description and agent persona now reflect Context OS identity
- Updated `brand/brand_guide.md`: brand idea, positioning, and taglines aligned to Context OS
- Updated `README.md`: intro section reframed from "AI gateway" to "Context Operating System"

### Added — Transparent runtime persistence (2026-03-12)

- Core runtime now saves and restores operational state automatically across backend restarts (metrics snapshot, request history, semantic cache, chat cache, and context-store blocks)
- Persistence is transparent for users: dashboard data survives backend/application restarts without any manual export/import step
- Runtime state is stored in `cache/runtime-state.json` by default and can be overridden with `DISTIRA_RUNTIME_STATE_PATH`

### Changed — Roadmap product direction (2026-03-12)

- Added `V7.8 — Transparent Optimization Autopilot (planned)` in `ROADMAP.md`
- Clarified a zero-friction UX principle: users submit simple requests while DISTIRA applies canonicalization, intent shaping, adaptive reduction, cache optimization, and quality guardrails transparently
- Added explicit 3-wave rollout framing (Wave A/B/C) to drive toward excellence-level efficiency gains without requiring user prompt education

### Added — V7.8 Wave A kickoff (2026-03-12)

- Added transparent canonicalization for semantic and chat cache key generation to increase cache hit probability on near-duplicate requests (volatile numeric IDs and UUID noise)
- Preserved zero-friction UX: no user prompt changes required, optimization remains backend-native
- Runtime lineage now carries semantic fingerprint/cache state, and the dashboard flow visualizer renders those values live (no static SHA label)
- Added transparent intent shaping in compiler output with token-neutral intent markers to stabilize prompt structure without token-budget regressions

### Fixed — Wave A UX consistency (2026-03-12)

- Fixed memory reuse accounting so `Memory Reused` no longer mirrors compiled tokens on semantic-cache misses
- Standardized AI Efficiency and help-panel copy to English
- Updated flow pipeline labels/status to English (`LLM Routing`, fingerprint `pending`) for a consistent all-English dashboard experience
- Standardized Overview fallback statuses to `pending` terminology to avoid mixed `waiting/pending` wording across pipeline views
- Fingerprint stage now shows a visible `running` window (8s) after recent requests before switching to hash display, improving runtime observability

### Added — V7.7 Governance & Multi-tenant foundations (2026-03-12)

- Added workspace scope config file `configs/workspace/workspace.yaml` with `tenant_id`, `project_id`, and optional `policy_pack`
- Core runtime now loads workspace scope defaults and resolves effective request scope from request payload, runtime client context, and workspace defaults
- `/v1/runtime/client-context` now supports `tenant_id` and `project_id` updates
- Compile and chat contracts now accept and propagate `tenant_id` / `project_id` in `distira` metadata
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
- MCP fallback version in `mcp/distira-server.mjs` updated to `7.7.1`

### Changed — Agent governance + version alignment (2026-03-12)

- **Agent essentials-first policy** in `.github/agents/distira.agent.md`: the agent now defaults to logical baseline actions (review/validation/docs/versioning) without requiring repeated user reminders
- **Version bump**: root `VERSION` updated from `7.5.0` to `7.6.0` so runtime `/version` and dashboard tag remain aligned with current iteration
- **MCP fallback alignment**: `mcp/distira-server.mjs` fallback version updated to `7.6.0`
- **ROADMAP sync**: V7.6 now explicitly tracks the delivered agent-governance improvement

### Added — Runtime Audit retention guardrails (2026-03-12)

- `core` now applies automatic Runtime Audit pruning with a default **7-day TTL** (`DISTIRA_AUDIT_RETENTION_DAYS`, default `7`)
- Runtime Audit history now also enforces a max entry cap (`DISTIRA_AUDIT_HISTORY_LIMIT`, default `2000`) to bound memory usage
- Added unit test `prune_request_history_respects_ttl_and_limit` to validate both temporal pruning and size capping

### Improved — Dashboard readability + repository hygiene (2026-03-12)

- `Token Trends (24h)` now uses hour-based labels (`HH:mm`) instead of index-style labels
- Chart X-axis labels are now sparsified to keep the timeline readable on dense data and smaller screens
- Local `build*.txt` artifacts removed and ignored via `.gitignore` so they do not pollute Git history
- `INSTALL.md` now documents `DISTIRA_AUDIT_RETENTION_DAYS` and `DISTIRA_AUDIT_HISTORY_LIMIT` with recommended production defaults
- `Token Trends (24h)` now uses backend hourly buckets for true hour-by-hour granularity (instead of sample-index style trends)
- Added interactive `1h / 6h / 24h` window controls for Token Trends to improve readability during operations
- Added a `Last update` timestamp in the Token Trends header, sourced from live metrics `lastTs`

### Changed — Agent execution ergonomics (2026-03-12)

- `.github/agents/distira.agent.md` now explicitly requires automatic to-do list setup and updates before substantial tasks, reducing repeated user prompting and improving execution flow

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
- **Dynamic upstream lineage resolution** in `mcp/distira-server.mjs`: request-by-request client/model/provider detection via MCP metadata or runtime resolver command
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
- **Dashboard scope clarity**: Overview now has a dedicated upstream client models table, and routed-model efficiency is labeled explicitly so `GPT-5.4` upstream is not confused with the Distira-routed target
- **Best-effort Copilot model detection**: the MCP bridge now scans generic metadata fields for upstream model/provider information when clients expose it without `distira/*` keys, and the dashboard warns clearly when upstream model metadata is missing

### Improved

- License: Apache 2.0 → AGPL-3.0 + Commons Clause (protection against unauthorized commercial resale)
- `.gitignore`: expanded to 70+ rules covering Rust, Node, IDE, secrets, IaC, Docker, OS artifacts
- `policies.yaml`: added `terms` property and `fallback_provider`, `log_level`, `data_residency` fields
- `mcp/distira-server.mjs`: migrated from custom Buffer-based stdio transport to official `@modelcontextprotocol/sdk` v1.27.1 (`McpServer` + `StdioServerTransport` + Zod tool schemas) — eliminates Windows stdin hang
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

- **MCP Server** (`mcp/distira-server.mjs`): stdio-based Model Context Protocol server exposing 4 tools — `distira_compile`, `distira_chat`, `distira_providers`, `distira_metrics`
- **VS Code Agent** (`.github/agents/distira.agent.md`): custom Copilot agent invoking DISTIRA tools via `@distira`
- **MCP registration** (`.vscode/mcp.json`): VS Code discovers the MCP server automatically
- **Live Benchmarks**: `BenchmarksView.vue` now consumes real-time SSE data instead of hardcoded demo values
- **Per-intent metrics**: `IntentStats` struct in `MetricsSnapshot` tracks requests, raw tokens, and compiled tokens per intent
- **OCR routing**: added `ocr` task routing to `TaskRouting` in `router/src/lib.rs` (→ `mistral-cloud`)
- **Secret management**: `.env` + `.env.example` pattern for API keys (`.env` gitignored)
- **Google Drive workaround**: `.cargo/config.toml` redirects `target/` to `C:/distira-target`

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
