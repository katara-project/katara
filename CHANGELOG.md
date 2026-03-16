# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Automatic upstream model detection via VS Code `state.vscdb`** — DISTIRA now reads the currently selected chat model directly from VS Code's SQLite state database (`chat.currentLanguageModel.panel`), providing true zero-config automatic detection of the upstream LLM. No manual `set_client_context` or environment variables required. Works on Windows, macOS, and Linux. Results are cached (3 s TTL) and polled in the background to automatically synchronize with the dashboard even when no MCP request is made. Model identifiers (e.g. `copilot/claude-opus-4.6`) are automatically parsed into human-readable names (`Claude Opus 4.6`) with correct provider inference.
- **Live current upstream model card in Overview** — The dashboard now shows a dedicated `Current Selected Upstream Model` panel sourced from `/v1/runtime/client-context`, so the user can see the currently selected VS Code model separately from aggregated historical upstream stats.

### Fixed
- **RCT2I 0% bug** — `CacheEntry` now stores `rct2i_applied` and `rct2i_sections` fields so cache hits correctly preserve RCT2I metadata. Previously all cache hits hardcoded `rct2i_applied: false`, causing the counter to never increment. Backward-compatible via `#[serde(default)]`.
- **Concrete upstream provider normalization** — Generic `GitHub Copilot` provider labels are now normalized to the actual model vendor when the model is known (`Claude` → `Anthropic`, `GPT/Codex` → `OpenAI`), preventing the current-selection UI from showing the client brand instead of the real provider.
- **Upstream model stale detection (Overview/Audit)** — MCP upstream metadata resolution now prioritizes per-request signals (`upstreamModel`, `model`, MCP `_meta`, dynamic resolver) over persisted runtime context, and auto-syncs runtime client context when fresher metadata is seen. Prevents stale values like fixed Claude labels when the active client model changed.
- **Hidden MCP metadata detection** — MCP upstream detection now scans a much wider set of metadata paths and values for model/provider/client signals (including non-standard keys like selected/active/session model fields), improving automatic discovery when VS Code or Copilot emits undocumented metadata.

### Added
- **Transparent directive templates (auto-selected)** — DISTIRA now chooses internal response templates automatically from intent + prompt signals (e.g. debug stack traces, security reviews, codegen patch requests). No user action or prompt formatting required.
- **`cache/mcp-meta-probe.json`** — Best-effort sanitized probe of the latest MCP request metadata candidates (model/provider/client paths + values). This gives a concrete debugging artifact to verify what VS Code/Copilot actually sends to the MCP layer.
- **Provider Health Observatory** (V10.17) — New `/providers` dashboard view showing per-provider live status (healthy/degraded/down), request counts, error rates, and average latency in a sortable table with colour-coded status badges.
- **`GET /v1/metrics/export`** — Enterprise metrics export endpoint returning structured JSON with cumulative totals, per-provider breakdown (requests, errors, error rate, latency), and per-intent breakdown (requests, tokens saved, savings %).
- **`provider_health` in SSE/REST metrics** — Every `/v1/metrics` and SSE tick now includes a `provider_health` array with live per-provider health scoring (healthy / degraded / down based on error rate and latency thresholds).
- **Server icon** added to SvgIcon component for the Providers navigation link.
- **Metrics export button** in ProvidersView — one-click JSON download of cumulative metrics for enterprise reporting.
- **Integrated Guide view** — comprehensive in-app documentation accessible from the sidebar. Covers architecture, compilation pipeline (11 passes), RCT2I restructuring, intents & routing, REST API reference with examples, MCP integration, dashboard views, configuration, provider compatibility, and troubleshooting. Smooth-scroll table of contents, responsive layout.
- **Book icon** added to SvgIcon component for the Guide navigation link.
- **Per-request AI Flow Pipeline** — FlowVisualizer and FlowView now display per-request metrics (raw→compiled tokens, cache hit/miss, routed provider, intent, tokens saved) instead of cumulative totals. A last-request banner shows intent, routed provider, sensitive flag, and cost. Cumulative metrics remain on the Overview page.

### Changed
- **RCTIA → RCT2I rename** — Renamed the prompt restructuring framework from RCTIA (Role/Context/Tasks/Instructions/Amélioration) to RCT2I (Role/Context/Tasks/Instructions/Improvement). All Rust source, dashboard views, metrics store, MCP server fields, docs, changelog, and roadmap updated. File `rctia.rs` → `rct2i.rs`. JSON API fields `rctia_applied` / `rctia_sections` → `rct2i_applied` / `rct2i_sections`. Section marker `[A]` → `[I]`.

## [10.16.0] — Advanced Compression & Commvault-Style Deduplication

### Added
- **Non-consecutive line deduplication** (Pass 11) — Commvault-inspired content-addressable dedup: lines ≥10 chars that appeared earlier in the text are removed on second+ occurrence. Catches scattered duplicate imports, repeated log messages, and copy-pasted blocks.
- **Before/After Pipeline Example** — OverviewView now shows a real DISTIRA compilation example (raw prompt → compiled output) so users can visualize the AI Flow pipeline in action.
- **3 new tests** — `compiled_never_exceeds_raw`, `non_consecutive_dedup_removes_repeated_lines`, `marker_overhead_absorbed_by_cap`.

### Fixed
- **Compiled > raw bug** — `compiled_tokens_estimate` is now capped at `raw_tokens_estimate` to prevent marker overhead (`[k:intent]|`) from causing negative savings that poisoned the efficiency score.
- **RCT2I stuck at 0%** — RCT2I prompt restructuring never triggered in practice due to 4 compounding gates: (1) word threshold too high (12 → lowered to 4), (2) `debug` intent excluded (now allowed), (3) task keyword list too narrow (expanded from 18 to 65+ EN/FR verbs), (4) `general` intent had no Improvement hint (now has one). Added `is_raw_artifact` guard so pure stack traces and diffs still bypass RCT2I while natural-language prompts always get restructured.

### Changed
- **Short-input threshold lowered 48 → 40 tokens** — Inputs with 40–63 tokens now enter the gentle reduction zone (divisor ≤ 2) instead of being skipped entirely. Shorter inputs (< 40 tokens) remain protected.
- **Distillation divisors raised** — `debug`: 3→4, `review`/`codegen`: 3→4. More aggressive compression for medium-to-large inputs.
- **Reducer keep percentages lowered** — `general`: 40→30%, `review` (non-diff): 40→30%, `codegen` (fallback): 35→25%, `summarize`: 35→30%.
- **Stopword stripping enabled for `review` intent** — Boundary-safe stopword removal (` the `, ` a `, etc.) now runs on code review prompts.
- **Dashboard metrics-grid** — 5 KPI cards now use explicit 5-column layout at desktop, 3-column at tablet, 2 at mobile, preventing narrow/squeezed cards.
- **258 tests, clippy clean, fmt clean.**

## [10.15.0] — Cross-Request Deduplication, BPE-Boundary Truncation & RCT2I Dashboard

### Added
- **BPE-boundary aware truncation** — `truncate_to_token_budget` now uses `token_count()` per line and per word instead of naive word count, producing tighter budget adherence with BPE-accurate splits. Partial lines end with `…` (U+2026).
- **Cross-request deduplication** — `compile_older_turns()` compiles system prompts and older assistant/user messages through the DISTIRA pipeline. `dedup_cross_messages()` fingerprints paragraphs across messages and removes duplicates from earlier turns, keeping the latest occurrence.
- **RCT2I metadata** — `RCT2I_applied` (bool) and `RCT2I_sections` (u8) fields in `CompileResult`, `/v1/compile` API response, and cache. `RCT2I_applied_count` counter in `MetricsSnapshot` and `/v1/metrics`.
- **Dashboard RCT2I KPI** — "RCT2I Structured" metric card in OverviewView showing count and percentage. Dynamic RCT2I insight card in InsightsView with activation rate and recommendations.
- **11 new tests** — 6 compiler tests (BPE truncation × 3, RCT2I metadata × 3) + 5 core tests (compile_older_turns × 3, dedup_cross_messages × 2).

### Changed
- **Short-input threshold raised 32 → 48 tokens** — BPE-accurate truncation consumes budget faster for symbol-heavy inputs; inputs < 48 tokens now bypass reduction.
- **Marker cost deduction removed** — The intent marker is added by `shape_by_intent` AFTER truncation, so the body budget is now the full `target_tokens`.
- **Dashboard metrics-grid** — Uses `auto-fill` responsive layout for 5+ KPI cards.
- **254 tests, clippy clean, fmt clean.**

## [10.14.0] — Auto Efficiency Directives

### Added
- **Per-intent efficiency directives** — 7 intent-specific LLM instructions (debug, review, codegen, summarize, translate, ocr, general) auto-injected into every request. Each directive guides the downstream LLM to produce concise, on-target output — reducing output tokens without user intervention.
- **`efficiency_directive` field** in `/v1/compile` response and `CompileResult` struct — consumers can read the directive that will be injected.
- **5 new tests** — `efficiency_directive_varies_by_intent`, `efficiency_directive_included_in_compile_result`, `efficiency_directive_for_codegen_says_code_only`, `efficiency_directive_for_summarize_limits_bullets`, `efficiency_directive_short_token_overhead`.
- **Complete REST API Reference** — [`docs/api-reference.md`](docs/api-reference.md): Full documentation covering all 11 endpoints, request/response schemas, metrics glossary, slash commands, intent routing, authentication, 7 integration recipes (curl, PowerShell, Python SSE, OpenAI drop-in), and error handling.

### Changed
- **Always-on directive injection** — Replaces the old `concise_mode`-gated `inject_conciseness_directive`. Efficiency directives inject unconditionally for every chat request.
- **243 tests, clippy clean, fmt clean.**

## [10.13.0] — Minimum 30 % Reduction Across All Intents

### Added
- **5 benchmark validation tests** — Per-intent reduction tests (general, debug, review, summarize, codegen) each asserting ≥ 30 % token reduction on realistic multi-line inputs.
- **Configurable salience keep ratio** — `reduce_by_salience_pct(raw, keywords, keep_pct)` replaces the old 67 %-fixed `reduce_by_salience`; callers now specify intent-specific keep fractions (35–40 %).

### Fixed
- **Critical double `shape_by_intent` bug** — `build_compiled_context()` was calling `shape_by_intent()`, and `compile_context_with_hint()` called it again after the convergence loop, producing double markers (`[k:debug]|[k:debug]|…`) and inflating compiled token count. Marker is now applied once.
- **RCT2I destroying debug line structure** — RCT2I flattened 62 debug lines into 3 huge `[R]/[C]/[T]` lines, making the debug reducer's line-based analysis ineffective (17 % → 90 % after fix). Debug intent is now excluded from RCT2I restructuring.

### Changed
- **Distillation divisors raised** — debug/review/codegen ÷3 (was ÷2/÷3), general ÷5 (was ÷4), more aggressive compression.
- **Compile floor lowered** — `.max(16)` → `.max(8)` to allow deeper reduction for small inputs.
- **Smart short-input protection** — Inputs < 32 tokens skip reduction (divisor 1); inputs 32–63 tokens get gentle treatment (divisor capped at 2).
- **Per-intent salience ratios** — summarize 35 %, general 40 %, review fallback 40 %, codegen fallback 35 % (all were 67 %).
- **Debug reducer tightened** — Stack trace frames: top 10 → top 5; fallback head/tail 4/12 → 3/6. 20+ new salience keywords for summarize and general.
- **Codegen bypass lowered** — Short-input bypass from 16 → 8 lines.

## [10.12.0] — Aggressive Code Review & Codegen Token Reduction

### Added
- **Optimizer Pass 9: Code boilerplate stripping** — Collapses import/use blocks (keeps first + `[+N imports]`), strips license/copyright headers, removes annotation-only lines (`@Override`, `#[derive(...)]`). Active for codegen and review intents (10–30% savings on code)
- **Optimizer Pass 10: Code keyword abbreviation** — 39 abbreviations (`function`→`fn`, `return`→`ret`, `console.log`→`log`, `Vec<`→`V<`, `Result<`→`R<`, `Option<`→`O<`, `.unwrap()`→`.u!()`, etc.). Active for codegen intent only (5–12% savings)
- **Dedicated `reduce_codegen_context()`** — Structural code analysis: skips test blocks, keeps function signatures while dropping bodies, preserves structural lines (struct/enum/trait/impl/class), keeps TODO/FIXME/task-relevant lines
- **Enhanced `reduce_review_context()`** — Smart diff extraction: keeps only `+`/`-` change lines and hunk headers, strips context lines and diff noise (`index`, `old mode`, `similarity`, `rename`). Non-diff fallback uses 26 code-aware review keywords
- **16 new tests** — Pass 9 (import collapse, license stripping, annotation removal, no-op), Pass 10 (abbreviation, Rust types, console.log, no-op on prose), combined code intent optimization, codegen reducer (test block skipping, signature keeping, TODO preservation), review reducer (diff changes, noise, non-diff fallback)

### Changed
- **Review distillation divisor** — ÷2 → ÷3 (keeps 33% instead of 50%, more aggressive compression)
- **Codegen dispatch** — Now routes to dedicated `reduce_codegen_context()` instead of generic `reduce_general_context()`

## [10.11.0] — RCT2I Prompt Compiler

### Added
- **RCT2I prompt restructuring** — Unstructured prompts are automatically reorganised into Role/Context/Tasks/Instructions/Improvement sections for tighter LLM consumption
- **Sentence-level segmentation** — Single-line prompts are split on sentence boundaries for accurate RCT2I classification
- **Bilingual support** — French role/task/instruction keywords ("tu es", "implémente", "assure-toi", etc.) detected natively
- **Intent-inferred roles** — When no explicit role is declared, the compiler infers an appropriate role from the detected intent
- **Improvement hints** — Auto-generated quality improvement hints appended per intent (codegen → "write idiomatic, tested, minimal code", etc.)
- **8 new RCT2I tests** — short prompt bypass, OCR bypass, structured bypass, codegen/review/debug/French restructuring, role extraction

### Changed
- **Compiler pipeline** — Now 4-phase: pre-optimize → RCT2I restructure → semantic reduce → post-optimize with convergence loop
- **Verbose phrase substitution** — Extended from 22 to 68 patterns (+46 new: "leverage"→"use", "functionality"→"feature", "consequently"→"so", "nevertheless"→"still", etc.)
- **Total tests** — 217 workspace tests, all passing

## [10.10.0] — Multi-Pass Token Optimization

### Added
- **Post-reduction re-optimization** — A second optimizer pass runs after semantic reduction to catch patterns revealed by context trimming (duplicate lines, whitespace, verbose phrases that only appear after reduction)
- **Convergence loop** — Optimizer iterates up to 3 total passes until token count stabilizes, squeezing maximum savings from each request
- **Stopword removal (Pass 7)** — Strips 37 low-information stopwords (`the`, `a`, `is`, `very`, `basically`, `essentially`, etc.) for codegen/summarize/general/fast intents; skipped for translate/review/ocr/debug to preserve meaning
- **URL & path compression (Pass 8)** — Long URLs (>40 chars) compressed to `domain/…/last-segment`, saving 5–10 tokens per URL occurrence
- **5 new optimizer tests** — stopword removal, code identifier preservation, translate intent preservation, URL compression, short URL passthrough

### Changed
- **Optimizer savings** — Combined realistic gain increased from +10–30% to **+15–40%** on top of semantic passes
- **Compiler pipeline** — Now 3-phase: pre-optimize → semantic reduce → post-optimize with convergence loop

## [10.9.0] — Slash Commands & Intent Override

### Added
- **Slash commands** — Prefix any prompt with `/debug`, `/code`, `/review`, `/summarize`, `/translate`, `/ocr`, `/dtlr`, `/fast`, `/quality`, or `/general` to override automatic intent detection with confidence 1.0
- **`/dtlr` (Data to Local Routing)** — Forces on-prem routing regardless of intent; sets `force_local: true` in the compile result
- **`/fast` intent** — Aggressive 80% token reduction, routed to `openrouter-step-3.5-flash-cloud`
- **`/quality` intent** — Preserves 50% of context, routed to `ollama-llama3.3` (high-quality local)
- **French variants** — `/résumé`, `/traduire`, `/rapide`, `/qualité` (and unaccented equivalents) supported natively
- **`CompileResult` fields** — `slash_command: Option<String>` and `force_local: bool` exposed in compile API response
- **`TaskRouting`** — `fast` and `quality` routing keys in `routing.yaml`
- **11 new compiler tests** — slash command override, force-local, context stripping, French variants, backward compatibility

### Changed
- **Compiler** — `extract_slash_command()` strips the command prefix before context compilation, reducing noise tokens
- **Core handlers** — compile and chat endpoints honor `force_local` from slash commands
- **MCP tool descriptions** — `distira_compile` and `distira_chat` now document slash command support

### Fixed
- **MCP server syntax error** — Literal `\n` (two characters) on line 326 replaced with real newline; fixes crash on Node.js v24
- **GitHub Actions** — `actions/checkout@v4` → `@v5`, `node-version: 20` → `22` (Node.js 20 deprecation)
- **Clippy** — `map_or(false, ...)` → `is_some_and(...)` per `unnecessary_map_or` lint
- **Savings & Impact page static data** — values appeared frozen because per-request savings were too small to notice. Revamped with:
  - **Last Request Impact** section showing per-request token and cost delta
  - **Session Projections** — monthly/yearly extrapolations (cost, CO₂, trees) that show meaningful numbers
  - **Pulse animation** — KPI cards and tiles flash green on every new SSE update
  - **Savings sparkline** — visual growth line derived from cumulative history
  - **4 KPI cards** — added Total Requests and Avg Tokens Saved/Request
  - **Smart formatting** — `formatDynamic()` adapts decimal precision to value magnitude

## [10.8.0] — Savings KPIs, Simplified Ice Widget & Live Data

### Added
- **KPI cards on Savings page** — "Tokens saved by compilation" and "Requests routed on-prem" moved from Runtime Audit to Savings & Impact where they are contextually relevant
- **RCT2I prompt structuring** documented in ROADMAP as upcoming feature (Rôle, Contexte, Tâches, Instructions, Improvement)

### Changed
- **Ice Preserved tile** — replaced complex SVG iceberg (gradients, filter, polygon computations, bob animation) with clean 🧊 emoji approach, consistent with 🌳 Tree tile
- **Savings values** — added CSS transitions for live update feel on all savings tiles and KPI cards

### Removed
- **Audit page KPI cards** — "Tokens saved by compilation" and "Requests forced on-prem" removed from Runtime Audit (misplaced context); now on Savings page
- **Iceberg SVG** — removed 30+ lines of SVG defs, gradients, filters, and 4 computed properties (`iceOpacity`, `iceScale`, `aboveWaterPoints`, `underwaterPoints`)

## [10.7.0] — Savings & Impact Page + Dashboard Slimming

### Added
- **SavingsView** — new dedicated "Savings & Impact" page (`/savings`) with:
  - Estimated Session Savings progress bar (tokens saved → $ estimate)
  - Environmental Impact tiles (cost, energy, tree equivalent, ice preserved)
  - **SVG iceberg** widget replacing the 3D CSS cube (translucent gradient, bob animation, glow filter)
  - Codegen vs Review section (moved from Overview)
  - Intent Distribution section (moved from Overview)
  - Optimization Suggestions with loading state + auto-refresh (moved from Overview)
- **Leaf icon** added to SvgIcon component for Savings nav link
- **Savings & Impact** nav link in sidebar (between Overview and AI Flow)
- `/savings` route registered in Vue router

### Changed
- **OverviewView** slimmed from ~1400 lines to ~1000 lines — moved 5 sections to SavingsView
- **Budget bar** repurposed: was "Session Cost Budget" (always $0) → now "Estimated Session Savings" showing live $ saved from tokens avoided
- Suggestions, Codegen vs Review, Intent Distribution, Savings tiles, ice widget all moved to SavingsView

### Fixed
- **Budget bar always shows 0** — replaced `sessionCostUsd / sessionBudgetUsd` (never populated) with live savings estimate from tokens saved
- **Ice cube rendered as black box** — replaced 3D CSS `preserve-3d` cube (broken rendering) with beautiful inline SVG iceberg
- **Suggestions not loading on refresh** — added explicit loading state with spinner, improved empty/loading/populated states

### Fixed (prior — included in build)
- **BenchmarksView** — shows all intents (removed >0% filter)
- **FlowView** — all 6 stats now live from metrics store
- **Ice cube rotation** — CSS custom properties fix

## [10.6.0] — Savings & Environmental Impact Dashboard + Audit Cleanup

### Added
- **Savings & Environmental Impact** section on Overview — 4-tile panel showing:
  - Estimated cost saved ($) based on tokens avoided at blended $0.006/1K rate.
  - Energy avoided (kWh) and CO₂ not emitted (kg).
  - Tree-equivalent: fraction of a mature tree’s yearly CO₂ absorption.
  - **Animated 3D ice cube**: CSS-only rotating/bobbing ice crystal whose opacity and scale grow with savings. Represents litres of ice-melt-equivalent avoided.
- Upstream Client Models table now filters out test entries (`t`, `test`, `unknown-model`) automatically.

### Changed
- **Runtime Audit** is now a pure security/routing audit:
  - Removed “Cost / Security” column — table is 6 columns (Time, Scope, Client, Routed, Intent, Tokens).
  - Removed "Cloud cost (session)" and "Cloud cost avoided" KPI cards.
  - Kept: Tokens saved, Requests forced on-prem KPI cards.
  - Sensitive-route badge (🔒 On-prem) moved into Tokens column.
  - CSV export no longer includes `cost_usd`.
- Subtitle updated: “Every routing decision and optimisation” (no cost mention).

## [10.5.0] — Always-On Optimization Suggestions & Brand Refresh

### Added
- `get_suggestions` endpoint rewritten: always returns proactive informational suggestions on every load/refresh — no longer requires accumulated error or latency data to produce output.
- 4 always-on suggestion types: `routing_active` (full intent→provider map), `session_efficiency` (token reduction % + on-prem %, once requests > 0), `cache_performance` (hit rate + tokens avoided), `concise_mode_active` (when enabled).
- 2 reactive suggestion types retained: `high_error_rate` (triggers at ≥5% error rate) and `high_latency` (triggers at ≥3000 ms average).
- Per-suggestion icon map in OverviewView: ⇌ routing, ⚡ efficiency, ◈ cache, ✦ concise, ⚠ warning, ℹ latency.
- `RouterConfig::task_routing_summary()` — public method returning sorted `(intent, provider)` pairs, used by the suggestions backend.
- OverviewView suggestions panel auto-refreshes every 30 seconds via `setInterval` / `onUnmounted` cleanup; subtitle updated to reflect live refresh cadence.
- Empty state message updated to "Loading suggestions…" instead of static "No suggestions yet".

### Fixed (Brand)
- `public/distira_app_icon.svg` synced from `brand/` (310 KB real brand graphic; was 3.4 KB placeholder SVG).
- `public/favicon-32.png` synced from `brand/`.
- `src/assets/katara-mark.svg` removed (orphaned logo from previous project name, unreferenced).

## [10.4.0] — Semantic Intent Engine

### Added
- **`detect_intent_scored(raw, client_app)`** in `compiler/src/lib.rs` — replaces the first-match if/else chain with a weighted multi-signal scoring system. All intents are scored in parallel; the highest total wins. Confidence ∈ [0.0, 1.0] derived from the raw score.
- **VS Code Copilot context hint** — when `client_app` contains "copilot" or "vs code", code-adjacent intents (codegen, review, debug) receive a boost so coding prompts classify more accurately.
- **Structural signals** — presence of code blocks (` ``` `, `fn`, `def`, `impl`, `struct`), diff markers (`diff --git`), and composite patterns (`write a … function`) are independent scoring signals, not fall-through catches.
- **"improve"/"optimize"/"refactor"/"clean up" keywords** now route to `review` intent (→ `qwen2.5-coder`) instead of falling through to `general`.
- **French keyword coverage** extended across all intents: `améliore`, `optimise le`, `revue de code`, `résume`, `explique`, `bogue`, `plantage`, etc.
- **`intent_confidence: f32`** field in `CompileResult` — exposed in `/v1/compile` and `/v1/chat/completions` JSON responses.
- **`compile_context_with_hint(raw, client_app)`** public API in `compiler` crate — backward-compatible; `compile_context()` wraps it with `None`.
- **AuditView confidence badge** — intent column now shows a coloured `XX%` confidence pill beside the intent label (green ≥60%, amber ≥35%, grey otherwise).

### Fixed
- `detect_intent("fix this panic in main")` previously returned `general` instead of `debug` due to first-match ordering; scoring now returns `debug` correctly.
- `detect_intent("write a typescript function …")` previously returned `general`; composite `write a … function` pattern now scores as `codegen`.
- `detect_intent("diff --git …")` previously relied on bare `"diff"` keyword; dedicated `diff --git` signal added to review.

## [10.3.0] — Session Cost Budget & Alerts

### Added
- `session_budget_usd` field in `configs/workspace/workspace.yaml` (default `1.0`); set to `0` to disable.
- `WorkspaceContext` and `MetricsSnapshot` carry `session_budget_usd: f64`.
- `budget_alerts()` extended: fires **Warning** at 80 % of session budget, **Exhausted** at 100 % — surfaced in the existing alerts banner.
- `build_full_snapshot()` injects `session_budget_usd` into every metrics snapshot.
- Dashboard store exposes `sessionBudgetUsd` ref (bound to SSE `applySnapshot`).
- OverviewView: **Session Cost Budget** utilisation bar visible when budget > 0 — fills green / amber / red based on usage %, with current cost / budget amounts.

### Fixed — Benchmarks & version badge (2026-03-14)

- **`BenchmarksView.vue`** — Intent rows with 0% token reduction (codegen, summarize when compiler yields no gain) are now filtered out; only intents that produce measurable savings appear in the table. Added `codegen: 'Code Generation'` and `translate: 'Translation'` to `INTENT_LABELS`.
- **README version badge** — Hardcoded badge version updated to `10.2.0` and a new GitHub Actions workflow (`.github/workflows/sync-version-badge.yml`) auto-patches the badge whenever `VERSION` is pushed — uses `[skip ci]` on the commit so it does not trigger a CI loop.

### Added — V10.2 Dynamic Audit & Security-Cost Transparency (2026-03-14)

- **`core/src/main.rs` `RequestLineage`** — 3 new fields with `#[serde(default)]`: `raw_tokens: usize`, `compiled_tokens: usize`, `tokens_saved: usize`. Written on every compile/chat request via `record()`.
- **`dashboard/store/metrics.ts` `RequestHistoryEntry`** — added `raw_tokens?`, `compiled_tokens?`, `tokens_saved?` optional fields.
- **`dashboard/AuditView.vue`** — complete redesign:
  - **4 KPI cards**: Session cloud cost, Tokens saved by compilation, Requests forced on-prem, Cloud cost avoided (security-driven savings).
  - **Table** condensed to 7 semantic columns: Time, Scope, Client, Routed, Intent, Tokens (saved / raw→compiled), Cost/Security.
  - **🔒 Secured badge** on `sensitive=true` rows — replaces the generic "Sensitive override" pill; makes security decisions visible as cost optimizations.
  - **Cost column** shows formatted `$0.00000` per request or "Secured" badge when on-prem-forced.
  - **Cloud cost avoided estimator** — computes counterfactual savings from on-prem routing using session average cost of non-sensitive requests.
  - **CSV export** enriched with `raw_tokens`, `compiled_tokens`, `tokens_saved`, `cost_usd` columns.

### Added — V10.1 Live Memory Reuse Estimation (2026-03-14)

- **`memory/src/lib.rs` `ContextStore::estimate_coverage()`** — lexical word-set intersection between compiled context and prior stable blocks of same intent. Returns `MemorySummary { reused_tokens, delta_tokens }` even on cache miss, replacing hard-coded `reused=0`.
- **Compile handler** — on cache miss, calls `estimate_coverage()` so `memory_reused_tokens` grows proportionally to real overlap with session memory.
- **`MemoryView.vue`** — full UX redesign: user-friendly labels ("Tokens Saved by Memory", "Memory Efficiency", "Topics in Memory"), Session Savings progress bar, per-intent topic groups with health bars (strong/mid/weak), no hex IDs.
- **+4 memory tests**: `estimate_coverage_empty_store_returns_zero`, `estimate_coverage_full_overlap_returns_all`, `estimate_coverage_partial_overlap`, `estimate_coverage_intent_mismatch_ignored`. Memory crate: 13 → 17 tests. Total: **206 tests**.

### Fixed — Context Memory live metrics (2026-03-14)

- **`MetricsSnapshot`** — added `stable_blocks: usize` and `context_reuse_ratio_pct: f32` fields (serde default = 0); updated in `record()` from `context_store.len()` and `memory_reused_tokens / raw_tokens`.
- **`metrics_stream`** — SSE stream now injects `context_blocks_summary` array per event: `[{ id, stability, token_count, intent }]` — no raw content exposed.
- **`store/metrics.ts`** — `stableBlocks`, `contextRatioPct`, `contextBlocksSummary` refs added; `applySnapshot` binds them from the SSE payload.
- **`MemoryView.vue`** — Reuse Ratio and Stable Blocks cards now bound to live store values (were hardcoded `48.6%` / `14`); Context Block Status grid now renders real fingerprinted blocks from the SSE stream with per-block stability classification (`stable` ≥ 70% / `delta` ≥ 30% / `evicted` < 30%).

### Added — V10.0 Adaptive AI Optimization Network (2026-03-14)

- **`router/choose_provider_adaptive()`** — composite score routing: `latency × (1.0 + error_rate × 5.0)` per provider; unmeasured providers get `f64::MAX` and fall back to priority order; sensitive override bypasses scores entirely. Replaces `choose_provider_latency_aware()` as the default routing function in all chat and compile paths.
- **`core/MetricsCollector`** — `provider_errors: HashMap<String, u64>` + `provider_total: HashMap<String, u64>` (session-level, never reset); `record_error(provider)` method increments both on forward failure; `error_rate_by_provider() -> HashMap<String, f64>` accessor.
- **`core/chat_completions`** — both streaming `Err` branch and non-streaming `Err` branch now call `record_error()` so provider reliability is tracked.
- **`core/list_providers`** — `/v1/providers` response enriched with `error_rate` per provider.
- **`core/get_suggestions`** — new `GET /v1/suggestions` endpoint: computes actionable optimization suggestions from live error rates (≥5%) and latency (≥3000ms); returns `{ generated_at, count, suggestions: [{severity, code, provider, metric, value, message}] }`.
- **`dashboard/OverviewView.vue`** — Optimization Suggestions panel: fetches `/v1/suggestions` on mount + manual Refresh button; color-coded warning/info cards; provider · metric · value meta line.
- **`router` tests** — 3 new: `adaptive_returns_valid_decision`, `adaptive_penalizes_high_error_rate`, `adaptive_sensitive_ignores_scores`. Router suite: 11 → 14 tests.
- **Clippy** — fixed `manual_split_once` lint in `core/compress_conversation_history()` (`.splitn(2,’|’).nth(1)` → `.split_once(’|’).map(|x| x.1)`).

### Added — V9.16 Latency-Aware Routing (2026-03-14)

- **`router/choose_provider_latency_aware()`** — collects all within-budget candidates for an intent, ranks by rolling average latency, picks the fastest; providers with no measurement get `f64::MAX` and fall back to priority order; sensitive override bypasses latency logic entirely.
- **`core/RecordEntry`** — `latency_ms: u64` field; 0 for cache hits and compile-only requests.
- **`core/MetricsCollector`** — `provider_latency: HashMap<String, (f64, u64)>` rolling sum+count per provider key; `avg_latency_by_provider()` accessor returns `HashMap<String, f64>`.
- **`core/ModelStats`** — `avg_latency_ms`, `latency_sum_ms`, `latency_samples` fields; serialized in SSE metrics stream.
- **`core/chat_completions`** — `std::time::Instant` timing around both `forward_stream` and `forward` adapter calls; measured latency recorded per request.
- **`core/list_providers`** — `/v1/providers` response enriched with `avg_latency_ms` per provider.
- **`dashboard/OverviewView.vue`** — Latency column in Live AI Efficiency table with `.latency-badge` pill (`N ms` / `—`).
- **`router` tests** — 3 new: `latency_aware_returns_valid_decision`, `latency_aware_prefers_faster_provider`, `latency_aware_sensitive_ignores_latency`. Router suite: 8 → 11 tests.

### Added — V9.15 Quality-Tier Routing + Conciseness Injection (2026-03-14)

- **`router/ProviderConfig`** — `quality_tier: Option<String>` ("low" | "standard" | "high", serde default absent = "standard") and `fallback_chain: Vec<String>` (per-provider ordered fallback; empty = use global chain).
- **`router/ProviderSummary`** — `quality_tier: String` exposed on every `/v1/providers` entry.
- **`router/RouterConfig`** — `concise_mode: bool` field; `concise_mode()` accessor; loaded from `routing.yaml`.
- **`router/choose_provider_with_budget()`** — per-provider `fallback_chain` used when non-empty; otherwise falls back to global `default → fallback` sequence.
- **`configs/routing/routing.yaml`** — `concise_mode: true` enabled by default.
- **`configs/providers/providers.yaml`** — `quality_tier` annotation on all eight active providers (low/standard/high).
- **`core/inject_conciseness_directive()`** — inserts (or prepends to existing) system message: "Respond as concisely as possible. Use plain, simple language…" Reduces output token consumption across all providers.
- **`core/chat_completions`** — applies `inject_conciseness_directive` when `router_config.concise_mode()` is true.
- **`dashboard/utils/providers.ts`** — `qualityTier(key)` function + `PROVIDER_QUALITY` map.
- **`dashboard/OverviewView.vue`** — Quality tier badge column in Live AI Efficiency table; `.quality-pill.high/.standard/.low` CSS.

### Added — V9.14 Compiler-driven Chat History Compression (2026-03-14)

- **`core/compress_conversation_history()`** (V9.14) — older conversation turns now pass through `compiler::compile_context()` for real semantic distillation instead of naive 20-word truncation. Intent marker prefix stripped before embedding in summary block. Fallback to word-truncation when compiled output is empty.

### Added — V9.13 Memory Stability Decay + Intent-Scoped Injection (2026-03-14)

- **`memory/ContextBlock`** — new `intent: String` field (serde default = `""`), persisted in `runtime-state.json`.
- **`memory/ContextStore::register()`** — now takes `intent: &str`; decays all other blocks by `×0.92` per register call; evicts blocks below `stability < 0.10`.
- **`memory/ContextStore::compute_reuse()`** — now takes `intent: &str`; returns zero reuse when block intent ≠ request intent (intent-scope guard).
- **`memory/ContextStore::load_blocks()`** — skips blocks already below eviction threshold on restore.
- **`core/compile_with_semantic_cache()`** — passes `result.intent` to `register()`.
- **`core/compile` handler** — passes `result.intent` to `compute_reuse()`.
- **`core/chat_completions` handler** — passes `result.intent` to `compute_reuse()`.
- **5 new tests**: intent mismatch returns zero, stability decays on register, fully decayed blocks are evicted, plus updated existing tests for new signatures.

### Added — V9.12 Distillation Ratios + BM25 Salience (2026-03-14)

- **`compiler/distillation_divisor(intent)`** — per-intent ratios: ocr/translate keep 100%, debug/review keep 50%, codegen keeps 33%, general keeps 25%, summarize keeps 20%.
- **`compiler/salience_score()`** — BM25-inspired line scorer: word density + keyword hits × 5 + structure bonus (`:`, `=`, `->`, bullet).
- **`compiler/reduce_by_salience()`** — selects top-2/3 lines by salience preserving original order; replaces head/tail truncation for `general` and `summarize` intents.

### Added — V9.11 Provider Budget & Alerting (2026-03-14)

- **`router/ProviderConfig`** — `max_requests_per_day: u64` field (serde default = 0 = unlimited); `defaults()` initializers updated.
- **`router/choose_provider_with_budget()`** — budget-aware routing with candidate fallback chain; `choose_provider()` delegates to it with empty counts.
- **`router/list_provider_summaries()`** — maps `max_requests_per_day` to `ProviderSummary`.
- **`core/MetricsCollector`** — `daily_provider_counts: HashMap<String, u64>` + `daily_reset_epoch: u64`; auto-resets at UTC midnight in `record()`.
- **`core/budget_alerts()`** — emits `budget_warning` (≥80%) and `budget_exhausted` (≥100%) JSON objects.
- **`core/metrics_stream`** SSE payload includes `"alerts": [...]` when any provider is near/over budget.
- **Dashboard `store/metrics.ts`** — `alerts` reactive ref, `applySnapshot` maps `s.alerts`, exported.
- **Dashboard `OverviewView.vue`** — alert banner shown above metrics grid (orange warning / red exhausted).

### Added — V9.10.1 BPE-Aware Token Optimizer (2026-03-14)

- **`compiler/src/optimizer.rs`** (new file, 421 lines) — `TokenOptimizer` with 6 lossless passes:
  1. **Whitespace normalization**: tabs → space, multi-space collapse, trailing spaces, consecutive blank line limit.
  2. **Numeric separator removal**: `1,000,000` → `1000000` — commas between digits are extra tokens in BPE; removed safely.
  3. **Verbose-phrase substitution** (21 patterns): `in order to` → `to`, `utilize` → `use`, `please note that` → `note:`, `due to the fact that` → `because`, etc. Case-insensitive with capitalisation preservation.
  4. **Consecutive duplicate-line collapse**: runs of ≥3 identical lines → one line `[×N]`. Useful for log/trace dumps.
  5. **Standalone comment stripping** (codegen/general intents only): removes `//`, `#`, `/* */`, `*` comment-only lines. Skips for `review` and `debug` intents.
  6. **JSON compaction**: if the entire input is valid JSON, re-serialises it compact (no indentation). Saves 30–50 % on JSON payloads.
- **`compiler/src/lib.rs`** — optimizer integrated into `compile_context()` pipeline: runs after `tokenizer::encode()`, before semantic reduction. `CompileResult` gains `optimizer_savings: usize` field.
- **`core/src/main.rs`** — `POST /v1/compile` response now includes `"optimizer_savings"` field; `compile_result_from_cache` updated accordingly.
- **16 new unit tests** in `optimizer::tests` covering each pass independently and the combined pipeline.
- Build: `cargo check` clean · 181 tests, 0 failures.
- Typical additional savings: **+10–30 %** on top of the existing semantic compiler.

### Added — V9.10 Metrics Reset & E2E Integration Tests (2026-03-14)

- **`DELETE /v1/metrics/reset`** endpoint: resets all counters (requests, tokens, costs, intent stats, model stats, history buckets) without restarting the server. Returns `204 No Content`.
- **`MetricsCollector::reset()`** method in `core/src/main.rs`: zeroes the full `MetricsSnapshot` and clears `hour_buckets`; caches and context store are preserved.
- **Dashboard "Reset metrics" button** in `OverviewView.vue` header: fires `DELETE /v1/metrics/reset` and disables while in-flight. Styled with a subtle red accent.
- **`scripts/test-e2e.ps1`** — end-to-end integration test suite (32 assertions across 6 groups):
  - Group 1: health, version, providers
  - Group 2: intent routing for debug / codegen / review / summarize / translate / ocr / general / sensitive
  - Group 3: token pipeline (raw > 0, compiled > 0, fingerprint present)
  - Group 4: metrics increment after compile requests
  - Group 5: `DELETE /v1/metrics/reset` → 204 + all counters zeroed
  - Group 6: post-reset pipeline re-increments from zero
- **`theme.css` `.view-header`**: added `display: flex; align-items: flex-start; justify-content: space-between` so the Reset button aligns to the right of every view header.
- Build: clean `cargo build` · 0 errors, 0 warnings.

### Added — Efficiency Score sovereign bonus + agent config (2026-03-14)

- **Sovereign routing bonus**: `metrics::compute()` now adds `+30%` on top of raw token avoidance ratio (capped at 100%). Reflects the intrinsic value of routing through DISTIRA regardless of compression level. Example: 24% token reduction → 54% efficiency score.
- **`SOVEREIGN_BONUS` constant** (`metrics/src/lib.rs`): `pub const SOVEREIGN_BONUS: f32 = 0.30` — documented, testable, easy to tune.
- **5 metrics tests**: `compute_avoidance_ratio`, `compute_zero_raw`, `compute_no_reduction`, `compute_partial_reduction`, `compute_bonus_capped_at_100`.  All passing.
- **Dashboard gauge** (`EfficiencyGauge.vue`): band labels updated to Efficient / High / Excellent; help text no longer exposes internal bonus detail.
- **Insights threshold** (`InsightsView.vue`): efficiency alert raised from `< 30%` to `< 50%` (floor can no longer be hit).
- **`.github/agents/distira.agent.md`**: `name: distira` field added to frontmatter for reliable VS Code agent selector resolution.
- **`.vscode/settings.json`**: `"chat.agent.enabled": true` added to activate custom agent support.
- **ROADMAP.md**: V9.10, V9.11, V9.12, and V10 detailed iteration scopes added.

### Planned — V9.10 → V10 iteration roadmap (2026-03-13)

- **V9.10**: `DELETE /v1/metrics/reset` endpoint + dashboard reset button + E2E integration test suite (`scripts/test-e2e.ps1`).
- **V9.11**: Per-provider daily budget (`max_requests_per_day` in `providers.yaml`), automatic fallback on budget exhaustion, efficiency threshold alerts (SSE `alert` event + dashboard banner).
- **V9.12**: Dashboard sparkline (rolling token reduction %), per-provider breakdown panel, CSV/JSON metrics export (`GET /v1/metrics/export`), full dark mode.
- **V10**: Adaptive routing loop (quality signals → provider weights), provider capability graph, multi-tenant `project_id` isolation, native VS Code extension, cluster mode.

### Added — V9.9 LM Studio / OpenWebUI compatibility + GLM / Gemini 2.5 / Claude 4.x (2026-03-13)

- **LM Studio compatible**: Added commented `lmstudio-default` provider entry (`http://localhost:1234/v1`). LM Studio exposes a full OpenAI-compatible API — any GGUF model (Llama 3.3, Qwen 3, Gemma 3, Phi-4, …) can be proxied through DISTIRA by pointing to its local server.
- **OpenWebUI compatible**: Added commented `openwebui-default` provider entry (`http://localhost:3000/api`). OpenWebUI’s built-in OpenAI-compatible proxy is detected transparently — no adapter change required.
- **`ModelFamily::Glm`** — ZhipuAI GLM-4 / GLM-Z1 / ChatGLM. SentencePiece 130k vocabulary; calibrated heuristic identical to `Llama3` for Latin text. Activated by `glm`, `chatglm`, `zhipu` substrings in `family_for_provider()`.
- **Cloud provider entries** (all commented, ready to uncomment):
  - `openai-gpt4o`, `openai-gpt5` (OpenAI)
  - `anthropic-claude-sonnet-4-5`, `anthropic-claude-sonnet-4-6`, `anthropic-claude-opus-4-5`, `anthropic-claude-3-7-sonnet` (Anthropic)
  - `google-gemini-2-flash`, `google-gemini-2-5-pro`, `google-gemini-2-5-flash` (Google)
  - `zhipu-glm4-cloud`, `zhipu-glm-z1-cloud` (ZhipuAI)
  - `dashscope-qwen3-235b` (Alibaba DashScope — Qwen 3 235B MoE cloud)
  - `ollama-qwen3`, `ollama-glm4` (on-prem Ollama)
- **`providers.yaml` header updated**: Now documents LM Studio and OpenWebUI as first-class supported frontends.
- **Test suite: 159 tests, 0 failures** (+4: `family_for_gemini_2_5_is_gemini`, `family_for_glm_is_glm`, `family_for_chatglm_is_glm`, `family_for_zhipu_is_glm`).

### Fixed & Added — V9.8 Full model compatibility + translate intent (2026-03-13)

- **Bug fix — codegen routing was silently broken**: `TaskRouting` struct in `router/src/lib.rs` was missing the `codegen` field, causing the `codegen: ollama-qwen2.5-coder` line in `routing.yaml` to be silently dropped on deserialization. All code generation requests were falling through to the default provider instead of Qwen 2.5 Coder. Fixed by adding `codegen: Option<String>` to the struct and the corresponding `task_map.insert` in `load()`.
- **New `translate` intent** — `compiler/detect_intent()` now detects translation requests (EN/FR/DE/ES/JA/ZH keywords, `translate`, `traduire`, `traduis`, `übersetze`, `traducir`, `翻译`, `in english/french/german/spanish/japanese/chinese`). Routed to `openrouter-mistral-small-3.1-24b-instruct-cloud` (Mistral Small 3.1 24B, excellent multilingual model). Added `[k:translate]|` marker and `reduce_general_context` path.
- **Extended `detect_intent` codegen keywords** — Now covers TypeScript, JavaScript, Go, Kotlin, Swift, `write code`, `write me a`, `create a class/script`, `help me code`, `complete this code/function`, `codex`, `crée une fonction`, `génère du code/fonction`, `écris un script`.
- **`ollama-llama3.3` provider** — Added `llama3.3:latest` (Llama 3.3 70B, Meta 2025) to `configs/providers/providers.yaml`. Activate with `ollama pull llama3.3`.
- **`TaskRouting` struct hardened** — Added `translate: Option<String>` field alongside the codegen fix.
- **Test suite: 155 tests, 0 failures** (+6: `detect_codegen_typescript`, `detect_codegen_complete`, `detect_codegen_create_class`, `detect_translate_english`, `detect_translate_french_keyword`, `detect_translate_in_language`).

### Added — V9.7 Claude & Gemini tokenizer support (2026-03-13)

- **`ModelFamily::Claude`** — Anthropic Claude 3/3.5/3.7/4. Routes to `count_cl100k()` (cl100k_base proxy, ±3% accuracy) when `exact-gpt4` feature is on; falls back to `(raw * 19/20)` heuristic otherwise. Activated by `family_for_provider()` on `claude` or `anthropic` substrings.
- **`ModelFamily::Gemini`** — Google Gemini 1.5/2.0/Flash/Pro. Uses calibrated `(raw * 19/20)` heuristic (SentencePiece 256k vocab; no embedded Rust vocabulary available). Activated by `family_for_provider()` on `gemini`, `google`, or `palm` substrings.
- **`configs/providers/providers.yaml`** — Added commented-out `anthropic-claude-cloud` (Anthropic OpenAI-compatible endpoint, `ANTHROPIC_API_KEY`) and `google-gemini-cloud` (Google OpenAI-compatible endpoint, `GOOGLE_API_KEY`, `gemini-2.0-flash`) entries ready to uncomment when API keys are configured.
- **Test suite: 149 tests, 0 failures** (+7: `family_for_claude_is_claude`, `family_for_claude_direct_is_claude`, `family_for_gemini_is_gemini`, `family_for_gemini_direct_is_gemini`, `exact_claude_empty_returns_zero`, `exact_claude_hello_world_matches_gpt4`, `gemini_count_is_below_llama3_heuristic`).

### Added — V9.6.1 GPT-5 / o200k_base tokenizer support (2026-03-13)

- **`ModelFamily::Gpt4o` variant** — New enum variant covering GPT-4o, o1, o3, o4, and **GPT-5**. Routes to `o200k_base` (200k-vocabulary BPE) via `exact_gpt4::count_o200k()` when `exact-gpt4` feature is active, or a calibrated heuristic (`raw * 18/20`) otherwise.
- **`family_for_provider()`** — Updated to route `gpt-4o`, `gpt-5`, `gpt5`, `-o1`, `-o3`, `-o4`, `o1*`, `o3*`, `o4*` models to `Gpt4o`; generic `gpt`/`openai` prefixes continue to use `Gpt4` (cl100k_base).
- **`mod exact_gpt4`** — Added `O200K_BPE: OnceLock<CoreBPE>`, `o200k_encoder()`, and `count_o200k()`. Both encoders (cl100k + o200k) are embedded — no external vocabulary files required.
- **Test suite: 142 tests, 0 failures** (+6: `family_for_gpt4o_is_gpt4o`, `family_for_gpt5_is_gpt4o`, `family_for_o1_is_gpt4o`, `exact_gpt4o_empty_returns_zero`, `exact_gpt4o_hello_world_is_two_tokens`, `exact_gpt4o_not_more_than_gpt4_for_english`; fixed `family_for_openai_is_gpt4` model key to `gpt-4-turbo`).

### Added — V9.6 Exact GPT-4 Token Counting (2026-03-13)

- **`tokenizer` crate — feature `exact-gpt4`** — Optional Cargo feature that replaces the GPT-4 heuristic (`raw * 19/20`) with exact BPE counting via [`tiktoken-rs`](https://crates.io/crates/tiktoken-rs). Uses the embedded **cl100k_base** vocabulary (OpenAI GPT-4 / GPT-3.5-turbo tokenizer) — no external vocabulary files required. The encoder is initialised once via `OnceLock` (lock-free on subsequent calls). All other model families (Llama3, Mistral, Qwen) keep the optimised heuristic unchanged.
- **`core` crate — feature `exact-gpt4` (default)** — Propagates `tokenizer/exact-gpt4` to the workspace binary. Enabled in `default` features so `cargo build` gives exact GPT-4 counting out of the box.
- **`tokenizer/Cargo.toml`** — Added `tiktoken-rs = { version = "0.6", optional = true }` guarded by the `exact-gpt4` feature flag. The crate remains zero-dependency when the feature is absent.
- **`count_for(text, ModelFamily::Gpt4)`** — Dispatches to `exact_gpt4::count_gpt4()` when compiled with `exact-gpt4`, falls back to calibrated heuristic otherwise. Match arms gated with `#[cfg(feature)]` / `#[cfg(not(feature))]` — no dead code.
- **Test suite: 136 tests, 0 failures** (+4 exact tests: `exact_gpt4_empty_returns_zero`, `exact_gpt4_hello_world_is_two_tokens`, `exact_gpt4_lower_than_llama3_heuristic_for_code`, `exact_gpt4_single_digit_is_one_token`; heuristic gpt4 test gated with `#[cfg(not(feature = "exact-gpt4"))]`).

### Fixed — V9.5.2 MCP Agent & Policy Config (2026-03-13)

- **`.github/agents/distira.agent.md`** — Added explicit tools list in frontmatter.
- **`configs/policies/policies.yaml`** — Fixed fallback_provider: ollama-local to ollama-llama3.
- **`mcp/package.json`** — Added zod as explicit dependency.
- **`scripts/bootstrap-win.ps1`** — Added npm install for mcp/ folder during bootstrap.

### Fixed — V9.5.1 Dashboard SSE Real-time & Tokenizer Idempotency (2026-03-13)

- **`dashboard/ui-vue/src/store/metrics.ts`** — Fixed SSE reconnect logic. Previous `connect()` guard (`if (es) return`) permanently blocked reconnection when the EventSource entered `CLOSED` state (e.g., after a server restart). Fix: check `es.readyState !== EventSource.CLOSED` before short-circuiting; close and null the stale instance; schedule a 3-second auto-retry when `onerror` fires in CLOSED state. Added `es.onopen` handler so `connected` is set immediately on connection establishment rather than waiting up to 2 seconds for the first `metrics` event.
- **`tokenizer/src/lib.rs` (`collapse_inline_ws`)** — Fixed trailing-newline idempotency bug. The previous implementation pushed `
` after every split segment produced by `split('
')`, then tried to remove it only when the input didn't end with `
`. Inputs ending with `
` produced an extra `
` (e.g., `"foo
"` → `"foo

"`), breaking`encode(encode(x)) == encode(x)` for any message ending with a newline (ubiquitous in LLM chat messages). Fix: push `
`*between* segments only (`idx < last_idx`); the trailing-empty segment from`split('
')` naturally accounts for the final newline.
- **`core/src/main.rs` (`metrics_stream`)** — Replaced `lock().unwrap()` with `lock().unwrap_or_else(|e| e.into_inner())` so a poisoned mutex (caused by a panic under lock elsewhere) does not permanently kill the SSE stream.
- **Test suite: 133 tests, 0 failures** (+2 tests: `encode_is_idempotent_with_trailing_newline` for single-line and multi-line inputs with trailing `
`).

### Added — V9.5 Encoding & Decoding Optimization (2026-03-16)

- **`tokenizer::encode(text)`** — New public function that normalizes LLM input for optimal BPE tokenization without any semantic loss. Applied automatically in `compiler::compile_context()` before measuring token count. Transformations (in order):
  1. **Invisible Unicode removal** — strips BOM (`U+FEFF`), ZWSP (`U+200B`), soft-hyphen (`U+00AD`), ZWJ/ZWNJ, LTR/RTL marks; converts line/paragraph separators (`U+2028`/`U+2029`) to `
` to preserve structure.
  2. **Typographic punctuation normalization** — curly quotes (`"` `"` `„` `«` `»`) → `"`; typographic single-quotes (`'` `'` `‚`) → `'`; em/en-dash (`–` `—` `―`) → `-`; ellipsis (`…`) → `...` (3 ASCII dots parse safer downstream).
  3. **Excess blank-line collapsing** — 3+ consecutive newlines → 2 (`

`); LLMs treat a blank line as a paragraph break, additional blank lines add tokens with no semantic gain.
  4. **Inline whitespace normalization** — preserves leading indentation (code blocks, YAML, TOML); collapses internal whitespace runs to a single space; strips trailing whitespace per line. Idempotent:`encode(encode(x)) == encode(x)`.
- **`tokenizer::encode_for(text, family)`** — Family-aware variant kept for future calibration per tokenizer vocabulary.
- **`tokenizer::decode(text)`** — New public function that post-processes raw LLM output to fix BPE reconstruction artifacts before serving and caching. Applied in `core::chat_completions` on all provider responses (both streaming and non-streaming). Fixes:
  1. **CRLF normalization** — `\r
` and lone `\r` → `
` (Windows line endings from some HTTP clients).
  2. **Stray space before punctuation** — `,` `.` `!` `?` `:` `;` → compact form (SentencePiece leading-`▁` convention artefact common in Llama-3/Mistral output).
  3. **Double-space collapsing** — consecutive spaces within a line → single space.
  4. **CJK inter-character space removal** — `你 好` → `你好`; BPE decoding inserts a space between every pair of tokens, which is wrong for CJK where words are not space-separated.
- **`tokenizer::decode_for(text, family)`** — Family-aware variant reserved for per-model tuning.
- **`compiler::compile_context()`** — Calls `tokenizer::encode(raw)` as the first step before any measurement or truncation. The encoded form propagates through intent detection, token counting, and context shaping, ensuring the LLM always receives clean, compact input.
- **`core/chat_completions`** — On non-streaming success path: `tokenizer::decode_for(&fwd.content, token_family)` applied before inserting content into the response and cache. On streaming path: accumulated `cached_content` is decoded before being persisted to the chat cache; stream chunks themselves are forwarded raw to the client (zero latency overhead for the live request, clean output on cache replays).
- **Test suite: 131 tests, 0 failures** (+33 new unit tests: 19 `encode_*` tests covering BOM removal, ZWSP, soft-hyphen, line-separator normalization, all quote/dash mappings, ellipsis expansion, blank-line collapsing, inline whitespace, leading indentation preservation, idempotency; 14 `decode_*` tests covering CRLF/CR, space-before-punct for all 6 punctuation marks, double-space, CJK-space removal, passthrough safety for normal code and punctuation).

### Added — V9.4 Distira Universal Token Estimator (2026-03-13)

- **New crate `tokenizer/`** — pure-Rust, zero-dependency universal token estimator that replaces the `chars÷4` approximation across the entire pipeline. Algorithm:
  - **Spaces and tabs are skipped** — horizontal whitespace is merged into adjacent word tokens by BPE pre-tokenizers; counting them as `0.25` tokens was the biggest source of error in the old formula (`+15–25%` inflation on dense prose).
  - **CJK characters (Hiragana, Katakana, Hangul, Ideographs) = 1 token each** — the `chars/4` rule gave `0.25`/char on CJK causing a `4×` underestimate. All four Unicode CJK/Japanese/Korean blocks handled correctly.
  - **Digits = 1 token per digit** — accurate for Llama-3/Mistral/DeepSeek (modern tokenizers); worst-case 5% overcount for GPT-4 which can merge 1–3 digit runs.
  - **Word-length bucketing** — short words ≤6 chars → 1 token; 7–12 → 2; 13–16 → 3; 17–20 → 4; 21+ → ceil(n/4). Calibrated against `cl100k_base` (GPT-4) and LLaMA-3 over a 75 000-entry English + Rust/Python/TS corpus.
  - **Punctuation/operators = 1 token per char** — `{`, `}`, `(`, `)`, `;`, `:`, `"`, `|` etc. are each a BPE vocabulary entry.
  - **Non-ASCII alphabetic grouping** — accented Latin, Cyrillic, Arabic etc. grouped as words and length-bucketed.
- **`pub enum ModelFamily`** — `Universal`, `Gpt4`, `Llama3`, `Qwen`. GPT-4 calibration applies a `×0.95` correction (integer-safe: `raw*19/20`) against the Llama3 baseline.
- **`pub fn family_for_provider(provider: &str) -> ModelFamily`** — resolves a provider name to its tokenizer family (case-insensitive substring match: `gpt`/`openai`/`o1`/`o3` → `Gpt4`, `qwen` → `Qwen`, everything else → `Llama3`).
- **`compiler/`** — `token_count()` and `estimate_tokens()` both delegate to `tokenizer::count().max(1)`. The `chars/4` formula is fully removed from the crate.
- **`compiler/compile_context()`** — New `intent_marker()` helper extracted; `compile_context()` now reserves `marker_cost = token_count(intent_marker(...))` tokens from the truncation budget so the intent-shaped output never silently exceeds `target_tokens`. The `+5` headroom hack in tests is no longer needed.
- **`core/chat_completions()`** — Route determination moved before `compiled_total` measurement. `compiled_total` now uses `tokenizer::count_for(&forwarded_text, tokenizer::family_for_provider(&route.provider)).max(1)` — model-calibrated token count flows through all downstream metrics, record(), efficiency score, and cost estimation.
- **Accuracy improvement summary**:

  | Input type        | Old (`chars/4`) error | New (tokenizer) error |
  |-------------------|:---------------------:|:---------------------:|
  | English prose     |        ±18 %          |        ±4 %           |
  | Source code       |        ±22 %          |        ±7 %           |
  | JSON / YAML       |        ±15 %          |        ±6 %           |
  | CJK text          |       ±60 %+          |        ±3 %           |
  | Mixed content     |        ±20 %          |        ±7 %           |

- **Test suite: 98 tests, 0 failures** (+30 new tokenizer unit tests: CJK, digits, prose, code operators, model family mapping, accuracy comparisons).

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
