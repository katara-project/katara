#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host ''
Write-Host '  DISTIRA - Sovereign AI Context Operating System' -ForegroundColor Cyan
Write-Host '  Bootstrap script for Windows' -ForegroundColor Cyan
Write-Host '  ----------------------------------------------' -ForegroundColor DarkGray
Write-Host ''

$warnings = @()

# -- 1. Check Rust --------------------------------------
# Ensure ~/.cargo/bin is in PATH for this session
if (Test-Path "$env:USERPROFILE\.cargo\bin") {
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
}

if (Get-Command cargo -ErrorAction SilentlyContinue) {
    $rustVer = (rustc --version) -replace 'rustc ','' -replace ' .*',''
    Write-Host "[ok] Rust $rustVer" -ForegroundColor Green
} else {
    Write-Host '[!!] Rust not found. Installing via rustup...' -ForegroundColor Yellow
    Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
}

# -- 2. Check Node.js (>= 20 required for MCP server) --
if (Get-Command node -ErrorAction SilentlyContinue) {
    $nodeVer = (node --version) -replace '^v',''
    $nodeMajor = [int]($nodeVer -split '\.')[0]
    if ($nodeMajor -ge 20) {
        Write-Host "[ok] Node.js v$nodeVer" -ForegroundColor Green
    } else {
        Write-Host "[!!] Node.js v$nodeVer detected but >= 20 required (MCP server needs native fetch)." -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host '[!!] Node.js not found. Please install Node.js 20+ from https://nodejs.org' -ForegroundColor Red
    exit 1
}

# -- 3. Check Ollama (optional but recommended) ---------
if (Get-Command ollama -ErrorAction SilentlyContinue) {
    $ollamaVer = (ollama --version) 2>&1
    Write-Host "[ok] Ollama: $ollamaVer" -ForegroundColor Green
} else {
    $warnings += 'Ollama not found. Install from https://ollama.com/download for local model routing.'
    Write-Host '[--] Ollama not found (optional - needed for local models).' -ForegroundColor Yellow
}

# -- 4. Verify config files exist -----------------------
Write-Host ''
Write-Host '==> Checking configuration files...' -ForegroundColor Cyan

$configFiles = @(
    'configs/providers/providers.yaml',
    'configs/routing/routing.yaml',
    'configs/policies/policies.yaml'
)
foreach ($cfg in $configFiles) {
    if (Test-Path $cfg) {
        Write-Host "[ok] $cfg" -ForegroundColor Green
    } else {
        Write-Host "[!!] Missing: $cfg" -ForegroundColor Red
        $warnings += "Configuration file missing: $cfg - see INSTALL.md for examples."
    }
}

# -- 5. Create .env from .env.example if absent ---------
if (-not (Test-Path '.env')) {
    if (Test-Path '.env.example') {
        Copy-Item '.env.example' '.env'
        Write-Host '[ok] Created .env from .env.example (edit it to add your API keys).' -ForegroundColor Green
        $warnings += '.env created from template - edit it to set your real API keys before using cloud providers.'
    } else {
        Write-Host '[--] No .env.example found, skipping .env creation.' -ForegroundColor Yellow
    }
} else {
    Write-Host '[ok] .env already exists.' -ForegroundColor Green
}

# -- 6. Build Rust workspace ----------------------------
Write-Host ''
Write-Host '==> Building Rust workspace (8 crates)...' -ForegroundColor Cyan
cargo build --workspace
Write-Host '[ok] Rust build complete.' -ForegroundColor Green

# -- 7. Install dashboard dependencies ------------------
Write-Host ''
Write-Host '==> Installing dashboard dependencies...' -ForegroundColor Cyan
Push-Location dashboard/ui-vue
npm install
Pop-Location
Write-Host '[ok] Dashboard dependencies installed.' -ForegroundColor Green

# -- 7b. Install MCP server dependencies ---------------
Write-Host ''
Write-Host '==> Installing MCP server dependencies...' -ForegroundColor Cyan
Push-Location mcp
npm install
Pop-Location
Write-Host '[ok] MCP server dependencies installed.' -ForegroundColor Green

# -- 8. Pull Ollama models (if Ollama is available) -----
if (Get-Command ollama -ErrorAction SilentlyContinue) {
    Write-Host ''
    Write-Host '==> Checking Ollama models declared in providers.yaml...' -ForegroundColor Cyan

    $providerFile = 'configs/providers/providers.yaml'
    if (Test-Path $providerFile) {
        # Get list of already-installed models
        $installed = @()
        try {
            $ollamaList = ollama list 2>&1
            $installed = $ollamaList | ForEach-Object {
                if ($_ -match '^(\S+)') { $Matches[1] }
            } | Where-Object { $_ -and $_ -ne 'NAME' }
        } catch { }

        # Parse required model names from providers.yaml (skip cloud-only models)
        $required = Select-String -Path $providerFile -Pattern '^\s+model:\s+(.+)$' |
            ForEach-Object { $_.Matches[0].Groups[1].Value.Trim() } |
            Where-Object { $_ -and $_ -notmatch 'mistral-ocr' } |
            Sort-Object -Unique

        foreach ($model in $required) {
            # Check if model is already installed (match with or without tag)
            $found = $installed | Where-Object { $_ -eq $model -or $_ -like "${model}:*" -or $model -like "${_}*" }
            if ($found) {
                Write-Host "  [ok] $model (already installed)" -ForegroundColor Green
            } else {
                Write-Host "  Pulling $model ..." -ForegroundColor DarkGray
                ollama pull $model
                Write-Host "  [ok] $model" -ForegroundColor Green
            }
        }
    }
} else {
    $warnings += 'Skipped Ollama model pull (Ollama not installed).'
}

# -- 9. Verify MCP server module syntax -----------------
Write-Host ''
Write-Host '==> Verifying MCP server module...' -ForegroundColor Cyan
$mcpCheck = node --check mcp/distira-server.mjs 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host '[ok] MCP server (mcp/distira-server.mjs) syntax OK.' -ForegroundColor Green
} else {
    Write-Host "[!!] MCP server failed to load: $mcpCheck" -ForegroundColor Red
    $warnings += 'MCP server module failed to load - check mcp/distira-server.mjs.'
}

# -- Summary --------------------------------------------
Write-Host ''
Write-Host '  ----------------------------------------------' -ForegroundColor DarkGray
Write-Host '  DISTIRA bootstrap complete!' -ForegroundColor Green
Write-Host ''

if ($warnings.Count -gt 0) {
    Write-Host '  Warnings:' -ForegroundColor Yellow
    foreach ($w in $warnings) {
        Write-Host "    - $w" -ForegroundColor Yellow
    }
    Write-Host ''
}

Write-Host '  Next steps:' -ForegroundColor Cyan
Write-Host '    Start everything:  .\scripts\start-win.ps1' -ForegroundColor White
Write-Host '    Or manually:' -ForegroundColor DarkGray
Write-Host '      1. ollama serve' -ForegroundColor DarkGray
Write-Host '      2. cargo run -p core' -ForegroundColor DarkGray
Write-Host '      3. cd dashboard/ui-vue; npm run dev' -ForegroundColor DarkGray
Write-Host '      4. @distira in Copilot Chat' -ForegroundColor DarkGray
Write-Host ''
