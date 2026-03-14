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

### V10 — Adaptive AI Optimization Network

- Learning routing loop (feedback from response quality)
- Provider capability graph
- Automated optimization recommendations
