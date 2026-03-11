# Ollama Local Routing

This example demonstrates routing sensitive context to a local Ollama instance.

## Prerequisites

- [Ollama](https://ollama.ai) running on `localhost:11434`
- KATARA core running on `localhost:8080`

## How it works

1. The request is tagged as `sensitive: true`.
2. The router selects `ollama-local` based on the sensitivity policy.
3. Context never leaves the local network.

See `configs/routing/routing.yaml` for the routing strategy.
