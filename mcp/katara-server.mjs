#!/usr/bin/env node
/**
 * KATARA MCP Server
 *
 * Exposes KATARA gateway endpoints as tools accessible from
 * VS Code Copilot Chat via the Model Context Protocol (stdio transport).
 *
 * Tools:
 *   katara_compile   — Compile context (no LLM call)
 *   katara_chat      — Compile + forward to LLM
 *   katara_providers — List configured providers
 *   katara_metrics   — Get current metrics snapshot
 */

import { createInterface } from "node:readline";

const KATARA_URL = process.env.KATARA_URL || "http://127.0.0.1:8080";

// ── JSON-RPC helpers ──────────────────────────────────

function respond(id, result) {
  const msg = JSON.stringify({ jsonrpc: "2.0", id, result });
  process.stdout.write(`Content-Length: ${Buffer.byteLength(msg)}\r\n\r\n${msg}`);
}

function respondError(id, code, message) {
  const msg = JSON.stringify({ jsonrpc: "2.0", id, error: { code, message } });
  process.stdout.write(`Content-Length: ${Buffer.byteLength(msg)}\r\n\r\n${msg}`);
}

function notify(method, params) {
  const msg = JSON.stringify({ jsonrpc: "2.0", method, params });
  process.stdout.write(`Content-Length: ${Buffer.byteLength(msg)}\r\n\r\n${msg}`);
}

// ── Tool definitions ──────────────────────────────────

const TOOLS = [
  {
    name: "katara_compile",
    description:
      "Send context through the KATARA pipeline (fingerprint → cache → compiler → memory → router) WITHOUT calling the LLM. Returns intent, compiled tokens, routing decision, cache hit status, and efficiency metrics.",
    inputSchema: {
      type: "object",
      properties: {
        context: {
          type: "string",
          description: "The raw context/prompt to compile and optimize.",
        },
        sensitive: {
          type: "boolean",
          description:
            "If true, forces routing to a local-only provider (sovereign mode). Default: false.",
        },
      },
      required: ["context"],
    },
  },
  {
    name: "katara_chat",
    description:
      "Compile context through KATARA then forward to the best LLM provider (Ollama local, Mistral cloud, etc.). Returns an OpenAI-compatible chat completion with a katara section showing optimization stats.",
    inputSchema: {
      type: "object",
      properties: {
        message: {
          type: "string",
          description: "The user message to send to the LLM via KATARA.",
        },
        model: {
          type: "string",
          description:
            "Optional: force a specific model (e.g. 'llama3:latest', 'mistral-ocr-2512'). If omitted, KATARA routes automatically based on intent.",
        },
        sensitive: {
          type: "boolean",
          description:
            "If true, forces routing to a local-only provider. Default: false.",
        },
      },
      required: ["message"],
    },
  },
  {
    name: "katara_providers",
    description: "List all LLM providers configured in KATARA.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "katara_metrics",
    description:
      "Get the current KATARA metrics snapshot: total requests, token counts, efficiency score, cache stats, routing breakdown, and per-intent statistics.",
    inputSchema: { type: "object", properties: {} },
  },
];

// ── Tool execution ────────────────────────────────────

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

async function executeTool(name, args) {
  switch (name) {
    case "katara_compile":
      return callKatara("/v1/compile", "POST", {
        context: args.context,
        sensitive: args.sensitive ?? false,
      });
    case "katara_chat":
      return callKatara("/v1/chat/completions", "POST", {
        messages: [{ role: "user", content: args.message }],
        model: args.model || undefined,
        sensitive: args.sensitive ?? false,
      });
    case "katara_providers":
      return callKatara("/v1/providers");
    case "katara_metrics":
      return callKatara("/v1/metrics");
    default:
      throw new Error(`Unknown tool: ${name}`);
  }
}

// ── MCP protocol handler ──────────────────────────────

async function handleMessage(msg) {
  const { id, method, params } = msg;

  switch (method) {
    case "initialize":
      respond(id, {
        protocolVersion: "2024-11-05",
        capabilities: { tools: {} },
        serverInfo: {
          name: "katara-mcp",
          version: "7.0.1",
        },
      });
      break;

    case "notifications/initialized":
      // Client acknowledged — nothing to do
      break;

    case "tools/list":
      respond(id, { tools: TOOLS });
      break;

    case "tools/call": {
      const { name, arguments: args } = params;
      try {
        const result = await executeTool(name, args || {});
        respond(id, {
          content: [
            { type: "text", text: JSON.stringify(result, null, 2) },
          ],
        });
      } catch (err) {
        respond(id, {
          content: [{ type: "text", text: `Error: ${err.message}` }],
          isError: true,
        });
      }
      break;
    }

    default:
      if (id !== undefined) {
        respondError(id, -32601, `Method not found: ${method}`);
      }
  }
}

// ── stdio transport (Content-Length framing) ──────────

let buffer = "";

process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  buffer += chunk;

  while (true) {
    const headerEnd = buffer.indexOf("\r\n\r\n");
    if (headerEnd === -1) break;

    const header = buffer.slice(0, headerEnd);
    const match = header.match(/Content-Length:\s*(\d+)/i);
    if (!match) {
      buffer = buffer.slice(headerEnd + 4);
      continue;
    }

    const contentLength = parseInt(match[1], 10);
    const bodyStart = headerEnd + 4;

    if (buffer.length < bodyStart + contentLength) break;

    const body = buffer.slice(bodyStart, bodyStart + contentLength);
    buffer = buffer.slice(bodyStart + contentLength);

    try {
      const msg = JSON.parse(body);
      handleMessage(msg);
    } catch {
      // Ignore malformed JSON
    }
  }
});

process.stdin.on("end", () => process.exit(0));
