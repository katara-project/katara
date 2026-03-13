# Installation & Configuration Guide

> **DISTIRA v7.7.1** — Sovereign AI Context Operating System

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Clone & Bootstrap](#clone--bootstrap)
3. [Configure Providers](#configure-providers)
4. [Configure Routing](#configure-routing)
5. [Configure Policies](#configure-policies)
6. [Run DISTIRA](#run-distira)
7. [Test Your Setup](#test-your-setup)
8. [Dashboard](#dashboard)
9. [Ollama Multi-Model Setup](#ollama-multi-model-setup)
10. [Cloud Providers (Optional)](#cloud-providers-optional)
11. [Docker](#docker)
12. [Kubernetes / Helm](#kubernetes--helm)
13. [Troubleshooting](#troubleshooting)

---

## Prerequisites

| Tool        | Minimum Version | Install                          |
|-------------|-----------------|----------------------------------|
| **Rust**    | 1.75+           | https://rustup.rs                |
| **Node.js** | 20+             | https://nodejs.org               |
| **Git**     | 2.x             | https://git-scm.com              |
| **Ollama**  | 0.3+ (optional) | https://ollama.com/download      |

Verify:

```bash
rustc --version    # rustc 1.75.0 or higher
node --version     # v20.x or higher
ollama --version   # ollama version 0.3.x (if using local models)
```

---

## Clone & Bootstrap

```bash
git clone https://github.com/distira-project/distira.git
cd distira
```

### Automatic (recommended)

**Windows (PowerShell):**

```powershell
.\scripts\bootstrap-win.ps1
```

**Linux / macOS:**

```bash
./scripts/bootstrap.sh
```

### Manual

```bash
# Build all 8 Rust crates
cargo build --workspace

# Install dashboard dependencies
cd dashboard/ui-vue
npm install
cd ../..
```

---

## Configure Providers

Edit `configs/providers/providers.yaml` to declare every LLM endpoint DISTIRA can reach.

### Minimal example (Ollama only)

```yaml
providers:
  ollama-llama3:
    type: openai-compatible
    base_url: http://localhost:11434/v1
    model: llama3.1
    deployment: on-prem
    description: Llama 3.1 local
```

### Multi-model Ollama

Each Ollama model gets its own provider entry. They all share the same `base_url`
but have different `model` values:

```yaml
providers:
  ollama-llama3:
    type: openai-compatible
    base_url: http://localhost:11434/v1
    model: llama3.1
    deployment: on-prem
    description: General tasks

  ollama-codellama:
    type: openai-compatible
    base_url: http://localhost:11434/v1
    model: codellama
    deployment: on-prem
    description: Code generation

  ollama-mistral:
    type: openai-compatible
    base_url: http://localhost:11434/v1
    model: mistral
    deployment: on-prem
    description: Debug and analysis
```

### Cloud provider (optional)

```yaml
  openai-cloud:
    type: openai-compatible
    base_url: https://api.openai.com/v1
    model: gpt-4o-mini
    deployment: cloud
    api_key_env: OPENAI_API_KEY        # reads from environment variable
    description: OpenAI cloud fallback
```

### Provider fields reference

| Field           | Required | Description                                              |
|-----------------|----------|----------------------------------------------------------|
| `type`          | no       | Adapter style (`openai-compatible`, `mistral`, `google`) |
| `base_url`      | **yes**  | API root URL. For Ollama: `http://localhost:11434/v1`    |
| `model`         | **yes**  | Model name as the provider expects it                    |
| `deployment`    | no       | `on-prem` or `cloud` (informational)                     |
| `description`   | no       | Human-readable note                                      |
| `api_key_env`   | no       | Name of the env var holding the API key                  |

---

## Configure Routing

Edit `configs/routing/routing.yaml` to tell DISTIRA which provider to use for each intent.

```yaml
routing:
  strategy: hybrid
  default_provider: ollama-llama3       # used when no task_routing matches
  fallback_provider: ollama-llama3      # used when configured provider is missing
  sensitive_override: ollama-llama3     # forced for sensitive: true requests

  task_routing:
    debug: ollama-mistral               # intent "debug" → Mistral 7B
    summarize: ollama-llama3            # intent "summarize" → Llama 3.1
    review: ollama-codellama            # intent "review" → CodeLlama
    general: ollama-llama3              # intent "general" → Llama 3.1
```

### Routing fields reference

| Field                | Description                                                       |
|----------------------|-------------------------------------------------------------------|
| `default_provider`   | Provider for unrecognized intents                                 |
| `fallback_provider`  | Used if the selected provider name is missing from providers.yaml |
| `sensitive_override` | All requests with `"sensitive": true` go here (sovereign)         |
| `task_routing.*`     | Maps intent → provider name (from providers.yaml keys)            |

### How intent detection works

DISTIRA automatically detects intent from the raw context:

| Detected keyword       | Intent      |
|------------------------|-------------|
| `error`, `trace`       | `debug`     |
| `summar`               | `summarize` |
| `diff`, `pull request` | `review`    |
| `anything else`        | `general`   |

---

## Configure Policies

Edit `configs/policies/policies.yaml` to set data handling rules:

```yaml
policies:
  sensitive_data: local_only            # sensitive data never leaves local
  optimize_for: cost                    # cost | latency | quality
  max_tokens_per_request: 4000          # safety limit per request
  data_residency: eu                    # for GDPR compliance
  log_level: info                       # debug | info | warn | error
```

### Runtime Audit retention (recommended production defaults)

To avoid keeping unnecessary audit data in memory, configure retention with these environment variables:

```bash
DISTIRA_AUDIT_RETENTION_DAYS=7
DISTIRA_AUDIT_HISTORY_LIMIT=2000
```

- `DISTIRA_AUDIT_RETENTION_DAYS`: time-based retention window for Runtime Audit entries.
- `DISTIRA_AUDIT_HISTORY_LIMIT`: max number of Runtime Audit entries kept in memory.

If both are set, DISTIRA applies both guards: entries older than the retention window are pruned, and the remaining history is capped to the configured limit.

### Runtime state persistence

DISTIRA persists runtime operational state automatically and restores it on startup. By default, the snapshot is stored at:

```bash
cache/runtime-state.json
```

You can override the location with:

```bash
DISTIRA_RUNTIME_STATE_PATH=/absolute/path/to/runtime-state.json
```

Persisted state includes metrics snapshot, runtime audit history, semantic/chat caches, and context-store blocks.

---

## Configure Workspace Scope (Tenant / Project)

V7.7 introduces workspace-level scope foundations for future per-tenant routing and policy packs.

Create or edit `configs/workspace/workspace.yaml`:

```yaml
---
workspace:
  tenant_id: "default-tenant"
  project_id: "distira-platform"
  policy_pack: "baseline"
```

You can also override scope per request via:

- `POST /v1/runtime/client-context` with `tenant_id` and `project_id`
- `POST /v1/compile` and `POST /v1/chat/completions` payloads with `tenant_id` and `project_id`

Resolution order is: request payload > runtime client-context > workspace defaults.

---

## Run DISTIRA

### Automatic (recommended)

Start **all services** (Ollama + backend + dashboard) with a single command:

**Windows PowerShell:**

```powershell
.\scripts\start-win.ps1
```

**Linux / macOS:**

```bash
./scripts/start.sh
```

This will:
1. Load `.env` secrets
2. Start Ollama (or detect it's already running)
3. Start the DISTIRA Rust backend on `:8080`
4. Start the Vue dashboard on `:5173`
5. Stream backend logs — press **Ctrl+C** to stop everything

### Manual (step by step)

#### Step 1: Start Ollama (if using local models)

```bash
ollama serve
```

Pull the models you declared in providers.yaml:

```bash
ollama pull llama3:latest          # general, summarize, default
ollama pull qwen2.5-coder:7b       # code review
ollama pull mistral:7b-instruct    # debug, analysis
```

Verify Ollama is running:

```bash
curl http://localhost:11434/v1/models
```

#### Step 2: Load secrets and start DISTIRA backend

**Linux / macOS:**

```bash
export $(grep -v '^#' .env | xargs)
cargo run -p core
```

**Windows PowerShell:**

```powershell
Get-Content .env | ForEach-Object {
  if ($_ -match '^([^#=]+)=(.*)$') {
    [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
  }
}
cargo run -p core
```

You will see:

```md
DISTIRA v7.7.1 — Sovereign AI Context OS
────────────────────────────────────────
  Config loaded from configs/
    provider: ollama-llama3
    provider: ollama-codellama
    provider: ollama-mistral
────────────────────────────────────────
Listening on 127.0.0.1:8080
  POST /v1/compile            — compile context only
  POST /v1/chat/completions   — compile + forward to LLM
  GET  /v1/providers          — list configured providers
  GET  /v1/metrics            — JSON snapshot
  GET  /v1/metrics/stream     — SSE live stream
```

#### Step 3: Start the dashboard

```bash
cd dashboard/ui-vue
npm run dev
```

Open http://localhost:5173 — the green **Live** dot confirms SSE connection.

---

## Test Your Setup

For the full test suite (intent routing, cache, MCP agent, dashboard), see **[TESTING.md](TESTING.md)**.

Quick smoke tests below:

### Health check

```bash
curl http://localhost:8080/healthz
# {"status":"ok","service":"distira-core","version":"7.7.1"}
```

### List providers

```bash
curl http://localhost:8080/v1/providers
# {"providers":["ollama-llama3","ollama-qwen2.5-coder","ollama-mistral","ollama-ocr-deepseek","mistral-ocr-cloud"]}
```

### Compile only (no LLM call)

```bash
curl -X POST http://localhost:8080/v1/compile \
  -H "Content-Type: application/json" \
  -d '{"context": "Debug this auth function with retry logic", "sensitive": false}'
```

Response includes `provider`, `model`, `intent`, `compiled_tokens`, `cache_hit`, `routing_reason`.

**Expected:** `intent="debug"`, `provider="ollama-mistral"`

### Full LLM call (compile + forward)

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [{"role": "user", "content": "Explain the circuit breaker pattern"}]
  }'
```

This will:
1. Detect intent → `general`
2. Route to `ollama-llama3` (`llama3:latest`)
3. Forward to Ollama on localhost:11434
4. Return OpenAI-compatible response with a `distira` section showing optimization stats

### Force sensitive routing

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [{"role": "user", "content": "Analyze this patient record"}],
    "sensitive": true
  }'
```

This forces the request to stay on `sensitive_override` (local Ollama), regardless of intent.

### Override model

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "codellama",
    "messages": [{"role": "user", "content": "Write a Rust TCP server"}]
  }'
```

Setting `model` explicitly bypasses task routing and uses the specified model.

---

## Ollama Multi-Model Setup

### Quick reference

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh    # Linux
# or download from https://ollama.com/download    # Windows/macOS

# Start Ollama
ollama serve

# Pull models (matching providers.yaml)
ollama pull llama3:latest          # general, summarize, default
ollama pull qwen2.5-coder:7b       # code review
ollama pull mistral:7b-instruct    # debug, analysis

# Verify
ollama list
```

### Map models to DISTIRA providers

Each model you pull in Ollama must have a corresponding entry in `configs/providers/providers.yaml`:

| Ollama model | providers.yaml key | Suggested routing |
|---|---|---|
| `llama3:latest` | `ollama-llama3` | default, general, summarize |
| `qwen2.5-coder:7b` | `ollama-qwen2.5-coder` | review (code) |
| `mistral:7b-instruct` | `ollama-mistral` | debug, analysis |

---

## Cloud Providers (Optional)

### OpenAI

1. Get an API key from https://platform.openai.com/api-keys
2. Set the environment variable:

```bash
export OPENAI_API_KEY="sk-..."       # Linux/macOS
$env:OPENAI_API_KEY = "sk-..."       # Windows PowerShell
```

3. Add to providers.yaml:

```yaml
  openai-cloud:
    type: openai-compatible
    base_url: https://api.openai.com/v1
    model: gpt-4o-mini
    deployment: cloud
    api_key_env: OPENAI_API_KEY
```

4. Add to routing.yaml (as fallback):

```yaml
  fallback_provider: openai-cloud
```

### Mistral AI

1. Copy the env template and fill in your key:

```bash
cp .env.example .env
# Edit .env:
# MISTRAL_API_KEY=your-key-here
```

2. On Windows PowerShell, load the `.env` before starting DISTIRA:

```powershell
Get-Content .env | ForEach-Object {
  if ($_ -match '^([^#=]+)=(.*)$') {
    [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
  }
}
cargo run -p core
```

On Linux / macOS:

```bash
export $(grep -v '^#' .env | xargs)
cargo run -p core
```

> **Note:** `.env` is listed in `.gitignore` — your keys will **never** be pushed to GitHub.
> Only `.env.example` (with placeholder values) is committed.

3. Provider config in `configs/providers/providers.yaml`:

```yaml
  mistral-cloud:
    type: mistral
    base_url: https://api.mistral.ai/v1
    model: mistral-ocr-2512
    deployment: cloud
    api_key_env: MISTRAL_API_KEY
    description: Mistral OCR 2512 cloud endpoint
```

---

## Docker

```bash
# Build
docker build -f deployments/docker/Dockerfile -t distira/core:7.7.1 .

# Run (with config mounted)
docker run -p 8080:8080 \
  -v $(pwd)/configs:/app/configs:ro \
  -e OPENAI_API_KEY="sk-..." \
  distira/core:7.7.1
```

For Ollama running on the host:

```bash
docker run -p 8080:8080 \
  -v $(pwd)/configs:/app/configs:ro \
  --add-host host.docker.internal:host-gateway \
  distira/core:7.7.1
```

Then change `base_url` in providers.yaml to `http://host.docker.internal:11434/v1`.

---

## Kubernetes / Helm

```bash
# Direct deployment
kubectl apply -f deployments/kubernetes/deployment.yaml

# Or via Helm
helm install distira deployments/helm/
```

Create a ConfigMap for your YAML configs:

```bash
kubectl create configmap distira-config \
  --from-file=configs/providers/providers.yaml \
  --from-file=configs/routing/routing.yaml \
  --from-file=configs/policies/policies.yaml
```

---

## Architecture

```md
┌─────────────────────┐         POST /v1/chat/completions   ┌──────────────────────────┐
│   Client (curl,     │ ──────────────────────────────────► │   DISTIRA Rust Backend    │
│   VS Code ext,      │                                     │                          │
│   any AI tool)      │ ◄────── OpenAI-compatible JSON ──── │  ① fingerprint           │
└─────────────────────┘                                     │  ② cache lookup          │
                                                            │  ③ context compiler      │
┌─────────────────────┐                                     │  ④ memory lensing        │
│   Vue Dashboard     │ ◄── SSE /v1/metrics/stream ──────── │  ⑤ intent → route        │
│   (Pinia + SSE)     │     every 2 seconds                 │  ⑥ forward to LLM        │
└─────────────────────┘                                     └──────┬───────────────────┘
                                                                   │
                                                            ┌──────┴────────────────────┐
                                                            │  Ollama (localhost:11434) │
                                                            │   llama3.1 / codellama    │
                                                            │   mistral / phi3 / ...    │
                                                            │                           │
                                                            │  Cloud (optional)         │
                                                            │   OpenAI / Mistral AI     │
                                                            └───────────────────────────┘
```

---

## VS Code Agent (MCP)

DISTIRA includes an MCP (Model Context Protocol) server that integrates with VS Code Copilot Chat.
It uses `@modelcontextprotocol/sdk` v1.27.1 with `StdioServerTransport` (JSON-RPC 2.0 over stdio).

### Requires

- VS Code with GitHub Copilot Chat extension
- Node.js 20+
- DISTIRA backend running on port 8080

### Setup

1. **Install MCP dependencies** (first-time only):

```powershell
Set-Location mcp
npm install
Set-Location ..
```

2. The MCP server is already registered in `.vscode/mcp.json` with the correct `cwd`. VS Code detects it automatically on startup.

3. Start the DISTIRA backend: `cargo run -p core`

4. In Copilot Chat, type `@distira` followed by your request.

### How it works

```text
VS Code Copilot Chat
  │
  └─ spawns: node distira-server.mjs (cwd: mcp/)
              │
              └─ stdio JSON-RPC 2.0 (Content-Length framing)
                   │
                   └─ HTTP → http://127.0.0.1:8080
```

### Available tools

| Tool | Description | Example |
|------|-------------|---------|
| `distira_compile` | Compile raw context (no LLM call) | `@distira compile this error trace` |
| `distira_chat` | Compile + forward to LLM | `@distira explain circuit breaker pattern` |
| `distira_set_client_context` | Update live upstream model/provider context | `@distira set client context to Claude Sonnet 4.6 on Anthropic` |
| `distira_providers` | List configured providers | `@distira list providers` |
| `distira_metrics` | Fetch live metrics snapshot | `@distira show metrics` |

### Upstream model lineage

The MCP server automatically forwards upstream client metadata to DISTIRA so the dashboard can distinguish:

- the assistant or client model selected by the user
- the model actually routed by DISTIRA

Default behavior:

- `client_app` → `VS Code Copilot Chat`
- `upstream_model` → the `model` argument passed to `distira_chat`, MCP request `_meta`, or a runtime resolver command
- `upstream_provider` → MCP request `_meta`, runtime resolver command, or inferred from the model family when possible

Optional environment overrides:

```text
DISTIRA_CLIENT_APP=VS Code Copilot Chat
DISTIRA_UPSTREAM_PROVIDER=GitHub Copilot
DISTIRA_UPSTREAM_MODEL=GPT-5.4
```

These static environment variables are only fallbacks. For dynamic behavior, prefer a runtime resolver command that is evaluated on every request:

```text
DISTIRA_CLIENT_CONTEXT_CMD=powershell -File ..\scripts\resolve-upstream-context.ps1
```

Expected command output:

```json
{
  "client_app": "VS Code Copilot Chat",
  "upstream_provider": "Anthropic",
  "upstream_model": "Claude Sonnet 4.6"
}
```

DISTIRA also accepts request metadata keys when the MCP client can send them:

- `distira/client_app`
- `distira/upstream_provider`
- `distira/upstream_model`

This is the dynamic path. If the upstream client changes model from one request to another and exposes that value, DISTIRA will reflect it immediately without restart.

If the client does not expose it directly, you can still update the live runtime context without restart:

```powershell
.\scripts\set-upstream-context.ps1 -UpstreamProvider "Anthropic" -UpstreamModel "Claude Sonnet 4.6"
```

DISTIRA also exposes `GET/POST /v1/runtime/client-context` for programmatic updates.

### Validate the MCP integration

See [TESTING.md — MCP Agent Tests](TESTING.md#mcp-agent-tests-vs-code) for step-by-step validation.

### Manual test (without VS Code)

```powershell
Set-Location mcp
node distira-server.mjs
# Send JSON-RPC 2.0 initialize + tools/list messages via stdin
```

---

## Google Drive Workspace

If your workspace is stored on Google Drive (e.g. via Google Drive File Stream), the Rust `target/` folder will cause **file-locking errors** during compilation.

The project includes `.cargo/config.toml` which redirects build output to `C:/distira-target` (local disk). This is applied automatically — no manual action needed.

To change the path, edit `.cargo/config.toml`:

```toml
[build]
target-dir = "C:/distira-target"
```

---

## Troubleshooting

| Problem | Solution |
|---|---|
| `No config files found — using built-in defaults` | Run DISTIRA from the repo root: `cargo run -p core` |
| `Provider returned 404` | Check that the model is pulled in Ollama: `ollama list` |
| `HTTP error: Connection refused` | Ollama not running. Start with `ollama serve` |
| `Provider returned 401` | API key missing. Set the env var listed in `api_key_env` |
| Dashboard shows **Offline** | Backend not running on port 8080, or CORS blocked |
| npm install fails on Google Drive | Copy to local disk first: `robocopy . $env:TEMP/distira /MIR` |
| Rust build fails (MSVC not found) | Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" |

---

## Endpoints Reference

| Method | Path | Description |
|---|---|---|
| `GET` | `/healthz` | Health check |
| `GET` | `/version` | Version info |
| `GET` | `/v1/providers` | List configured providers |
| `POST` | `/v1/compile` | Compile context (no LLM call) |
| `POST` | `/v1/chat/completions` | Compile + forward to LLM |
| `GET` | `/v1/metrics` | Metrics JSON snapshot |
| `GET` | `/v1/metrics/stream` | SSE live metrics stream |

---

*License: AGPL-3.0 — Free and open-source. Copyright 2024-2026 Christophe Freijanes*
