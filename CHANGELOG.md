# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Moved `workflows/` to `.github/workflows/` for GitHub Actions compatibility
- Fixed README bootstrap script reference (`bootstrap-win.ps1`)
- Hardened `.gitignore` (added `.env`, `*.zip`, `Thumbs.db`)
- Added `---` document start markers to all YAML configs
- Expanded all documentation to canonical standards
- Added unit tests to every Rust crate
- Replaced `|| true` guards in CI with proper error handling
- Added Dockerfile `EXPOSE`, healthcheck, and non-root user
- Converted bootstrap scripts from echo-only to functional installers

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
