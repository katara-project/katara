# Requires -Version 5.1
<#
.SYNOPSIS
    Quick smoke-test script for the KATARA backend API.

.DESCRIPTION
    Runs 7 assertions against the KATARA REST API:
      - Health check
      - Providers list
      - Compile with debug intent
      - Compile with summarize intent
      - Compile with review intent
      - Sensitive mode override
      - Metrics snapshot

    Requires the KATARA backend to be running on localhost:8080.
    Does NOT require Ollama (all tests use /v1/compile, not /v1/chat/completions).

.EXAMPLE
    .\scripts\test-api.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Continue'   # Don't abort on test failures

$base  = "http://localhost:8080"
$pass  = 0
$fail  = 0
$total = 0

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
            ContentType = "application/json"
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

Write-Host ""
Write-Host "  KATARA API Test Suite" -ForegroundColor Cyan
Write-Host "  ─────────────────────────────────" -ForegroundColor DarkGray
Write-Host "  Target: $base" -ForegroundColor DarkGray
Write-Host ""

# ── Test 1: Health check ──────────────────────────────
Invoke-Test -Name "Health check" -Method "Get" -Path "/healthz" `
    -Check { $args[0].status -eq "ok" }

# ── Test 2: Providers list ────────────────────────────
Invoke-Test -Name "Providers list" -Method "Get" -Path "/v1/providers" `
    -Check { $args[0].providers.Count -gt 0 }

# ── Test 3: Compile — debug intent ───────────────────
Invoke-Test -Name "Compile: debug intent" -Method "Post" -Path "/v1/compile" `
    -Body @{ context = "error trace in tokio runtime scheduler"; sensitive = $false } `
    -Check { $args[0].intent -eq "debug" -and $args[0].provider -eq "ollama-mistral" }

# ── Test 4: Compile — summarize intent ───────────────
Invoke-Test -Name "Compile: summarize intent" -Method "Post" -Path "/v1/compile" `
    -Body @{ context = "summarize the Rust ownership model for a beginner"; sensitive = $false } `
    -Check { $args[0].intent -eq "summarize" -and $args[0].provider -eq "ollama-llama3" }

# ── Test 5: Compile — review intent ──────────────────
Invoke-Test -Name "Compile: review intent" -Method "Post" -Path "/v1/compile" `
    -Body @{ context = "review this pull request diff for the login module"; sensitive = $false } `
    -Check { $args[0].intent -eq "review" -and $args[0].provider -eq "ollama-qwen2.5-coder" }

# ── Test 6: Sensitive override ────────────────────────
Invoke-Test -Name "Sensitive override → local only" -Method "Post" -Path "/v1/compile" `
    -Body @{ context = "analyze this patient medical record transcript"; sensitive = $true } `
    -Check { $args[0].provider -eq "ollama-llama3" }

# ── Test 7: Metrics snapshot ──────────────────────────
Invoke-Test -Name "Metrics snapshot" -Method "Get" -Path "/v1/metrics" `
    -Check { $args[0].total_requests -ge 3 }

# ── Summary ───────────────────────────────────────────
Write-Host ""
Write-Host "  ─────────────────────────────────" -ForegroundColor DarkGray

$color = if ($fail -eq 0) { "Green" } else { "Red" }
Write-Host ("  Results: {0}/{1} passed" -f $pass, $total) -ForegroundColor $color

if ($fail -gt 0) {
    Write-Host ""
    Write-Host "  Troubleshooting:" -ForegroundColor Yellow
    Write-Host "    - Backend not running?  cargo run -p core" -ForegroundColor Yellow
    Write-Host "    - Wrong intent?         Check configs/routing/routing.yaml" -ForegroundColor Yellow
    Write-Host "    See TESTING.md for the full test guide." -ForegroundColor Yellow
}
Write-Host ""
