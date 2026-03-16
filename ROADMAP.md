# DISTIRA Roadmap

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
| V6.3 | DISTIRA rebrand |
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

- **MCP Server** (`mcp/distira-server.mjs`) — stdio-based, 4 tools: compile, chat, providers, metrics
- **VS Code Agent** (`.github/agents/distira.agent.md`) — `@distira` in Copilot Chat
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
- Live metrics snapshot now distinguishes upstream client model vs DISTIRA-routed model
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
- Agent governance reinforced in `.github/agents/distira.agent.md` with an essentials-first workflow so users no longer need to repeatedly request baseline actions (review/validation/docs/version updates)
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

### V7.8 — Transparent Optimization Autopilot (planned)

**Status:** In progress (Wave A started).

- Product principle: users should only send a simple request; DISTIRA handles optimization automatically without requiring prompt engineering education
- Objective: make `~50%` token/cost gains reachable on repetitive workflows while preserving response quality
- Automatic canonicalization pipeline before fingerprinting (normalize volatile values, reduce formatting variance) to increase cacheability
- Intent-native prompt shaping templates applied transparently (`debug`, `review`, `summarize`, `general`, `codegen`) to stabilize request structure
- Adaptive context budgeting per intent and confidence level, with automatic fallback to safer (less aggressive) reduction when quality risk increases
- Multi-level semantic cache strategy (exact, canonical, semantic-nearest) to convert near-duplicate user requests into reusable compiled contexts
- Delta-first forwarding for long conversations (reuse stable blocks, send only meaningful changes)
- Built-in quality guardrails (sampled dual-path validation and auto-tuning) so optimization remains invisible to end users
- Excellence KPIs surfaced for operators (cacheability, avoidable tokens, quality drift) without exposing complexity to end users

Implementation waves:

- Wave A (fast ROI): canonicalization + intent templates + operational KPIs
- Wave B (scale gains): multi-level cache + stronger delta-forwarding
- Wave C (autonomous quality): adaptive optimizer with automatic aggressiveness tuning

Wave A progress:

- Canonicalization is now applied to semantic and chat cache key construction to reduce misses caused by volatile IDs and noisy formatting without changing user behavior
- Flow visualizer now displays live semantic fingerprint/cache lineage signals instead of static fingerprint labels, improving operator trust in real-time automation
- Transparent intent shaping is now active in compiler outputs via token-neutral per-intent markers (`debug`, `review`, `summarize`, `general`, `codegen`, `ocr`) to stabilize downstream behavior without increasing user complexity
- Memory reuse telemetry now differentiates misses vs reuse correctly (no duplicate-looking compiled/memory values on misses), and Wave A UI labels are aligned to all-English copy
- Runtime state is now transparently persisted and restored across backend restarts (metrics snapshot, runtime audit history, semantic/chat caches, context-store blocks) via `cache/runtime-state.json`

## Future iterations

- Role-based access control (RBAC)
- Audit trail and compliance logging
- Multi-tenant org / team / project model
- Policy packs (GDPR, SOC2, HIPAA templates)

### V7.9 — Open-Source Release + Context OS Identity (2026-03-13)

**Status:** Delivered.

- **Open-source relicensing** — Commons Clause removed; project is now pure AGPL-3.0 (free for any use, copyleft on distribution)
- **Product identity alignment** — "Sovereign AI Context OS" terminology adopted consistently across all project files:
  - `README.md`, `docs/branding.md`, `docs/architecture.md`, `brand/brand_guide.md`, `.github/agents/distira.agent.md`, `configs/policies/policies.yaml`, `CONTRIBUTING.md`, `INSTALL.md`
- **README rewritten** around the Context OS vision:
  - "Not a proxy. Not a gateway." differentiation table vs Kong AI / LiteLLM / PortKey
  - Proof-of-value table with concrete numbers (12 000 → 2 400 tokens, €0.036 → €0.007, 81% efficiency score)
  - Three credibility use cases: debug logs, git diff review, IDE agent workflows
  - Four differentiating building blocks: Context Budget Compiler, Context Memory Lensing, Hybrid Sovereign Router, AI Flow Visualizer
- **Author section added** in README with GitHub and LinkedIn links and open-source call to action
- **Brand assets corrected** in `docs/branding.md` — all references now point to existing files in `brand/`
- **Dashboard branding** — replaced placeholder "K" text icon with the real `distira_app_icon.svg` in sidebar and mobile header
- **Favicon updated** — `public/favicon.svg` now uses the real `distira_symbol.svg` (compression frame); `favicon.ico` and `favicon-32.png` copied from `brand/`; `index.html` declares all three with correct priority order

### V8.0 — Production Hardening (2026-03-13)

**Status:** Delivered.

- **Stable deterministic hashing** — `DefaultHasher` (non-deterministic across Rust versions and restarts) replaced by `FnvHasher` (Fowler-Noll-Vo, stable across all platforms and versions) in `fingerprint/` crate; semantic cache keys are now fully reproducible across deployments
- **SemanticCache TTL eviction** — `CacheEntry` now carries a `created_at` epoch timestamp; `get()` returns `None` for entries older than `DISTIRA_CACHE_TTL_SECS` (default 24h); `evict_expired(ttl)` available for background cleanup; cache no longer grows unboundedly
- **Accurate token estimation** — `token_count()` in `compiler/` replaced `split_whitespace().count()` (30% undercount for mixed prose/code) with chars-÷-4 BPE approximation (±10% for multilingual text); efficiency metrics in metrics snapshot and dashboard are now credible
- **Optional API key auth** — Bearer-token middleware on all `/v1/*` routes; if `DISTIRA_API_KEY` env var is set, every `/v1` request requires `Authorization: Bearer <key>` header; if not set, behavior is unchanged (open). `/healthz` and `/version` remain unauthenticated
- **Provider pricing table** — `cost_per_1k_input_tokens` and `cost_per_1k_output_tokens` added to all 7 providers in `configs/providers/providers.yaml`; on-prem providers are 0.0; `ProviderConfig` and `ProviderSummary` structs in `router/` now carry the fields; `cost_estimate_usd()` helper method exposed on `RouterConfig`
- **Real benchmark fixtures** — Replaced placeholder benchmark data with 3 real JSONL fixture sets in `benchmarks/token-reduction/fixtures/`: `bench_debug_log.jsonl`, `bench_git_diff.jsonl`, `bench_conversation.jsonl`; `benchmarks/token-reduction/results.md` updated with measured 77–88% reduction ratios per intent
- Test suite expanded: 62 tests, 0 failures (5 new cache TTL tests, compiler tests updated to property-based assertions)

### V8.1 — Pipeline Accuracy Fix (2026-03-13)

**Status:** Delivered.

- **Full-context compilation in chat_completions** — Compiler now receives the entire conversation history (`extract_conversation_text`) instead of only the latest user message. Raw token count now reflects the true dialogue size, and intent detection has full context to work with.
- **Honest compiled token measurement** — Introduced `compiled_total` (actual forwarded token count post-compression) to replace `compiled_tokens_estimate` in all pipeline metrics. Dashboard compression ratios are now accurate.
- **Multi-turn injection architecture** — Compiler runs on full context for routing; latest user message is compiled separately and injected back to preserve LLM dialogue structure. Single-turn requests inject the full compiled context directly.
- **Token budget floor: 32 → 16** — Reduces over-truncation on medium inputs while enabling ~67% compression on 100+ token conversations. Floor of 32 was silently blocking all compression below 128 chars.
- **`pub fn estimate_tokens`** exposed from `compiler/` crate for honest cross-crate token measurement.
- 62 tests, 0 failures.

### V9.0 — Context Memory Lensing — Delta-Forwarding (2026-03-13)

**Status:** Delivered.

- **`memory::compute_delta`** — new public function implementing the delta-forwarding principle: prior conversation turns are already in the LLM's context window (reused), only the latest user message is the delta (new). Produces non-zero `context_reuse_ratio` on every multi-turn session.
- **`core/chat_completions`** — Memory Lensing wired: `prior_tokens = raw_tokens - latest_user_tokens`; `memory::compute_delta(prior_tokens, latest_user_tokens)` called for all multi-turn non-cache-hit requests. Semantic cache hits use `ContextStore::compute_reuse` (exact block match). Single-turn: zero reuse (correct).
- `memory_reused_tokens` now shows real non-zero values in dashboard Memory Reused card and in the `history_reused` SSE stream.
- Live verified: 17 tokens reused on first cache hit; multi-turn sessions projected at 70–90% context_reuse_ratio on typical 5-turn IDE workflows.
- 4 new unit tests for `compute_delta`. Test suite: 66 tests, 0 failures.

### V9.1 — Cost USD Per-Request in Pipeline + Dashboard

**Status:** Delivered.

- Wire `cost_estimate_usd()` from `RouterConfig` into `MetricsSnapshot` and `RecordEntry`
- Add per-request cost field to distira response blocks
- Dashboard: Session Cost and Last Request Cost cards in Overview (live USD from SSE stream)

### V9.2 — AI Flow Visualizer — Animated Pipeline Nodes

**Status:** Delivered.

- Per-stage active animation computed in FlowVisualizer: each of 6 nodes glows in sequence after each request using timing offsets from `lastRequest.ts`
- Variant-specific glow colors (cyan/purple/yellow/green) per node type
- Rendered from SSE stream, updated per request

### V9.3 — Policies + PII Masking Runtime Enforcement

**Status:** Delivered.

- `configs/policies/policies.yaml` loaded at startup into `PolicyConfig` and added to `AppState`
- `compiler::mask_pii()` masks emails, API keys, Bearer tokens, JWTs, credit cards, phone numbers before compilation
- `max_tokens_per_request` enforced as character budget truncation in both `/v1/compile` and `/v1/chat/completions`
- PII masking activated when `sensitive=true`, `pii_masking: true`, or `sensitive_data: local_only` in policies
- Test suite: 68 tests, 0 failures

### V9.4 — Distira Universal Token Estimator

**Status:** Delivered.

- New `tokenizer/` Rust crate: zero-dependency BPE-style token counting. Handles CJK (1 token/char), digits (1/digit), word-length bucketing, punctuation counting, and non-ASCII grouping. Replaces `chars/4` across the entire pipeline.
- `ModelFamily` enum + `family_for_provider()` for model-aware calibration: GPT-4 (cl100k_base \u00d70.95), Llama3/Mistral (baseline), Qwen (Llama3 alias).
- `compiler/` fully migrated to `tokenizer::count()`; `intent_marker()` extracted so `compile_context()` correctly reserves marker overhead before truncation.
- `core/chat_completions` uses `tokenizer::count_for(text, family_for_provider(provider))` for model-calibrated `compiled_total` in all metrics.
- Test suite: 98 tests, 0 failures (+30 tokenizer tests).
- Accuracy: ±4% prose / ±7% code vs ±18-22% for chars/4.

### V9.5 — Encoding & Decoding Optimization

**Status:** Delivered.

- `tokenizer::encode(text)` — lossless input normalization pipeline: invisible Unicode removal (BOM, ZWSP, soft-hyphen, ZWJ, directional marks), typographic punctuation → ASCII (curly quotes, guillemets, em/en-dash, horizontal ellipsis), excess blank lines collapsed (3+ → 2), trailing whitespace stripped, internal whitespace runs collapsed (leading indentation preserved). Idempotent.
- `tokenizer::decode(text)` — BPE reconstruction artifact cleanup: CRLF normalization, stray space before punctuation removed (SentencePiece `▁` artifact from Llama/Mistral), double-space collapse, CJK inter-character spaces removed.
- `compiler::compile_context()` calls `tokenizer::encode()` as the very first step; the normalized form flows through all token counting, intent detection, and context shaping.
- `core/chat_completions` applies `tokenizer::decode_for(content, token_family)` on all non-streaming provider responses and on accumulated streaming content before cache insertion.
- Test suite: 131 tests, 0 failures (+33 encode/decode unit tests).

### V9.9 — LM Studio / OpenWebUI + GLM + Gemini 2.5 + Claude 4.x provider coverage

**Status:** Delivered.

- **LM Studio** (`http://localhost:1234/v1`) and **OpenWebUI** (`http://localhost:3000/api`) are now documented as first-class on-prem frontends. Both expose an OpenAI-compatible API — no adapter code changes required. Ready-to-use commented provider stanzas added to `providers.yaml`.
- **`ModelFamily::Glm`**: GLM-4, GLM-Z1, ChatGLM (ZhipuAI). On-prem via Ollama (`glm4:9b`) or cloud via DashScope/ZhipuAI API.
- **Full provider coverage added (commented)**:
  - Anthropic: Claude Sonnet 4.5, 4.6, Opus 4.5, Claude 3.7 Sonnet
  - Google: Gemini 2.0 Flash, Gemini 2.5 Pro, Gemini 2.5 Flash
  - ZhipuAI: GLM-4, GLM-Z1 Flash
  - Alibaba: Qwen 3 235B (MoE, DashScope cloud)
  - OpenAI: GPT-4o, GPT-5
- **Complete `ModelFamily` matrix** at V9.9: Universal, Llama3, Qwen, Gpt4, Gpt4o, Claude, Gemini, Glm.
- Test suite: 159 tests, 0 failures.

### V9.8 — Full model compatibility + translate intent

**Status:** Delivered.

- **Bug fix — codegen routing**: `TaskRouting` deserializer was silently dropping the `codegen` YAML key. Fixed.
- **`translate` intent**: Full pipeline — detection (13 language/keyword triggers in EN+FR+DE+ES+JA+ZH), `[k:translate]|` marker, routed to Mistral Small 3.1 24B (multilingual leader).
- **codegen keyword coverage**: TypeScript, JavaScript, Go, Kotlin, Swift, `codex`, `create a class`, `complete this function`, FR: `crée une fonction`, `génère du code`.
- **`ollama-llama3.3`**: Llama 3.3 70B provider entry (on-prem, `ollama pull llama3.3`).
- **Model compatibility matrix** at V9.8:
  - Llama 3 / 3.1 / 3.3 ─ `Llama3` family, on-prem via Ollama
  - Mistral 7B / Small 3.1 ─ `Llama3` family, on-prem + cloud
  - Qwen 2.5 Coder ─ `Qwen` family, on-prem
  - GPT-4 / GPT-3.5 ─ `Gpt4` (cl100k_base exact), cloud optional
  - GPT-4o / o1 / o3 / o4 / **GPT-5** ─ `Gpt4o` (o200k_base exact), cloud optional
  - **Codex** ─ deprecated by OpenAI; `codegen` intent routes to Qwen locally (best available)
  - **Claude** 3/3.5/3.7/4 ─ `Claude` family (cl100k proxy), cloud optional
  - **Gemini** 1.5/2.0/Flash/Pro ─ `Gemini` family (calibrated heuristic), cloud optional
  - DeepSeek-R1 ─ `Llama3` family, cloud via OpenRouter
- Test suite: 155 tests, 0 failures.

### V9.7 — Claude & Gemini tokenizer support

**Status:** Delivered.

- **`ModelFamily::Claude`** — Anthropic Claude 3/3.5/3.7/4. cl100k_base used as an accurate proxy (±3%) via the existing `exact-gpt4` pipeline. Falls back to `(raw * 19/20)` heuristic when feature is off.
- **`ModelFamily::Gemini`** — Google Gemini 1.5/2.0/Flash/Pro. Calibrated `(raw * 19/20)` heuristic (SentencePiece 256k; no embedded Rust vocabulary available).
- **`family_for_provider()`** updated: `claude`/`anthropic` → `Claude`; `gemini`/`google`/`palm` → `Gemini`. Both checks run before OpenAI/GPT checks to avoid false matches.
- **`configs/providers/providers.yaml`** — Added commented-out cloud provider entries for Anthropic Claude and Google Gemini (OpenAI-compatible endpoints). Activate by setting `ANTHROPIC_API_KEY` / `GOOGLE_API_KEY` and uncommenting.
- Test suite: 149 tests, 0 failures.
- DISTIRA now covers all major AI actors: **Llama3/Mistral/Gemma/DeepSeek** (on-prem), **Qwen** (on-prem), **GPT-4/GPT-4o/GPT-5** (OpenAI), **Claude** (Anthropic), **Gemini** (Google).

### V9.6 — Exact GPT-4 Token Counting via tiktoken-rs + GPT-5 / o200k_base support

**Status:** Delivered.

- Optional Cargo feature `exact-gpt4` in `tokenizer/` crate: replaces the GPT-4 heuristic with exact cl100k_base BPE counting using [`tiktoken-rs`](https://crates.io/crates/tiktoken-rs). Vocabulary is embedded in the binary — no external files required.
- Feature propagated to `core/` and enabled by default (`default = ["exact-gpt4"]`), so `cargo build` yields exact GPT-4 token counts out of the box.
- **`ModelFamily::Gpt4o`** variant added for GPT-4o, o1, o3, o4, and **GPT-5**: routes to `o200k_base` (200k-vocabulary BPE). `family_for_provider()` correctly dispatches all `gpt-4o*`, `gpt-5*`, `o1*`, `o3*`, `o4*` model keys.
- All other families (Llama3, Mistral, Qwen, Universal) keep the optimised heuristic (zero extra dependencies when `exact-gpt4` is disabled).
- Match arms in `count_for()` gated with `#[cfg(feature)]` — clean compilation in both modes, no dead-code warnings.
- Test suite: 142 tests, 0 failures (+6 Gpt4o tests, +4 exact GPT-4 tests).

### V9.10 — Metrics Reset & E2E Integration Tests

**Status:** Delivered (2026-03-14).

- `DELETE /v1/metrics/reset` endpoint — resets all counters without restarting the server. Returns `204 No Content`.
- `MetricsCollector::reset()` zeroes the full `MetricsSnapshot` and `hour_buckets`; caches preserved.
- Dashboard "Reset metrics" button in `OverviewView.vue` header (fires DELETE, disables in-flight, styled with red accent).
- `theme.css` `.view-header` upgraded to flex layout so the button aligns to the right on all views.
- `scripts/test-e2e.ps1` — 32-assertion E2E suite covering 6 groups: health, intent routing (8 intents), token pipeline, metrics increment, metrics reset (204 + all zeroed), and post-reset re-increment.

### V9.10.1 — BPE-Aware Token Optimizer

**Status:** Delivered (2026-03-14).

- **`compiler/src/optimizer.rs`** (new) — 6 lossless BPE-friendly passes applied before semantic compilation:
  1. Whitespace normalization (tabs, multi-space, trailing, blank runs)
  2. Numeric separator removal (`1,000,000` → `1000000`)
  3. Verbose-phrase substitution (21 patterns: `in order to` → `to`, `utilize` → `use`, `please note that` → `note:`, …)
  4. Consecutive duplicate-line collapse (≥3 identical lines → `[×N]`)
  5. Standalone comment stripping (codegen/general; skipped for review/debug)
  6. JSON compaction (valid JSON input → compact re-serialisation, −30–50 % tokens)
- `CompileResult.optimizer_savings` — new field: tokens saved by the optimizer alone.
- `POST /v1/compile` response: `"optimizer_savings"` field exposed.
- 16 new unit tests · 181 total · 0 failures.
- Typical additional gain: **+10–30 %** on top of the existing semantic compiler.

### V9.11 — Provider Budget & Alerting

**Status:** Delivered (2026-03-14).

- Per-provider daily request budget (`max_requests_per_day`) configurable in `providers.yaml`.
- Automatic fallback to the next available provider when budget is exhausted.
- Budget warning at ≥80%, exhausted at ≥100%; SSE `alerts` array emitted in metrics stream.
- Dashboard alert banner displayed above metrics grid (orange warning / red exhausted).

### V9.12 — Distillation Ratios + BM25 Salience Scoring

**Status:** Delivered (2026-03-14).

- Per-intent distillation ratios: ocr/translate 100%, debug/review 50%, codegen 33%, general 25%, summarize 20%.
- BM25-inspired `salience_score()` line scorer replacing head/tail truncation for `general` and `summarize`.
- `reduce_by_salience()` selects top-2/3 lines by signal density, preserves original order.

### V9.13 — Memory Stability Decay + Intent-Scoped Injection

**Status:** Delivered (2026-03-14).

- `ContextBlock` gains `intent` field for scoped injection.
- `register()` decays all other blocks ×0.92 per call; evicts blocks below stability 0.10.
- `compute_reuse()` intent-scope guard: returns zero reuse when block intent ≠ request intent.
- 5 new unit tests covering decay, eviction, and intent mismatch.

### V9.14 — Compiler-driven Chat History Compression

**Status:** Delivered (2026-03-14).

- `compress_conversation_history()` now runs each older turn through `compiler::compile_context()`.
- Intent marker prefix stripped before embedding in summary block.
- Fallback to word-truncation when compiled output is empty.

### V9.15 — Quality-Tier Routing + Conciseness Injection

**Status:** Delivered (2026-03-14 — VERSION 9.15.0).

- `quality_tier: low|standard|high` added to all active providers in `providers.yaml`.
- Per-provider `fallback_chain` in `ProviderConfig` overrides global fallback sequence when non-empty.
- `concise_mode: true` in `routing.yaml` — DISTIRA injects a conciseness directive into every forwarded LLM request, reducing output token usage across all providers.
- `inject_conciseness_directive()` helper in `core/main.rs` — prepends brevity system message; merges with existing system message if present.
- Dashboard: quality tier badge (`high`/`standard`/`low`) column in Live AI Efficiency table.

### V9.15.1 — Response Conciseness — rationale

Requesting the LLM to be concise in plain language (no emojis, no markdown decorations, short direct answers) reduces output token count, lowers latency, and makes responses more accessible to non-technical users. Enabled by default via `concise_mode: true` in `routing.yaml` — fully reversible by setting to `false`.

### V9.16 — Latency-Aware Routing

**Status:** Delivered (2026-03-14 — VERSION 9.16.0).

- Rolling average response latency tracked per provider in `MetricsCollector` (`provider_latency: HashMap<String, (f64, u64)>` — sum_ms + count).
- `choose_provider_latency_aware()` on `RouterConfig`: collects all within-budget candidates, picks minimum by avg latency; unmeasured providers get `f64::MAX` placeholder; when all are unmeasured falls back to priority order; sensitive override bypasses latency entirely.
- `avg_latency_by_provider()` accessor on `MetricsCollector` — computes live average on request.
- `core/chat_completions` — `std::time::Instant` timing around both streaming (`forward_stream`) and non-streaming (`forward`) adapter calls; latency stored in `RecordEntry.latency_ms`.
- `ModelStats` struct extended with `avg_latency_ms`, `latency_sum_ms`, `latency_samples`; emitted in SSE metrics stream.
- `/v1/providers` enriched with per-provider `avg_latency_ms`.
- Dashboard: Latency column in Live AI Efficiency table with `.latency-badge` pill.
- 3 new router unit tests. Router suite: 8 → 11.

### V10 — Adaptive AI Optimization Network

**Status:** Delivered (2026-03-14 — VERSION 10.0.0).

- **`choose_provider_adaptive()`** — composite score = `latency × (1.0 + error_rate × 5.0)` per provider; replaces latency-only routing with a learning loop that penalises unreliable providers proportionally.
- **`MetricsCollector.provider_errors` / `provider_total`** — session-level per-provider error tracking; `record_error()` called in both streaming and non-streaming `Err` branches of `chat_completions`.
- **`error_rate_by_provider()`** — computes live error rate map from session totals; fed into `choose_provider_adaptive()` on every request.
- **`GET /v1/suggestions`** — adaptive optimization suggestions endpoint: flags providers with ≥5% error rate (warning) or ≥3 000 ms latency (info/warning); returns structured JSON `{ severity, code, provider, metric, value, message }`.
- **`GET /v1/providers`** — enriched with `error_rate` per provider alongside `avg_latency_ms`.
- **Dashboard** — Optimization Suggestions panel in OverviewView: fetches `/v1/suggestions` on mount and on manual Refresh; color-coded warning/info cards with provider · metric metadata.
- **Clippy `manual_split_once`** — fixed in `compress_conversation_history()`.
- Router suite: 11 → 14 tests (3 new adaptive tests). All 202+ workspace tests pass.

*Multi-tenant isolation, native VS Code extension dedicated build, and cluster mode are scoped for V11+.*

### V10.1 — Live Memory Reuse Estimation

**Status:** Delivered (2026-03-14).

- `ContextStore::estimate_coverage()` in `memory/src/lib.rs` — lexical word-set intersection (compiled context ∩ prior stable blocks of same intent); replaces hard-coded `reused=0` on cache miss.
- Compile handler: `estimate_coverage()` called on every cache miss so `memory_reused_tokens` grows proportionally to real session memory overlap.
- `MemoryView.vue` fully redesigned: user-friendly labels, Session Savings progress bar, per-intent topic groups with health bars, no hex block IDs.
- +4 memory unit tests. Memory crate: 13 → 17 tests. Total workspace: **206 tests**.

### V10.2 — Dynamic Audit & Security-Cost Transparency

**Status:** Delivered (2026-03-14 — VERSION 10.2.0).

- `RequestLineage` struct extended with `raw_tokens`, `compiled_tokens`, `tokens_saved` (all `#[serde(default)]`); written on every compile/chat request.
- `AuditView.vue` redesigned:
  - 4 KPI cards: session cloud cost, tokens saved, on-prem-forced request count, cloud cost avoided.
  - Table: Time / Scope / Client / Routed / Intent / Tokens (saved + raw→compiled) / Cost or Security badge.
  - **🔒 Secured badge** makes `sensitive=true` routing visible as a cost optimization, not just a security decision.
  - Cloud cost avoided estimator: counterfactual savings from on-prem routing using session average cost.
  - CSV export enriched with `raw_tokens`, `compiled_tokens`, `tokens_saved`.

### V10.3 — Session Cost Budget & Alerts

**Status:** Delivered (2026-03-14 — VERSION 10.3.0).

- `session_budget_usd` field in `configs/workspace/workspace.yaml` — configures a USD cap for the session. Set to `0` to disable.
- `WorkspaceContext` and `MetricsSnapshot` carry `session_budget_usd: f64` (`#[serde(default)]`).
- `budget_alerts()` extended with session-cost threshold logic: **Warning** fires when session cost ≥ 80 % of budget; **Exhausted** fires at 100 %. Both surface in the existing dashboard alerts banner.
- `build_full_snapshot()` injects `session_budget_usd` into every SSE and REST metrics snapshot.
- Dashboard store: `sessionBudgetUsd` reactive ref bound via `applySnapshot`.
- **OverviewView**: Session Cost Budget utilisation bar — shows current cost / budget with colour-coded fill (green/amber/red) and percentage. Hidden when budget is `0` (disabled)., `cost_usd`.

### V10.6 — Savings & Environmental Impact Dashboard + Audit Cleanup

**Status:** Delivered (VERSION 10.6.0).

- **Overview: Savings & Environmental Impact** — 4 tiles: estimated cost saved, energy avoided (kWh), tree-equivalent CO₂, animated 3D ice cube representing ice-melt litres avoided. Constants: $0.006/1K tokens, 0.0005 kWh/1K tokens, 0.4 kg CO₂/kWh, 22 kg CO₂/tree/year, 5 L ice/kg CO₂.
- **Runtime Audit cleaned** — pure security/routing audit: cost column and cost KPIs removed; 6-column table, sensitive badge in tokens column.
- **Upstream table filtered** — test entries (`t`, `test`, `unknown-model`) excluded.

### V10.5 — Always-On Optimization Suggestions & Brand Refresh

**Status:** Delivered (VERSION 10.5.0).

- **`get_suggestions` redesigned** — always returns 4 proactive informational suggestions (routing map, session efficiency, cache performance, concise mode) on every load; reactive alerts (error rate, latency) fire on top. Panel never shows empty on fresh server start.
- **`RouterConfig::task_routing_summary()`** — new public method returning sorted (intent, provider) pairs for the suggestions backend.
- **OverviewView auto-refresh** — suggestions panel polls every 30 s via `setInterval`, cleaned up on `onUnmounted`. Per-suggestion icon map added (⇌ ⚡ ◈ ✦ ⚠ ℹ).
- **Brand coherence pass** — `public/distira_app_icon.svg` and `favicon-32.png` synced from `brand/`; orphaned `katara-mark.svg` removed.

### V10.4 — Semantic Intent Engine

**Status:** Delivered (2026-03-14 — VERSION 10.4.0).

- **`detect_intent_scored(raw, client_app)`** — replaces first-match if/else with weighted multi-signal scoring. All intents scored in parallel; highest total wins. Confidence ∈ [0.0, 1.0].
- **VS Code Copilot context hint** — `client_app` containing "copilot"/"vs code" boosts code-adjacent intents (codegen, review, debug) for more accurate routing.
- **Structural signals** — code blocks, `diff --git`, composite `write a … function` pattern, language tags (rust/typescript/python function) all contribute independent scores.
- **"improve"/"optimize"/"refactor"/"clean up"** keywords now route to `review` (→ `qwen2.5-coder`) instead of falling through to `general`.
- **French keywords** extended across all intents: améliore, revue de code, résume, bogue, etc.
- **`intent_confidence: f32`** in `CompileResult` + all compile/chat JSON responses.
- **`compile_context_with_hint(raw, client_app)`** public API; `compile_context()` is a backward-compatible wrapper.
- **AuditView**: confidence badge beside intent label — `XX%` pill, colour-coded

### V10.7 — Savings & Impact Page + Dashboard Slimming

**Status:** Delivered (VERSION 10.7.0).

- **New SavingsView page** (`/savings`) — dedicated page for economic and environmental impact
- **SVG iceberg** widget replacing 3D CSS cube (translucent gradient, bob animation, glow filter)
- **Estimated Session Savings** progress bar — shows live $ saved from tokens avoided (replaces broken budget bar)
- **Leaf icon** in sidebar nav, new route between Overview and AI Flow
- **Overview slimmed** — moved 5 sections (savings tiles, ice cube, codegen vs review, intent distribution, suggestions) to SavingsView
- **Loading states** for suggestions (spinner, disabled button, loading/empty/populated states) green/amber/grey.

### V10.8 — Savings KPIs, Simplified Ice Widget & Live Data

**Status:** Delivered (VERSION 10.8.0).

- **KPI cards relocated** — "Tokens saved by compilation" and "Requests routed on-prem" moved from Runtime Audit to Savings & Impact page
- **Ice Preserved simplified** — complex SVG iceberg (gradients, filters, 4 computed polygon properties, bob animation) replaced with clean 🧊 emoji, consistent with 🌳 Tree tile
- **Live transitions** — CSS transitions on all savings values and KPI cards for visible reactivity
- **AuditView cleaned** — removed misplaced KPI bar, unused computed properties and CSS

### V10.9 — Slash Commands & Intent Override

**Status:** Delivered (VERSION 10.9.0).

- **Slash command system** — Prefix any prompt with `/debug`, `/code`, `/review`, `/summarize`, `/translate`, `/ocr`, `/dtlr`, `/fast`, `/quality`, or `/general` to override automatic intent detection with confidence 1.0.
- **`/dtlr` (Data to Local Routing)** — Forces on-prem routing regardless of intent; sets `force_local: true`.
- **`/fast`** — Aggressive 80% token reduction, routed to `openrouter-step-3.5-flash-cloud`.
- **`/quality`** — Preserves 50% of context, routed to `ollama-llama3.3` (high-quality local).
- **French variants** — `/résumé`, `/traduire`, `/rapide`, `/qualité` (and unaccented equivalents) supported natively.
- **`TaskRouting`** extended with `fast` and `quality` routing keys.
- **MCP** — `distira_compile` and `distira_chat` tool descriptions document slash commands.
- **11 new compiler tests**, 204 total workspace tests, 0 failures.
- **MCP crash fix** — literal `\n` syntax error on line 326 fixed (Node.js v24).
- **GitHub Actions** — `actions/checkout@v5`, `node-version: 22`.

### V10.10 — Multi-Pass Token Optimization

**Status:** Delivered (VERSION 10.10.0).

- **Post-reduction re-optimization** — Second optimizer pass runs after semantic reduction to catch patterns revealed by context trimming.
- **Convergence loop** — Optimizer iterates up to 3 total passes until token count stabilizes, squeezing maximum savings.
- **Stopword removal (Pass 7)** — 37 low-information stopwords stripped for codegen/summarize/general/fast intents.
- **URL & file-path compression (Pass 8)** — Long URLs (>40 chars) compressed to `domain/…/last-segment`.
- **Combined optimizer savings** — Increased from +10–30% to **+15–40%** on top of semantic passes.
- **Compiler pipeline** — Now 3-phase: pre-optimize → semantic reduce → post-optimize with convergence loop.
- **5 new optimizer tests** — Total 209 workspace tests, all passing.

### V10.11 — RCT2I Prompt Compiler

**Status:** Delivered (VERSION 10.11.0).

- **RCT2I prompt restructuring** — automatic prompt rewriting applying Rôle, Contexte, Tâches, Instructions, Improvement (RCT2I) sections before LLM submission.
- **Sentence-level segmentation** — single-line prompts split on `.` boundaries for accurate classification.
- **Bilingual** — French role/task/instruction keywords detected natively.
- **Intent-inferred roles** — compiler infers role when not explicitly declared.
- **Improvement hints** — per-intent quality improvement hints auto-appended.
- **8 new RCT2I tests** — 217 total workspace tests, all passing.
- **Pipeline** — Now 4-phase: pre-optimize → RCT2I restructure → semantic reduce → post-optimize with convergence loop.

### V10.12 — Aggressive Code Review & Codegen Token Reduction

**Status:** Delivered (VERSION 10.12.0).

- **Optimizer Pass 9** — Code boilerplate stripping: import/use collapse, license header removal, annotation stripping (codegen + review intents, 10–30% savings)
- **Optimizer Pass 10** — Code keyword abbreviation: 39 pattern replacements for codegen intent (5–12% savings)
- **Dedicated codegen reducer** — Structural code analysis: test block skipping, signature-only extraction (body dropped), structural line preservation, TODO/task line detection
- **Enhanced review reducer** — Smart diff extraction: keeps only change lines (+/-) and hunk headers, strips context lines and diff noise. Non-diff: 26 code-aware review keywords
- **Review distillation** — ÷2 → ÷3 (33% target instead of 50%)
- **16 new tests** — Covering all new passes and reducers (233 total workspace tests)

### V10.13 — Minimum 30 % Reduction Across All Intents

**Status:** Delivered (VERSION 10.13.0).

- **Per-intent reduction targets enforced** — 5 benchmark tests validate ≥ 30 % token reduction for general, debug, review, summarize, codegen intents.
- **Configurable salience keep ratio** — `reduce_by_salience_pct` with per-intent fractions (35–40 %) replaces the fixed 67 % keep.
- **Double `shape_by_intent` bug fixed** — Marker now applied once; eliminates double-prefix inflation.
- **RCT2I excluded for debug intent** — Preserves natural line structure (errors/stack traces) for the debug reducer.
- **Raised distillation divisors** — debug/review/codegen ÷3, general ÷5. Compile floor lowered to 8. Smart short-input protection (< 32 tokens = no reduction, 32–63 = gentle).
- **Tighter debug reducer** — Top 5 trace frames (was 10), fallback head/tail 3/6 (was 4/12).
- **238 tests, clippy clean, fmt clean.**

### V10.14 — Auto Efficiency Directives

**Status:** Delivered (VERSION 10.14.0).

- **Per-intent efficiency directives** — 7 intent-specific LLM instructions auto-injected into every request to reduce output tokens (debug, review, codegen, summarize, translate, ocr, general).
- **Always-on injection** — Replaces config-gated `concise_mode`; directives inject unconditionally via `inject_efficiency_directive`.
- **`efficiency_directive` field** exposed in `/v1/compile` response and `CompileResult` struct.
- **5 new tests** — Directive variation, content checks, compile integration, token overhead cap.
- **243 tests, clippy clean, fmt clean.**

### V10.15 — Cross-Request Deduplication, BPE-Boundary Truncation & RCT2I Dashboard

**Status:** Delivered (VERSION 10.15.0).

- **BPE-boundary aware truncation** — `truncate_to_token_budget` uses `token_count()` per line/word for accurate budget adherence. Partial lines terminated with `…` (U+2026).
- **Cross-request deduplication** — `compile_older_turns()` compiles system prompts and older messages through DISTIRA. `dedup_cross_messages()` fingerprint-deduplicates repeated paragraphs across conversation turns.
- **RCT2I metadata & tracking** — `RCT2I_applied`, `RCT2I_sections` in CompileResult and `/v1/compile` response. `RCT2I_applied_count` counter in MetricsSnapshot and `/v1/metrics`.
- **Dashboard RCT2I viewer** — "RCT2I Structured" KPI card in OverviewView. Dynamic RCT2I insight card in InsightsView showing activation rate and recommendations.
- **Short-input threshold raised 32 → 48** — BPE-accurate truncation needs more headroom for symbol-heavy inputs.
- **Marker cost deduction removed** — Intent marker added by `shape_by_intent` AFTER truncation, body budget no longer reduced.
- **11 new tests** — 6 compiler (BPE truncation, RCT2I metadata) + 5 core (compile_older_turns, dedup_cross_messages).
- **254 tests, clippy clean, fmt clean.**

### V10.16 — Advanced Compression & Commvault-Style Deduplication

**Status:** Delivered (VERSION 10.16.0).

- **Non-consecutive line deduplication** (Pass 11) — Commvault-inspired content-addressable dedup removes repeated lines scattered throughout a prompt. Uses HashSet fingerprints (lowercase + collapsed whitespace) to keep first occurrence and remove subsequent duplicates.
- **Compiled ≤ raw cap** — `compiled_tokens_estimate` capped at `raw_tokens_estimate` to fix marker overhead (`[k:intent]|`) causing negative savings.
- **Stronger compression** — Short-input threshold 48→40, distillation divisors debug/review/codegen 3→4, keep percentages lowered across all reducers.
- **Stopwords for review** — Boundary-safe stopword stripping now applies to code review intent.
- **Dashboard improvements** — 5-column KPI grid, Before/After Pipeline Example section in OverviewView.
- **RCT2I always-on** — word threshold lowered 12→4, `debug` intent enabled, 65+ EN/FR task/instruction keywords, `general` intent now gets [A] hint, `is_raw_artifact` guard preserves stack traces and diffs.
- **3 new tests** (compiled cap, dedup, marker absorption). 258 total, clippy clean.

### V10.17 — Provider Health Observatory & Metrics Export

**Status:** Delivered (VERSION 10.17.0).

- **Provider Health Observatory** — New `/providers` dashboard view with per-provider live status (healthy/degraded/down), request counts, error rates, and average latency. Colour-coded status badges, sortable table, responsive layout.
- **`GET /v1/metrics/export`** — Enterprise metrics export endpoint. Returns structured JSON: cumulative totals, per-provider breakdown (requests, errors, error rate, latency), per-intent breakdown (requests, tokens saved, savings %). Designed for enterprise reporting and observability integration.
- **`provider_health` in SSE/REST metrics** — Every `/v1/metrics` snapshot and SSE tick includes a `provider_health` array with per-provider health scoring based on error rate (≥50% = down, ≥10% = degraded) and latency (≥5000ms = degraded).
- **Server icon** in SvgIcon. Dashboard sidebar: 10 views (Overview, Savings, AI Flow, Memory, Insights, Benchmarks, Audit, Providers, Guide).
- **Metrics export button** — One-click JSON download in ProvidersView for enterprise reporting.
- **258 tests, clippy clean, fmt clean.**

### V10.17.1 — Transparent Template Intelligence

**Status:** Delivered.

- **Automated directive template selection** — Compiler now selects an internal efficiency template automatically from intent + content signals (no manual template prompt needed).
- **Signal-aware variants** — Built-in specializations for debug stack traces / CI failures, security-focused reviews, and codegen patch/test requests.
- **Cache-hit parity** — Cached compile responses now rebuild `efficiency_directive` using context-aware selection, keeping behavior consistent across cache hits and misses.
- **Validation** — Added template-selection tests in `compiler` crate; workspace tests remain green.

### V10.17.2 — Automatic Upstream Model Detection Hardening

**Status:** Delivered.

- **Freshness-first upstream lineage** — MCP now resolves upstream model/provider/client using per-request metadata first (tool args, chat `model`, MCP `_meta`, dynamic resolver), with persisted runtime context moved to fallback.
- **Runtime context auto-sync** — When newer upstream metadata is detected, MCP updates `/v1/runtime/client-context` automatically to keep Overview and Runtime Audit aligned.
- **Copilot fallback model** — Added safe Copilot fallback (`GPT-5.3-Codex`) when no upstream model signal is exposed by the client.
- **Provider inference update** — Upstream provider inference now recognizes `codex` and `o4` model families as OpenAI-family.

### V10.17.3 — MCP Metadata Probe & Hidden Model Discovery

**Status:** Delivered.

- **Broader hidden-signal detection** — MCP now scans arbitrary metadata paths/values for model, provider, and client clues instead of relying only on a small fixed key list.
- **Metadata probe artifact** — New `cache/mcp-meta-probe.json` records the latest candidate paths and values seen by the MCP layer, so undocumented VS Code/Copilot metadata can be verified empirically.
- **Product outcome** — After one real `@distira` request, DISTIRA can now prove whether VS Code exposed the selected model, instead of guessing silently.

### V10.18.0 — VS Code Live Model Detection

**Status:** Delivered.

- **Automatic upstream LLM detection** — DISTIRA now reads the currently active chat model directly from VS Code's `state.vscdb` SQLite database (`chat.currentLanguageModel.panel`). Zero manual configuration required — the dashboard Overview and Runtime Audit automatically show the real model selected by the user (e.g. Claude Opus 4.6, GPT-5.3-Codex).
- **Cross-platform support** — `state.vscdb` path resolved automatically on Windows, macOS, and Linux.
- **3 s read cache** — VS Code state is re-read at most every 3 seconds to minimize SQLite open/close overhead.
- **Model identifier humanization** — Raw VS Code identifiers (`copilot/claude-opus-4.6`) are parsed into human-readable names (`Claude Opus 4.6`) with correct provider inference.
- **Precedence update** — VS Code live state now slots between MCP `_meta` signals and dynamic resolver command in the upstream detection chain, superseding the static Copilot fallback model.

### V10.18.1 — Live Current Model Visibility

**Status:** Delivered.

- **Current selected upstream card** — Overview now shows the live selected VS Code model from runtime context as a dedicated panel, so operators can distinguish the current model from aggregated historical usage.
- **Provider normalization** — When the client brand is `GitHub Copilot` but the model family is known, the UI and MCP metadata now prefer the concrete provider (`Anthropic`, `OpenAI`, etc.).

- Streaming deduplication for SSE chat responses
- Dashboard: per-model latency heatmap over time
- Multi-tenant metrics isolation
- Prometheus / OpenTelemetry metrics export
- Dashboard RCT2I toggle + structured prompt viewer
