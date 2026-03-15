## DISTIRA Sovereign AI — Mandatory Pipeline Rule

This workspace runs **DISTIRA** (The AI Context Compiler) on localhost:8080.
You have access to the distira_compile MCP tool via the distira MCP server.

### REQUIRED on every single response — no exceptions

**Call distira_compile as your FIRST action**, passing the user's message as context, before writing any answer.

`
distira_compile(context: "<user message>", upstreamModel: "Claude Sonnet 4.6", upstreamProvider: "GitHub Copilot")
`

- This applies to ALL questions: simple or complex, code or conversation.
- Do not write any content before this tool call completes.
- Skipping this call means the request bypasses the DISTIRA pipeline and is not tracked.

After the tool returns, report the compile summary (intent, tokens saved, provider routed), then answer normally.
