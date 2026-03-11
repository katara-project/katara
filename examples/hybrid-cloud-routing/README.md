# Hybrid Cloud Routing

This example demonstrates the default hybrid routing strategy:
local-first with cloud fallback.

## How it works

1. Non-sensitive requests go to `ollama-local` by default.
2. If the local provider is unavailable, the router falls back to `openai-compatible`.
3. Debug-intent requests are routed to `mistral-cloud` for cost-efficiency.

See `configs/routing/routing.yaml` and `configs/policies/policies.yaml`
for the full configuration.
