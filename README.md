# KATARA

> **The Sovereign AI Flow Engine** — compile the smallest useful context before every LLM call.

[![CI](https://github.com/katara-project/katara/actions/workflows/ci.yml/badge.svg)](https://github.com/katara-project/katara/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-7.0.0-brightgreen.svg)](VERSION)

## What makes KATARA different

Most AI gateways route requests. KATARA goes further:

| Feature | Description |
| --- | --- |
| **Context Budget Compiler** | Reduces raw prompts, logs, diffs, and transcripts before model invocation |
| **Context Memory Lensing** | Reuses stable context blocks and sends only deltas when possible |
| **AI Flow Visualizer** | Makes every optimization step visible in a live dark dashboard |
| **Hybrid Sovereign Routing** | Routes intelligently across local, private, and cloud LLMs |
| **AI Efficiency Score** | Quantifies token savings, cost reduction, and context reuse |

## Architecture

```text
Clients / IDE / Agents
        │
  OpenAI-compatible API
        │
      KATARA
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

## Monorepo layout

| Directory | Purpose |
| --- | --- |
| `core/` | Gateway bootstrap and API entry point |
| `compiler/` | Context Budget Compiler and reducers |
| `memory/` | Context Memory Lensing and delta engine |
| `router/` | Provider selection and routing strategies |
| `adapters/` | Provider-specific HTTP clients |
| `metrics/` | Efficiency scoring and telemetry |
| `cache/` | Semantic cache scaffolding |
| `fingerprint/` | Prompt fingerprint graph |
| `dashboard/ui-vue/` | Vue 3 + Vite dark dashboard |
| `configs/` | Provider, routing, and policy configuration |
| `deployments/` | Docker, Kubernetes, and Helm manifests |
| `docs/` | Architecture and implementation notes |
| `examples/` | Quick integration examples |
| `benchmarks/` | Reproducible token-reduction fixtures |

## Quick start

### Windows

```powershell
.\scripts\bootstrap-win.ps1
```

### Linux / macOS

```bash
./scripts/bootstrap.sh
```

### Manual

```bash
# Rust backend
cargo build

# Vue dashboard
cd dashboard/ui-vue && npm install && npm run dev
```

## Version

Current scaffold version: **7.0.0**

See [CHANGELOG.md](CHANGELOG.md) for release history and [ROADMAP.md](ROADMAP.md) for planned iterations.

## Status

This is a **V7 advanced scaffold**: coherent, GitHub-ready, and implementation-oriented.
It is not yet a fully production-complete gateway across every provider.

## License

[Apache 2.0](LICENSE) — Copyright 2026 KATARA contributors.
