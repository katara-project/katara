# Testing KATARA

> **KATARA v7.7.1** — This guide walks you through every layer of verification:
> the Rust backend, the HTTP API, the Vue dashboard, and the MCP VS Code agent.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Smoke Tests — Backend API](#smoke-tests--backend-api)
3. [Intent Routing Test Matrix](#intent-routing-test-matrix)
4. [Sensitive Mode Test](#sensitive-mode-test)
5. [Metrics Verification](#metrics-verification)
6. [MCP Agent Tests (VS Code)](#mcp-agent-tests-vs-code)
7. [Dashboard Verification](#dashboard-verification)
8. [PowerShell Quick-Test Script](#powershell-quick-test-script)
9. [Expected Responses Reference](#expected-responses-reference)
10. [Common Errors](#common-errors)

---

## Prerequisites

Before running any test, KATARA must be running.

**One-command start (Windows):**

```powershell
.\scripts\start-win.ps1
```

Or manually:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo run -p core
```

Confirm the backend is up:

```powershell
Invoke-RestMethod http://localhost:8080/healthz
# Expected: status=ok, service=katara-core, version=7.7.1
```

**Ollama models (for LLM tests):**

```powershell
ollama pull llama3:latest          # default, general, summarize
ollama pull qwen2.5-coder:7b       # code review
ollama pull mistral:7b-instruct    # debug, analysis
```

> Tests 1–6 only call `/v1/compile` — they do NOT require Ollama.
> Tests 7–10 call `/v1/chat/completions` and forward to an LLM — Ollama must be running.

---

## Smoke Tests — Backend API

### Test 1 — Health check

```powershell
Invoke-RestMethod http://localhost:8080/healthz
```

**Expected:**

```json
{
  "status": "ok",
  "service": "katara-core",
  "version": "7.7.1"
}
```

---

### Test 2 — List providers

```powershell
Invoke-RestMethod http://localhost:8080/v1/providers
```

**Expected:**

```json
{
  "providers": [
    "ollama-llama3",
    "ollama-qwen2.5-coder",
    "ollama-mistral",
    "ollama-ocr-deepseek",
    "mistral-ocr-cloud"
  ]
}
```

---

### Test 3 — Compile (no LLM call)

```powershell
Invoke-RestMethod -Method Post http://localhost:8080/v1/compile `
  -ContentType "application/json" `
  -Body '{"context": "Debug this auth function with retry logic", "sensitive": false}'
```

**Expected fields:**

| Field | Expected value |
|-------|----------------|
| `intent` | `"debug"` |
| `provider` | `"ollama-mistral"` |
| `model` | `"mistral:7b-instruct"` |
| `cache_hit` | `false` (first call) |
| `compiled_tokens` | ≤ `raw_tokens` |

---

### Test 4 — Compile a summarize request

```powershell
Invoke-RestMethod -Method Post http://localhost:8080/v1/compile `
  -ContentType "application/json" `
  -Body '{"context": "Summarize the main concepts of Rust ownership", "sensitive": false}'
```

**Expected:**

| Field | Expected value |
|-------|----------------|
| `intent` | `"summarize"` |
| `provider` | `"ollama-llama3"` |
| `model` | `"llama3:latest"` |

---

### Test 5 — Compile a code review request

```powershell
Invoke-RestMethod -Method Post http://localhost:8080/v1/compile `
  -ContentType "application/json" `
  -Body '{"context": "Review this pull request diff: +fn login()", "sensitive": false}'
```

**Expected:**

| Field | Expected value |
|-------|----------------|
| `intent` | `"review"` |
| `provider` | `"ollama-qwen2.5-coder"` |
| `model` | `"qwen2.5-coder:7b"` |

---

### Test 6 — Cache hit (run Test 3 twice)

Run the **exact same request** as Test 3 a second time.

**Expected:** `cache_hit: true` on the second call.

---

## Intent Routing Test Matrix

| Context keyword | Expected `intent` | Expected `provider`       | Expected `model`         |
|-----------------|-------------------|---------------------------|--------------------------|
| `error`, `trace`| `debug`           | `ollama-mistral`          | `mistral:7b-instruct`    |
| `summarize`     | `summarize`       | `ollama-llama3`           | `llama3:latest`          |
| `diff`, `pull request` | `review` | `ollama-qwen2.5-coder`   | `qwen2.5-coder:7b`       |
| `ocr`, `scan`   | `ocr`             | `mistral-ocr-cloud`       | `mistral-ocr-2512`       |
| anything else   | `general`         | `ollama-llama3`           | `llama3:latest`          |

Test with:

```powershell
$intents = @(
  @{context="error trace in tokio runtime"; expect="debug"},
  @{context="summarize this README"; expect="summarize"},
  @{context="review this pull request diff"; expect="review"},
  @{context="explain Rust closures"; expect="general"}
)

foreach ($t in $intents) {
  $res = Invoke-RestMethod -Method Post http://localhost:8080/v1/compile `
    -ContentType "application/json" `
    -Body (ConvertTo-Json @{context=$t.context; sensitive=$false})
  $status = if ($res.intent -eq $t.expect) { "PASS" } else { "FAIL" }
  Write-Host "[$status] intent=$($res.intent)  provider=$($res.provider)  (expected: $($t.expect))"
}
```

---

## Sensitive Mode Test

```powershell
Invoke-RestMethod -Method Post http://localhost:8080/v1/compile `
  -ContentType "application/json" `
  -Body '{"context": "Analyze this patient medical record", "sensitive": true}'
```

**Expected:**

| Field | Expected value |
|-------|----------------|
| `provider` | `"ollama-llama3"` |
| `routing_reason` | contains `"sensitive"` |

Regardless of the keyword content, `sensitive: true` **must always** route to `ollama-llama3` (the `sensitive_override` in `routing.yaml`).

---

## Metrics Verification

After running Tests 3–5 (3 compile calls), check the metrics:

```powershell
Invoke-RestMethod http://localhost:8080/v1/metrics
```

**Expected:**

```json
{
  "total_requests": 3,
  "local_ratio": 100,
  "cache_misses": 3,
  "intent_stats": {
    "debug":     { "requests": 1 },
    "summarize": { "requests": 1 },
    "review":    { "requests": 1 }
  }
}
```

After running Test 6 (cache hit), check again:

```json
{
  "total_requests": 4,
  "cache_hits": 1,
  "cache_misses": 3
}
```

---

## MCP Agent Tests (VS Code)

### Setup check

1. Open VS Code in the `katara/` workspace.
2. Open Copilot Chat (`Ctrl+Alt+I`).
3. Type `@katara` — VS Code should auto-complete it and show the agent badge.

If `@katara` does not appear, check:
- The backend is running on `:8080`
- `.vscode/mcp.json` is present with `"cwd": "${workspaceFolder}/mcp"`
- `mcp/node_modules/@modelcontextprotocol` exists (run `npm install` in `mcp/` if not)

### MCP Test 1 — List providers

```md
@katara list all configured providers
```

**Expected in chat:** A JSON list with 5 provider keys.

---

### MCP Test 1b — Set live client context

```md
@katara set client context to Claude Sonnet 4.6 on Anthropic
```

**Expected in chat:** JSON response from `/v1/runtime/client-context` showing `upstream_provider="Anthropic"` and `upstream_model="Claude Sonnet 4.6"`.

---

### MCP Test 2 — Compile context

```md
@katara compile this context: debug an error in the tokio runtime scheduler
```

**Expected in chat:** JSON with `intent="debug"`, `provider="ollama-mistral"`, `cache_hit=false`.

---

### MCP Test 3 — Metrics after compile

```md
@katara show current metrics
```

**Expected in chat:** JSON with `total_requests ≥ 1`, `local_ratio=100`.

---

### MCP Test 4 — Chat (requires Ollama running with llama3:latest)

```md
@katara what is the difference between Arc and Rc in Rust?
```

**Expected in chat:** A JSON response with an OpenAI-compatible `choices[0].message.content` plus a `katara` section showing `intent`, `provider`, `compiled_tokens`.

If Ollama is not running, you will see an error. Start it with `ollama serve`.

---

### MCP Test 4b — Upstream model lineage

Verify that the MCP bridge is forwarding upstream client metadata dynamically.

1. Configure a runtime resolver command in `.vscode/mcp.json`:

```json
{
  "servers": {
    "katara": {
      "env": {
        "KATARA_CLIENT_APP": "VS Code Copilot Chat",
        "KATARA_CLIENT_CONTEXT_CMD": "powershell -File ..\\scripts\\resolve-upstream-context.ps1"
      }
    }
  }
}
```

2. Make the resolver return one model, for example:

```json
{
  "client_app": "VS Code Copilot Chat",
  "upstream_provider": "GitHub Copilot",
  "upstream_model": "GPT-5.4"
}
```

3. In Copilot Chat, run:

```md
@katara explain the difference between Arc and Mutex in Rust
```

4. Then call:

```powershell
Invoke-RestMethod http://localhost:8080/v1/metrics | ConvertTo-Json -Depth 8
```

**Expected in metrics JSON:**

- `last_request.client_app = "VS Code Copilot Chat"`
- `last_request.upstream_provider = "GitHub Copilot"`
- `last_request.upstream_model = "GPT-5.4"`
- `last_request.routed_model` may differ, which is the expected behavior

This verifies that KATARA distinguishes the upstream assistant/client model from the routed model.

5. Change the resolver output to:

```json
{
  "client_app": "VS Code Copilot Chat",
  "upstream_provider": "Anthropic",
  "upstream_model": "Claude Sonnet 4.6"
}
```

6. Run another `@katara` request and re-check `/v1/metrics`.

**Expected:** `last_request.upstream_model = "Claude Sonnet 4.6"` without changing backend code or restarting KATARA.

7. Open the Overview dashboard and confirm the `Last Request` panel now shows:

- `Claude Sonnet 4.6` as the upstream model
- the current routed provider and routed model
- `Cache hit` or `Cache miss`
- `Sensitive override` only when the request was sent as sensitive

---

### HTTP Test — Chat streaming

KATARA now supports `stream=true` on `/v1/chat/completions` and proxies the provider SSE stream.

```powershell
$body = @{
  model = "llama3:latest"
  stream = $true
  client_app = "powershell test"
  upstream_provider = "manual"
  upstream_model = "llama3:latest"
  messages = @(@{ role = "user"; content = "Say hello in five words" })
} | ConvertTo-Json -Depth 5

Invoke-WebRequest http://localhost:8080/v1/chat/completions `
  -Method Post `
  -ContentType "application/json" `
  -Body $body
```

**Expected:**

- Response header `Content-Type: text/event-stream`
- Response body contains `data:` frames and ends with `[DONE]`
- `/v1/metrics` increments `total_requests`
- Repeating the same streamed request should eventually surface a `Cache hit` in the Overview or Runtime Audit views after the first response has been fully cached

---

### HTTP Test — Multi-turn compatibility

Verify that KATARA preserves prior chat turns and forwards extra OpenAI-compatible options.

```powershell
$body = @{
  model = "llama3:latest"
  temperature = 0.2
  client_app = "powershell test"
  upstream_provider = "manual"
  upstream_model = "llama3:latest"
  messages = @(
    @{ role = "system"; content = "Answer in one sentence." },
    @{ role = "assistant"; content = "Previous answer for context." },
    @{ role = "user"; content = "Explain what a mutex does." }
  )
} | ConvertTo-Json -Depth 6

Invoke-RestMethod http://localhost:8080/v1/chat/completions `
  -Method Post `
  -ContentType "application/json" `
  -Body $body | ConvertTo-Json -Depth 8
```

**Expected:**

- request succeeds without dropping earlier `system` or `assistant` messages
- `/v1/compile` returns a non-empty `compiled_context` for the active user prompt
- routed provider receives the extra `temperature` option
- the latest user turn is what gets reduced before forwarding, while prior history remains present
- repeating the exact same payload can produce a cache hit
- repeated compile/chat requests for the same input can also report `semantic_cache_hit = true` in the `katara` metadata once the compiler result has been reused

---

### MCP Test 5 — Sensitive mode

```md
@katara explain what a JWT token is — treat this as sensitive
```

The agent description says to set `sensitive: true` for sensitive contexts.  
**Expected:** `provider="ollama-llama3"`, routing_reason mentions sensitive override.

---

## Dashboard Verification

1. Start the Vue dashboard:

```powershell
Set-Location dashboard/ui-vue
npm run dev
```

2. Open http://localhost:5173

3. Verify the green **Live** badge is visible (SSE connection to `/v1/metrics/stream`).

4. Verify the bottom-left version tag matches `GET /version`:

```powershell
Invoke-RestMethod http://localhost:8080/version
```

**Expected:** the sidebar version in the dashboard matches the `version` field returned by the backend.

5. Run a few compile requests in parallel:

```powershell
1..5 | ForEach-Object {
  Start-Job {
    Invoke-RestMethod -Method Post http://localhost:8080/v1/compile `
      -ContentType "application/json" `
      -Body '{"context": "summarize this text about machine learning"}'
  }
}
```

6. Watch the dashboard update in real time — the efficiency chart, last-request lineage, and model scope panels should reflect the new requests within 2 seconds.

7. Open the `Runtime Audit` navigation entry and verify that recent requests appear in reverse chronological order with:

- upstream client/provider/model
- routed provider/model
- cache hit or miss
- sensitive override or standard routing
- intent and timestamp

8. In the Overview page, verify that:

- `Upstream Client Models` shows the model reported by the client, such as `GPT-5.4`, when that metadata is available
- `Live AI Efficiency by Routed Model` can still show a different target such as `gpt-4o-mini` or a local Ollama model
- this difference is expected and reflects upstream-vs-routed scope, not a dashboard mismatch
- if the client does not expose upstream model metadata, a visible warning banner appears in `Model Scope Clarity` instead of pretending the upstream model is known

---

## PowerShell Quick-Test Script

Save this as `scripts/test-api.ps1` for a repeatable smoke test:

```powershell
#Requires -Version 5.1
$base = "http://localhost:8080"
$pass = 0; $fail = 0

function Test-Endpoint($name, $method, $path, $body, $check) {
  try {
    $params = @{ Method=$method; Uri="$base$path"; ContentType="application/json" }
    if ($body) { $params.Body = $body | ConvertTo-Json }
    $res = Invoke-RestMethod @params
    if (& $check $res) {
      Write-Host "[PASS] $name" -ForegroundColor Green
      $script:pass++
    } else {
      Write-Host "[FAIL] $name — unexpected response: $($res | ConvertTo-Json -Compress)" -ForegroundColor Red
      $script:fail++
    }
  } catch {
    Write-Host "[FAIL] $name — $($_.Exception.Message)" -ForegroundColor Red
    $script:fail++
  }
}

# --- Run tests ---
Test-Endpoint "Health check"         Get  "/healthz"      $null     { $args[0].status -eq "ok" }
Test-Endpoint "Providers list"       Get  "/v1/providers" $null     { $args[0].providers.Count -gt 0 }
Test-Endpoint "Compile (debug)"      Post "/v1/compile"   @{context="error trace in auth module"; sensitive=$false} { $args[0].intent -eq "debug" }
Test-Endpoint "Compile (summarize)"  Post "/v1/compile"   @{context="summarize the Rust book"; sensitive=$false}    { $args[0].intent -eq "summarize" }
Test-Endpoint "Compile (review)"     Post "/v1/compile"   @{context="review this pull request diff"; sensitive=$false} { $args[0].intent -eq "review" }
Test-Endpoint "Sensitive override"   Post "/v1/compile"   @{context="patient record"; sensitive=$true} { $args[0].provider -eq "ollama-llama3" }
Test-Endpoint "Metrics"              Get  "/v1/metrics"   $null     { $args[0].total_requests -ge 3 }

Write-Host ""
Write-Host "Results: $pass passed, $fail failed" -ForegroundColor $(if ($fail -eq 0) { "Green" } else { "Red" })
```

Run it:

```powershell
.\scripts\test-api.ps1
```

Expected output:

```md
[PASS] Health check
[PASS] Providers list
[PASS] Compile (debug)
[PASS] Compile (summarize)
[PASS] Compile (review)
[PASS] Sensitive override
[PASS] Metrics

Results: 7 passed, 0 failed
```

---

## Expected Responses Reference

### `/v1/compile` full response

```json
{
  "intent": "debug",
  "fingerprint": "13153826650143948697",
  "raw_tokens": 9,
  "compiled_tokens": 9,
  "memory_reused_tokens": 3,
  "context_reuse_ratio": 0.33,
  "provider": "ollama-mistral",
  "model": "mistral:7b-instruct",
  "cache_hit": false,
  "routing_reason": "Intent [debug] → routed to ollama-mistral."
}
```

### `/v1/metrics` full response

```json
{
  "total_requests": 7,
  "cache_hits": 1,
  "cache_misses": 6,
  "raw_tokens": 63,
  "compiled_tokens": 63,
  "memory_reused_tokens": 21,
  "efficiency_score": 33,
  "local_ratio": 100,
  "routes_local": 7,
  "routes_cloud": 0,
  "intent_stats": {
    "debug":     { "requests": 2, "raw_tokens": 18, "compiled_tokens": 18 },
    "summarize": { "requests": 3, "raw_tokens": 27, "compiled_tokens": 27 },
    "review":    { "requests": 2, "raw_tokens": 18, "compiled_tokens": 18 }
  }
}
```

---

## Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `Connection refused :8080` | Backend not started | Run `cargo run -p core` |
| `Connection refused :11434` | Ollama not running | Run `ollama serve` |
| `KATARA 500: model not found` | Ollama model not pulled | `ollama pull llama3:latest` |
| `@katara not found in Copilot Chat` | MCP not connected | Check `.vscode/mcp.json` + run backend |
| `Cannot find module '@modelcontextprotocol/sdk'` | MCP deps not installed | `cd mcp && npm install` |
| `cargo: command not found` | Cargo not in PATH | `$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH` |
| `AddrInUse :8080` | Stale backend process | `.\scripts\start-win.ps1` (kills existing process automatically) |
