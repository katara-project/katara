#!/usr/bin/env node
/**
 * KATARA MCP Server (using official @modelcontextprotocol/sdk)
 *
 * Exposes KATARA gateway endpoints as tools accessible from
 * VS Code Copilot Chat via the Model Context Protocol (stdio transport).
 *
 * Tools:
 *   katara_compile   - Compile context (no LLM call)
 *   katara_chat      - Compile + forward to LLM
 *   katara_providers - List configured providers
 *   katara_metrics   - Get current metrics snapshot
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const KATARA_URL = process.env.KATARA_URL || "http://127.0.0.1:8080";

// -- HTTP helper to call KATARA backend -----------------

async function callKatara(path, method = "GET", body = undefined) {
  const opts = {
    method,
    headers: { "Content-Type": "application/json" },
  };
  if (body) opts.body = JSON.stringify(body);

  const res = await fetch(`${KATARA_URL}${path}`, opts);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`KATARA ${res.status}: ${text}`);
  }
  return res.json();
}

// -- Create MCP Server ----------------------------------

const server = new McpServer({
  name: "katara-mcp",
  version: "7.0.1",
});

// Tool: katara_compile
server.tool(
  "katara_compile",
  "Send context through the KATARA pipeline (fingerprint, cache, compiler, memory, router) WITHOUT calling the LLM. Returns intent, compiled tokens, routing decision, cache hit status, and efficiency metrics.",
  {
    context: z.string().describe("The raw context/prompt to compile and optimize."),
    sensitive: z.boolean().optional().default(false).describe("If true, forces routing to a local-only provider (sovereign mode)."),
  },
  async ({ context, sensitive }) => {
    const result = await callKatara("/v1/compile", "POST", { context, sensitive });
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] };
  }
);

// Tool: katara_chat
server.tool(
  "katara_chat",
  "Compile context through KATARA then forward to the best LLM provider (Ollama local, Mistral cloud, etc.). Returns an OpenAI-compatible chat completion with a katara section showing optimization stats.",
  {
    message: z.string().describe("The user message to send to the LLM via KATARA."),
    model: z.string().optional().describe("Optional: force a specific model (e.g. 'llama3:latest', 'mistral-ocr-2512'). If omitted, KATARA routes automatically based on intent."),
    sensitive: z.boolean().optional().default(false).describe("If true, forces routing to a local-only provider."),
  },
  async ({ message, model, sensitive }) => {
    const result = await callKatara("/v1/chat/completions", "POST", {
      messages: [{ role: "user", content: message }],
      model: model || undefined,
      sensitive,
    });
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] };
  }
);

// Tool: katara_providers
server.tool(
  "katara_providers",
  "List all LLM providers configured in KATARA.",
  {},
  async () => {
    const result = await callKatara("/v1/providers");
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] };
  }
);

// Tool: katara_metrics
server.tool(
  "katara_metrics",
  "Get the current KATARA metrics snapshot: total requests, token counts, efficiency score, cache stats, routing breakdown, and per-intent statistics.",
  {},
  async () => {
    const result = await callKatara("/v1/metrics");
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] };
  }
);

// -- Start server with stdio transport ------------------

const transport = new StdioServerTransport();
await server.connect(transport);
