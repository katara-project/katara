# Requires -Version 5.1
<#
.SYNOPSIS
    Start all KATARA services (Ollama, backend, dashboard) in one command.

.DESCRIPTION
    Launches Ollama (if installed), waits for it to be ready,
    starts the KATARA Rust backend on :8080, and the Vue dashboard on :5173.
    Each service runs in its own background job. Press Ctrl+C to stop everything.

.EXAMPLE
    .\scripts\start-win.ps1
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host ''
Write-Host '  KATARA - Starting all services' -ForegroundColor Cyan
Write-Host '  --------------------------------' -ForegroundColor DarkGray
Write-Host ''

$rootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $rootDir

# -- Load .env secrets into current process --------------
if (Test-Path '.env') {
    Get-Content '.env' | ForEach-Object {
        if ($_ -match '^([^#=]+)=(.*)$') {
            [Environment]::SetEnvironmentVariable($Matches[1].Trim(), $Matches[2].Trim(), 'Process')
        }
    }
    Write-Host '[ok] .env loaded.' -ForegroundColor Green
} else {
    Write-Host '[--] No .env file found (cloud providers will have no API keys).' -ForegroundColor Yellow
}

# -- Track background jobs for cleanup -------------------
$jobs = @()

function Stop-AllJobs {
    Write-Host ''
    Write-Host '  Stopping all KATARA services...' -ForegroundColor Yellow
    foreach ($j in $jobs) {
        Stop-Job -Job $j -ErrorAction SilentlyContinue
        Remove-Job -Job $j -Force -ErrorAction SilentlyContinue
    }
    # Also stop any ollama serve we started
    Get-Process -Name 'ollama' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Write-Host '  All services stopped.' -ForegroundColor Green
}

# Register Ctrl+C handler
$null = Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action { Stop-AllJobs }

# -- 1. Start Ollama ------------------------------------
$ollamaRunning = $false
if (Get-Command ollama -ErrorAction SilentlyContinue) {
    # Check if Ollama is already serving
    try {
        $null = Invoke-RestMethod -Uri 'http://localhost:11434/api/tags' -TimeoutSec 2 -ErrorAction Stop
        Write-Host '[ok] Ollama is already running.' -ForegroundColor Green
        $ollamaRunning = $true
    } catch {
        Write-Host '==> Starting Ollama...' -ForegroundColor Cyan
        $ollamaJob = Start-Job -ScriptBlock { ollama serve 2>&1 }
        $jobs += $ollamaJob

        # Wait for Ollama to be ready (max 30s)
        $ready = $false
        for ($i = 0; $i -lt 30; $i++) {
            Start-Sleep -Seconds 1
            try {
                $null = Invoke-RestMethod -Uri 'http://localhost:11434/api/tags' -TimeoutSec 2 -ErrorAction Stop
                $ready = $true
                break
            } catch {
                # Not ready yet
            }
        }
        if ($ready) {
            Write-Host '[ok] Ollama started on :11434' -ForegroundColor Green
            $ollamaRunning = $true
        } else {
            Write-Host '[!!] Ollama failed to start within 30s.' -ForegroundColor Red
        }
    }
} else {
    Write-Host '[--] Ollama not installed - skipping local models.' -ForegroundColor Yellow
}

# -- 2. Start KATARA backend ----------------------------
Write-Host '==> Starting KATARA backend...' -ForegroundColor Cyan
$backendJob = Start-Job -ScriptBlock {
    param($dir)
    Set-Location $dir
    # Forward .env into job
    if (Test-Path '.env') {
        Get-Content '.env' | ForEach-Object {
            if ($_ -match '^([^#=]+)=(.*)$') {
                [Environment]::SetEnvironmentVariable($Matches[1].Trim(), $Matches[2].Trim(), 'Process')
            }
        }
    }
    cargo run -p core 2>&1
} -ArgumentList $rootDir
$jobs += $backendJob

# Wait for backend to be ready (max 60s)
$backendReady = $false
for ($i = 0; $i -lt 60; $i++) {
    Start-Sleep -Seconds 1
    try {
        $null = Invoke-RestMethod -Uri 'http://localhost:8080/healthz' -TimeoutSec 2 -ErrorAction Stop
        $backendReady = $true
        break
    } catch {
        # Not ready yet - check if job failed
        if ($backendJob.State -eq 'Failed' -or $backendJob.State -eq 'Completed') {
            $output = Receive-Job -Job $backendJob 2>&1
            Write-Host "[!!] Backend exited: $output" -ForegroundColor Red
            break
        }
    }
}
if ($backendReady) {
    Write-Host '[ok] KATARA backend running on http://127.0.0.1:8080' -ForegroundColor Green
} else {
    if ($backendJob.State -eq 'Running') {
        Write-Host '[..] Backend still starting (compilation may take a moment)...' -ForegroundColor Yellow
    }
}

# -- 3. Start Vue dashboard -----------------------------
Write-Host '==> Starting dashboard...' -ForegroundColor Cyan
$dashboardJob = Start-Job -ScriptBlock {
    param($dir)
    Set-Location (Join-Path $dir 'dashboard/ui-vue')
    npm run dev 2>&1
} -ArgumentList $rootDir
$jobs += $dashboardJob

# Wait briefly for Vite to start
Start-Sleep -Seconds 5
Write-Host '[ok] Dashboard starting on http://localhost:5173' -ForegroundColor Green

# -- Summary ---------------------------------------------
Write-Host ''
Write-Host '  ----------------------------------------------' -ForegroundColor DarkGray
Write-Host '  All KATARA services launched!' -ForegroundColor Green
Write-Host ''
Write-Host '  Services:' -ForegroundColor Cyan
if ($ollamaRunning) {
    Write-Host '    Ollama        : http://localhost:11434' -ForegroundColor White
}
Write-Host '    KATARA API    : http://localhost:8080' -ForegroundColor White
Write-Host '    Dashboard     : http://localhost:5173' -ForegroundColor White
Write-Host '    VS Code Agent : @katara in Copilot Chat' -ForegroundColor White
Write-Host ''
Write-Host '  Press Ctrl+C to stop all services.' -ForegroundColor Yellow
Write-Host ''

# -- Keep alive - tail backend logs ----------------------
try {
    while ($true) {
        # Show any new output from backend
        $output = Receive-Job -Job $backendJob -ErrorAction SilentlyContinue
        if ($output) { $output | Write-Host }

        # Check if backend died
        if ($backendJob.State -ne 'Running') {
            Write-Host '[!!] Backend has stopped.' -ForegroundColor Red
            break
        }

        Start-Sleep -Seconds 2
    }
} finally {
    Stop-AllJobs
}
