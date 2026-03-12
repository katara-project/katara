# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`TESTING.md`**: dedicated test guide covering smoke tests, intent routing matrix, MCP agent tests, PowerShell quick-test script, expected responses reference, and common errors
- **`scripts/test-api.ps1`**: PowerShell quick-test script (7 assertions covering health, providers, intents, sensitive mode, metrics)
- **`mcp/test-mcp.mjs`**: standalone MCP handshake test harness (spawns server, sends `initialize` + `tools/list`, validates responses without VS Code)

### Improved

- License: Apache 2.0 → AGPL-3.0 + Commons Clause (protection against unauthorized commercial resale)
- `.gitignore`: expanded to 70+ rules covering Rust, Node, IDE, secrets, IaC, Docker, OS artifacts
- `policies.yaml`: added `terms` property and `fallback_provider`, `log_level`, `data_residency` fields
- `mcp/katara-server.mjs`: migrated from custom Buffer-based stdio transport to official `@modelcontextprotocol/sdk` v1.27.1 (`McpServer` + `StdioServerTransport` + Zod tool schemas) — eliminates Windows stdin hang
- `.vscode/mcp.json`: added `"cwd": "${workspaceFolder}/mcp"` so Node resolves SDK imports from `mcp/node_modules/`
- `scripts/start-win.ps1`: replaced `Get-NetTCPConnection` (unreliable) with `netstat -ano` for port 8080 process detection; passes `$cargoPath` explicitly to `Start-Job` to avoid PATH inheritance failures
- `scripts/bootstrap-win.ps1`: adds `~\.cargo\bin` to PATH before `Get-Command cargo` check to detect freshly installed Rust

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
