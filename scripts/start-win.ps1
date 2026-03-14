# Requires -Version 5.1
<#
.SYNOPSIS
    Start all DISTIRA services (Ollama, backend, dashboard) in one command.

.DESCRIPTION
    Launches Ollama (if installed), waits for it to be ready,
    starts the DISTIRA Rust backend on :8080, and the Vue dashboard on :5173.
    Each service runs in its own background job. Press Ctrl+C to stop everything.

.EXAMPLE
    .\scripts\start-win.ps1
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host ''
Write-Host '  DISTIRA - Starting all services' -ForegroundColor Cyan
Write-Host '  --------------------------------' -ForegroundColor DarkGray
Write-Host ''

$rootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $rootDir

# -- Ensure cargo is in PATH for this session -----------
if (Test-Path "$env:USERPROFILE\.cargo\bin") {
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
}

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
    Write-Host '  Stopping all DISTIRA services...' -ForegroundColor Yellow
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

# -- 2. Start DISTIRA backend ----------------------------
# Kill any previous core.exe to avoid AddrInUse on :8080
Get-Process -Name 'core' -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "[..] Stopping previous core.exe (PID $($_.Id))" -ForegroundColor Yellow
    $_ | Stop-Process -Force -ErrorAction SilentlyContinue
}
# Also kill by PID from netstat (catches jobs that Get-Process misses)
$netstatLines = netstat -ano 2>$null | Select-String ':8080\s'
foreach ($line in $netstatLines) {
    if ($line -match '\s(\d+)$') {
        $pid8080 = [int]$Matches[1]
        if ($pid8080 -gt 0) {
            Write-Host "[..] Stopping PID $pid8080 on :8080" -ForegroundColor Yellow
            Stop-Process -Id $pid8080 -Force -ErrorAction SilentlyContinue
        }
    }
}
Start-Sleep -Seconds 1

Write-Host '==> Starting DISTIRA backend...' -ForegroundColor Cyan
$cargoPath = "$env:USERPROFILE\.cargo\bin"

# Use pre-built release binary when available to skip recompilation on daily
# start. Rebuilds only when sources are newer than the binary.
$releaseBin = Join-Path $rootDir 'target\release\core.exe'
$srcDir      = Join-Path $rootDir 'core\src'
$needsBuild  = $true
if (Test-Path $releaseBin) {
    $binAge = (Get-Item $releaseBin).LastWriteTime
    $newest = (Get-ChildItem $srcDir -Recurse -Filter '*.rs' |
                Sort-Object LastWriteTime -Descending |
                Select-Object -First 1).LastWriteTime
    if ($null -ne $newest -and $newest -le $binAge) {
        $needsBuild = $false
        Write-Host '[ok] Release binary is up-to-date — skipping compilation.' -ForegroundColor Green
    } else {
        Write-Host '[..] Sources changed — rebuilding...' -ForegroundColor Yellow
    }
}

if ($needsBuild) {
    Write-Host '[..] Building release binary (first run or sources changed)...' -ForegroundColor Cyan
    $env:PATH = "$cargoPath;$env:PATH"
    if (Test-Path '.env') {
        Get-Content '.env' | ForEach-Object {
            if ($_ -match '^([^#=]+)=(.*)$') {
                [Environment]::SetEnvironmentVariable($Matches[1].Trim(), $Matches[2].Trim(), 'Process')
            }
        }
    }
    & "$cargoPath\cargo.exe" build --release -p core 2>&1 | ForEach-Object { Write-Host "  $_" }
    if ($LASTEXITCODE -ne 0) {
        Write-Host '[!!] Release build failed.' -ForegroundColor Red; exit 1
    }
}

$backendJob = Start-Job -ScriptBlock {
    param($dir, $bin)
    Set-Location $dir
    # Forward .env into job
    if (Test-Path '.env') {
        Get-Content '.env' | ForEach-Object {
            if ($_ -match '^([^#=]+)=(.*)$') {
                [Environment]::SetEnvironmentVariable($Matches[1].Trim(), $Matches[2].Trim(), 'Process')
            }
        }
    }
    & $bin 2>&1
} -ArgumentList $rootDir, $releaseBin
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
    Write-Host '[ok] DISTIRA backend running on http://127.0.0.1:8080' -ForegroundColor Green
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
Write-Host '  All DISTIRA services launched!' -ForegroundColor Green
Write-Host ''
Write-Host '  Services:' -ForegroundColor Cyan
if ($ollamaRunning) {
    Write-Host '    Ollama        : http://localhost:11434' -ForegroundColor White
}
Write-Host '    DISTIRA API   : http://localhost:8080' -ForegroundColor White
Write-Host '    Dashboard     : http://localhost:5173' -ForegroundColor White
Write-Host '    VS Code Agent : @distira in Copilot Chat' -ForegroundColor White
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
