# Basic OpenAI-Compatible Client

Point your client at `http://localhost:8080` and use KATARA
as an OpenAI-compatible gateway.

## Usage

```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "client_app": "curl demo",
    "upstream_provider": "OpenAI",
    "upstream_model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

The optional lineage fields make the dashboard capable of showing:

- what the user-facing client selected upstream
- what KATARA actually routed downstream

This is especially useful when a client surface says `GPT-5.4`, `Claude Sonnet`, or another upstream model, while KATARA routes to a different sovereign or cloud provider.
