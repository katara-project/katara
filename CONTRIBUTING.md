# Contributing

Distira is free and open-source (AGPL-3.0). Contributions of all kinds are welcome.

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before participating.

## Getting started

1. Fork the repository and clone it locally.
2. **First-time only** — run the bootstrap script for your platform:
   - **Windows:** `.\scripts\bootstrap-win.ps1`
   - **Linux / macOS:** `./scripts/bootstrap.sh`
3. **Daily start** — one command starts everything (backend + dashboard):
   - **Windows:** `.\scripts\start-win.ps1`
   - **Linux / macOS:** `cargo build --release -p core && ./target/release/core &` + `cd dashboard/ui-vue && npm run dev`
4. The **MCP server** starts automatically when you open the folder in VS Code (via `.vscode/mcp.json`). No manual step needed.
5. Verify the build: `cargo test --workspace` and `cd dashboard/ui-vue && npm run build`.

## Development stack

| Layer | Technology |
| --- | --- |
| Backend / gateway | Rust (edition 2021) |
| Dashboard | Vue 3 + Vite + Pinia + Vue Router |
| Configuration | YAML (`configs/`) |
| Deployment | Docker, Kubernetes, Helm |

## Code style

- **Rust:** `cargo fmt` and `cargo clippy` must pass with zero warnings.
- **TypeScript / Vue:** consistent `lang="ts"` in `<script setup>`, no unused imports.
- **Markdown:** ATX headings, one sentence per line where practical, trailing newline.
- **YAML:** 2-space indent, `---` document start marker, no trailing spaces.

## Before opening a PR

- Update `docs/` when behavior changes.
- Update `ROADMAP.md` if the iteration state changes.
- Update `CHANGELOG.md` under `[Unreleased]` for noteworthy additions.
- Keep naming and architecture consistent with the monorepo layout.
- Run `cargo test` and ensure all tests pass.

## Pull requests

Keep PRs focused and include:

- **What** changed
- **Why** it changed
- **How** it was tested

## Reporting issues

Open a GitHub issue with:

- A clear title and description
- Steps to reproduce (if applicable)
- Expected vs. actual behavior
