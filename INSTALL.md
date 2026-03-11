# Installation & Configuration Guide

> **KATARA v7.0.0** — Sovereign AI Context Operating System

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Clone & Bootstrap](#clone--bootstrap)
3. [Configure Providers](#configure-providers)
4. [Configure Routing](#configure-routing)
5. [Configure Policies](#configure-policies)
6. [Run KATARA](#run-katara)
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
git clone https://github.com/katara-project/katara.git
cd katara
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

Edit `configs/providers/providers.yaml` to declare every LLM endpoint KATARA can reach.

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

| Field           | Required | Description                                         |
|-----------------|----------|-----------------------------------------------------|
| `type`          | no       | Adapter style (`openai-compatible`, `mistral`, `google`) |
| `base_url`      | **yes**  | API root URL. For Ollama: `http://localhost:11434/v1` |
| `model`         | **yes**  | Model name as the provider expects it                |
| `deployment`    | no       | `on-prem` or `cloud` (informational)                |
| `description`   | no       | Human-readable note                                  |
| `api_key_env`   | no       | Name of the env var holding the API key              |

---

## Configure Routing

Edit `configs/routing/routing.yaml` to tell KATARA which provider to use for each intent.

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

| Field               | Description                                                   |
|----------------------|---------------------------------------------------------------|
| `default_provider`   | Provider for unrecognized intents                            |
| `fallback_provider`  | Used if the selected provider name is missing from providers.yaml |
| `sensitive_override` | All requests with `"sensitive": true` go here (sovereign)     |
| `task_routing.*`     | Maps intent → provider name (from providers.yaml keys)        |

### How intent detection works

KATARA automatically detects intent from the raw context:

| Detected keyword      | Intent       |
|------------------------|-------------|
| `error`, `trace`       | `debug`     |
| `summar`               | `summarize` |
| `diff`, `pull request` | `review`    |
| anything else          | `general`   |

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

---

## Run KATARA

### Step 1: Start Ollama (if using local models)

```bash
ollama serve
```

Pull the models you declared in providers.yaml:

```bash
ollama pull llama3.1
ollama pull codellama
ollama pull mistral
```

Verify Ollama is running:

```bash
curl http://localhost:11434/v1/models
```

### Step 2: Start KATARA backend

```bash
cargo run -p core
```

You will see:

```md
KATARA v7.0.0 — Sovereign AI Context OS
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

### Step 3: Start the dashboard

```bash
cd dashboard/ui-vue
npm run dev
```

Open http://localhost:5173 — the green **Live** dot confirms SSE connection.

---

## Test Your Setup

### Health check

```bash
curl http://localhost:8080/healthz
# {"status":"ok","service":"katara-core","version":"7.0.0"}
```

### List providers

```bash
curl http://localhost:8080/v1/providers
# {"providers":["ollama-llama3","ollama-codellama","ollama-mistral"]}
```

### Compile only (no LLM call)

```bash
curl -X POST http://localhost:8080/v1/compile \
  -H "Content-Type: application/json" \
  -d '{"context": "Debug this auth function with retry logic", "sensitive": false}'
```

Response includes `provider`, `model`, `intent`, `compiled_tokens`, `cache_hit`, etc.

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
2. Route to `ollama-llama3` (Llama 3.1)
3. Forward to Ollama on localhost:11434
4. Return OpenAI-compatible response with a `katara` section showing optimization stats

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

# Pull models
ollama pull llama3.1       # 8B general
ollama pull codellama      # code tasks
ollama pull mistral        # debug/analysis
ollama pull phi3           # lightweight alternative

# Verify
ollama list
```

### Map models to KATARA providers

Each model you pull in Ollama should have a corresponding entry in `configs/providers/providers.yaml`:

| Ollama model | providers.yaml key | Suggested routing |
|---|---|---|
| `llama3.1` | `ollama-llama3` | default, general, summarize |
| `codellama` | `ollama-codellama` | review (code) |
| `mistral` | `ollama-mistral` | debug |
| `phi3` | `ollama-phi3` | lightweight fallback |

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

```bash
export MISTRAL_API_KEY="..."
```

```yaml
  mistral-cloud:
    type: mistral
    base_url: https://api.mistral.ai/v1
    model: mistral-large-latest
    deployment: cloud
    api_key_env: MISTRAL_API_KEY
```

---

## Docker

```bash
# Build
docker build -f deployments/docker/Dockerfile -t katara/core:7.0.0 .

# Run (with config mounted)
docker run -p 8080:8080 \
  -v $(pwd)/configs:/app/configs:ro \
  -e OPENAI_API_KEY="sk-..." \
  katara/core:7.0.0
```

For Ollama running on the host:

```bash
docker run -p 8080:8080 \
  -v $(pwd)/configs:/app/configs:ro \
  --add-host host.docker.internal:host-gateway \
  katara/core:7.0.0
```

Then change `base_url` in providers.yaml to `http://host.docker.internal:11434/v1`.

---

## Kubernetes / Helm

```bash
# Direct deployment
kubectl apply -f deployments/kubernetes/deployment.yaml

# Or via Helm
helm install katara deployments/helm/
```

Create a ConfigMap for your YAML configs:

```bash
kubectl create configmap katara-config \
  --from-file=configs/providers/providers.yaml \
  --from-file=configs/routing/routing.yaml \
  --from-file=configs/policies/policies.yaml
```

---

## Architecture

```
┌─────────────────────┐         POST /v1/chat/completions   ┌──────────────────────────┐
│   Client (curl,     │ ──────────────────────────────────► │   KATARA Rust Backend    │
│   VS Code ext,      │                                     │                          │
│   any AI tool)      │ ◄────── OpenAI-compatible JSON ──── │  ① fingerprint           │
└─────────────────────┘                                     │  ② cache lookup          │
                                                            │  ③ context compiler      │
┌─────────────────────┐                                     │  ④ memory lensing        │
│   Vue Dashboard     │ ◄── SSE /v1/metrics/stream ──────── │  ⑤ intent → route        │
│   (Pinia + SSE)     │     every 2 seconds                 │  ⑥ forward to LLM        │
└─────────────────────┘                                     └──────┬───────────────────┘
                                                                   │
                                                            ┌──────┴───────────────────┐
                                                            │  Ollama (localhost:11434) │
                                                            │   llama3.1 / codellama   │
                                                            │   mistral / phi3 / ...   │
                                                            │                          │
                                                            │  Cloud (optional)        │
                                                            │   OpenAI / Mistral AI    │
                                                            └──────────────────────────┘
```

---

## Troubleshooting

| Problem | Solution |
|---|---|
| `No config files found — using built-in defaults` | Run KATARA from the repo root: `cargo run -p core` |
| `Provider returned 404` | Check that the model is pulled in Ollama: `ollama list` |
| `HTTP error: Connection refused` | Ollama not running. Start with `ollama serve` |
| `Provider returned 401` | API key missing. Set the env var listed in `api_key_env` |
| Dashboard shows **Offline** | Backend not running on port 8080, or CORS blocked |
| npm install fails on Google Drive | Copy to local disk first: `robocopy . $env:TEMP/katara /MIR` |
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

*License: AGPL-3.0 + Commons Clause — Copyright 2024-2026 Christophe Freijanes*
