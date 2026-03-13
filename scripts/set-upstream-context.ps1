param(
    [string]$ClientApp = "VS Code Copilot Chat",
    [string]$UpstreamProvider,
    [string]$UpstreamModel
)

$runtimeStatePath = Join-Path $PSScriptRoot "..\cache\client-context.json"
$runtimeStateDir = Split-Path $runtimeStatePath -Parent

if (-not (Test-Path $runtimeStateDir)) {
    New-Item -ItemType Directory -Path $runtimeStateDir | Out-Null
}

$payload = @{
    client_app = $ClientApp
    upstream_provider = $UpstreamProvider
    upstream_model = $UpstreamModel
    updated_at = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
}

$payload | ConvertTo-Json | Set-Content -Path $runtimeStatePath -Encoding UTF8
Get-Content $runtimeStatePath -Raw