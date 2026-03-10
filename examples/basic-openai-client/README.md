# Basic OpenAI-Compatible Client

Point your client at `http://localhost:8080` and use KATARA
as an OpenAI-compatible gateway.

## Usage

```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

> **Note:** The `/v1/chat/completions` endpoint is planned for V7.1.
> Currently only `/healthz` and `/version` are available.
