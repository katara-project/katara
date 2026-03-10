$project="katara"

Write-Host "Creating KATARA v6.7 project..."

mkdir $project -Force

$dirs = @(
"core/gateway",
"core/compiler",
"core/memory",
"core/router",
"core/adapters",
"core/metrics",
"dashboard/ui-vue/src/components",
"dashboard/ui-vue/src/views",
"dashboard/ui-vue/src/router",
"dashboard/ui-vue/src/store",
"docs",
"examples/basic-client",
"examples/ollama-routing",
"examples/hybrid-routing",
"benchmarks/token-reduction",
"benchmarks/latency",
"deployments/docker",
"deployments/kubernetes",
"deployments/helm",
"configs/providers",
"configs/routing",
"configs/policies",
".github/workflows",
"scripts"
)

foreach ($dir in $dirs) {
mkdir "$project/$dir" -Force
}

# README
@"
# KATARA

KATARA is an open-source sovereign AI gateway that compiles the smallest useful context before every LLM call.

## Core innovations

- Context Budget Compiler
- Context Memory Lensing
- AI Flow Visualizer
- Hybrid LLM Routing
- Efficiency Score
"@ | Out-File "$project/README.md"

# ROADMAP
@"
# ROADMAP

V6.1 Gateway
V6.2 Hybrid routing
V6.3 Rebranding
V6.4 Context Budget Compiler
V6.5 Optimization layer
V6.6 Flow Visualizer
V6.7 Windows monorepo
"@ | Out-File "$project/ROADMAP.md"

# CHANGELOG
@"
# CHANGELOG

## v6.7
- Monorepo structure
- Vue dashboard scaffold
- Windows bootstrap
"@ | Out-File "$project/CHANGELOG.md"

# SECURITY
@"
# SECURITY

Please report vulnerabilities via GitHub Security Advisories.
"@ | Out-File "$project/SECURITY.md"

# CONTRIBUTING
@"
# CONTRIBUTING

Fork the repo and submit a pull request.
"@ | Out-File "$project/CONTRIBUTING.md"

# VERSION
"6.7.0" | Out-File "$project/VERSION"

# example rust files
New-Item "$project/core/gateway/main.rs" -ItemType File
New-Item "$project/core/compiler/context_budget_compiler.rs" -ItemType File
New-Item "$project/core/memory/context_block.rs" -ItemType File
New-Item "$project/core/router/model_selector.rs" -ItemType File

# vue placeholders
New-Item "$project/dashboard/ui-vue/src/components/FlowVisualizer.vue" -ItemType File
New-Item "$project/dashboard/ui-vue/src/components/EfficiencyGauge.vue" -ItemType File
New-Item "$project/dashboard/ui-vue/src/views/DashboardView.vue" -ItemType File

Write-Host "KATARA v6.7 project created!"