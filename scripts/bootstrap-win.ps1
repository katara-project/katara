#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host '==> Bootstrapping KATARA on Windows...' -ForegroundColor Cyan

# 1. Check Rust
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    $rustVer = (rustc --version) -replace 'rustc ','' -replace ' .*',''
    Write-Host "[ok] Rust $rustVer" -ForegroundColor Green
} else {
    Write-Host '[!!] Rust not found. Installing via rustup...' -ForegroundColor Yellow
    Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
}

# 2. Check Node.js
if (Get-Command node -ErrorAction SilentlyContinue) {
    Write-Host "[ok] Node.js $(node --version)" -ForegroundColor Green
} else {
    Write-Host '[!!] Node.js not found. Please install Node.js 20+ from https://nodejs.org' -ForegroundColor Red
    exit 1
}

# 3. Build Rust workspace
Write-Host '==> Building Rust workspace...' -ForegroundColor Cyan
cargo build --workspace
Write-Host '[ok] Rust build complete.' -ForegroundColor Green

# 4. Install dashboard dependencies
Write-Host '==> Installing dashboard dependencies...' -ForegroundColor Cyan
Push-Location dashboard/ui-vue
npm install
Pop-Location
Write-Host '[ok] Dashboard dependencies installed.' -ForegroundColor Green

Write-Host ''
Write-Host '==> KATARA bootstrap complete.' -ForegroundColor Cyan
Write-Host '    Run: cargo run -p core       (gateway on :8080)'
Write-Host '    Run: cd dashboard/ui-vue && npm run dev  (dashboard on :5173)'
