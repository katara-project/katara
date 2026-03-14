# Requires -Version 5.1
<#
.SYNOPSIS
    End-to-end integration test suite for the DISTIRA backend.

.DESCRIPTION
    Verifies the full pipeline: compile → route → metrics for each supported intent.
    Also tests DELETE /v1/metrics/reset and validates counter zeroing.

    Test groups:
      1. Baseline health + providers
      2. Intent routing: debug, codegen, review, summarize, translate, ocr, general, sensitive
      3. Metrics increment: counters grow after requests
      4. Metrics reset: DELETE /v1/metrics/reset zeros all counters
      5. Post-reset: first new compile increments from zero

    Requires the DISTIRA backend running on localhost:8080.
    Does NOT require Ollama (all tests use /v1/compile, not /v1/chat/completions).

.EXAMPLE
    .\scripts\test-e2e.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Continue'

$base  = 'http://localhost:8080'
$pass  = 0
$fail  = 0
$total = 0

# ── Helpers ──────────────────────────────────────────────────────────────────

function Invoke-Test {
    param(
        [string]$Name,
        [string]$Method,
        [string]$Path,
        [hashtable]$Body,
        [scriptblock]$Check
    )
    $script:total++
    try {
        $params = @{
            Method      = $Method
            Uri         = "$base$Path"
            ContentType = 'application/json'
            ErrorAction = 'Stop'
        }
        if ($Body) { $params.Body = ($Body | ConvertTo-Json -Compress) }

        $res = Invoke-RestMethod @params

        if (& $Check $res) {
            Write-Host "  [PASS] $Name" -ForegroundColor Green
            $script:pass++
        } else {
            Write-Host "  [FAIL] $Name" -ForegroundColor Red
            Write-Host "         Response: $($res | ConvertTo-Json -Compress)" -ForegroundColor DarkRed
            $script:fail++
        }
    } catch {
        Write-Host "  [FAIL] $Name — $($_.Exception.Message)" -ForegroundColor Red
        $script:fail++
    }
}

function Invoke-RawTest {
    param(
        [string]$Name,
        [string]$Method,
        [string]$Path,
        [scriptblock]$Check
    )
    $script:total++
    try {
        $res = Invoke-WebRequest -Method $Method -Uri "$base$Path" -ErrorAction Stop
        if (& $Check $res) {
            Write-Host "  [PASS] $Name" -ForegroundColor Green
            $script:pass++
        } else {
            Write-Host "  [FAIL] $Name — status $($res.StatusCode)" -ForegroundColor Red
            $script:fail++
        }
    } catch {
        Write-Host "  [FAIL] $Name — $($_.Exception.Message)" -ForegroundColor Red
        $script:fail++
    }
}

function Section([string]$title) {
    Write-Host ""
    Write-Host "  $title" -ForegroundColor Cyan
    Write-Host "  $('─' * ($title.Length))" -ForegroundColor DarkGray
}

# ── Header ───────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "  DISTIRA E2E Integration Test Suite" -ForegroundColor Cyan
Write-Host "  ────────────────────────────────────────" -ForegroundColor DarkGray
Write-Host "  Target: $base" -ForegroundColor DarkGray
Write-Host ""

# ── Group 1: Baseline ────────────────────────────────────────────────────────

Section "1. Baseline health + providers"

Invoke-Test -Name "GET /healthz → status ok" -Method Get -Path '/healthz' `
    -Check { $args[0].status -eq 'ok' }

Invoke-Test -Name "GET /version → non-empty version" -Method Get -Path '/version' `
    -Check { $args[0].version -match '^\d+\.\d+' }

Invoke-Test -Name "GET /v1/providers → at least 1 provider" -Method Get -Path '/v1/providers' `
    -Check { $args[0].providers.Count -gt 0 }

# ── Group 2: Intent routing via /v1/compile ───────────────────────────────────

Section "2. Intent routing (compile → route)"

Invoke-Test -Name "debug intent detected" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'error: unexpected nil pointer exception in goroutine stack trace'; sensitive = $false } `
    -Check { $args[0].intent -eq 'debug' }

Invoke-Test -Name "codegen intent detected" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'implement a TypeScript function to parse JWT tokens'; sensitive = $false } `
    -Check { $args[0].intent -eq 'codegen' }

Invoke-Test -Name "review intent detected" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'review this pull request diff for security issues'; sensitive = $false } `
    -Check { $args[0].intent -eq 'review' }

Invoke-Test -Name "summarize intent detected" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'summarize the key points from this document'; sensitive = $false } `
    -Check { $args[0].intent -eq 'summarize' }

Invoke-Test -Name "translate intent detected" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'translate this paragraph from French to English'; sensitive = $false } `
    -Check { $args[0].intent -eq 'translate' }

Invoke-Test -Name "ocr intent detected" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'OCR extract text from this scanned image'; sensitive = $false } `
    -Check { $args[0].intent -eq 'ocr' }

Invoke-Test -Name "general intent (fallback)" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'what is the capital of France?'; sensitive = $false } `
    -Check { $args[0].intent -ne $null }

Invoke-Test -Name "sensitive flag forces sovereign provider" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'patient blood pressure data analysis'; sensitive = $true } `
    -Check { $args[0].provider -match 'ollama' }

# ── Group 3: Token reduction via /v1/compile ──────────────────────────────────

Section "3. Token pipeline (raw > 0, compiled > 0)"

Invoke-Test -Name "compile returns raw_tokens > 0" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'explain the difference between a mutex and a semaphore in concurrent programming'; sensitive = $false } `
    -Check { $args[0].raw_tokens -gt 0 }

Invoke-Test -Name "compile returns compiled_tokens > 0" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'explain the difference between a mutex and a semaphore in concurrent programming'; sensitive = $false } `
    -Check { $args[0].compiled_tokens -gt 0 }

Invoke-Test -Name "compile result has fingerprint" -Method Post -Path '/v1/compile' `
    -Body @{ context = 'debug the segfault in this C program'; sensitive = $false } `
    -Check { $args[0].fingerprint -ne $null -and $args[0].fingerprint -ne '' }

# ── Group 4: Metrics increment after requests ─────────────────────────────────

Section "4. Metrics: counters increment after compile requests"

# Send a known compile request to force at least 1 recorded request
$null = Invoke-RestMethod -Method Post -Uri "$base/v1/compile" `
    -ContentType 'application/json' `
    -Body ('{"context":"metrics increment test probe","sensitive":false}')

Invoke-Test -Name "GET /v1/metrics → total_requests >= 1" -Method Get -Path '/v1/metrics' `
    -Check { $args[0].total_requests -ge 1 }

Invoke-Test -Name "GET /v1/metrics → raw_tokens > 0" -Method Get -Path '/v1/metrics' `
    -Check { $args[0].raw_tokens -gt 0 }

Invoke-Test -Name "GET /v1/metrics → session_cost_usd field present" -Method Get -Path '/v1/metrics' `
    -Check { $null -ne $args[0].session_cost_usd }

# ── Group 5: Metrics reset ────────────────────────────────────────────────────

Section "5. DELETE /v1/metrics/reset"

Invoke-RawTest -Name "DELETE /v1/metrics/reset → 204 No Content" `
    -Method Delete -Path '/v1/metrics/reset' `
    -Check { $args[0].StatusCode -eq 204 }

# Give the backend a moment to apply the reset
Start-Sleep -Milliseconds 200

Invoke-Test -Name "After reset: total_requests == 0" -Method Get -Path '/v1/metrics' `
    -Check { $args[0].total_requests -eq 0 }

Invoke-Test -Name "After reset: raw_tokens == 0" -Method Get -Path '/v1/metrics' `
    -Check { $args[0].raw_tokens -eq 0 }

Invoke-Test -Name "After reset: cache_hits == 0 and cache_misses == 0" -Method Get -Path '/v1/metrics' `
    -Check { $args[0].cache_hits -eq 0 -and $args[0].cache_misses -eq 0 }

Invoke-Test -Name "After reset: intent_stats is empty" -Method Get -Path '/v1/metrics' `
    -Check {
        $stats = $args[0].intent_stats
        ($stats | Get-Member -MemberType NoteProperty).Count -eq 0
    }

# ── Group 6: Post-reset re-increment ─────────────────────────────────────────

Section "6. Post-reset pipeline still works"

# One compile after reset
$null = Invoke-RestMethod -Method Post -Uri "$base/v1/compile" `
    -ContentType 'application/json' `
    -Body ('{"context":"post-reset smoke test","sensitive":false}')

Invoke-Test -Name "total_requests back to 1 after reset+1 compile" -Method Get -Path '/v1/metrics' `
    -Check { $args[0].total_requests -eq 1 }

# ── Summary ───────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "  ────────────────────────────────────────" -ForegroundColor DarkGray
$color = if ($fail -eq 0) { 'Green' } else { 'Red' }
Write-Host ("  Results: {0} passed, {1} failed, {2} total" -f $pass, $fail, $total) -ForegroundColor $color
Write-Host ""

if ($fail -gt 0) { exit 1 }
