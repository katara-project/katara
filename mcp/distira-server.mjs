#!/usr/bin/env node
/**
 * DISTIRA MCP Server (using official @modelcontextprotocol/sdk)
 *
 * Exposes DISTIRA gateway endpoints as tools accessible from
 * VS Code Copilot Chat via the Model Context Protocol (stdio transport).
 *
 * Tools:
 *   distira_compile   - Compile context (no LLM call)
 *   distira_chat      - Compile + forward to LLM
 *   distira_providers - List configured providers
 *   distira_metrics   - Get current metrics snapshot
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { z } from "zod";

const DISTIRA_URL = process.env.DISTIRA_URL || "http://127.0.0.1:8080";
const DEFAULT_CLIENT_APP = process.env.DISTIRA_CLIENT_APP || "VS Code Copilot Chat";
const DEFAULT_UPSTREAM_PROVIDER = process.env.DISTIRA_UPSTREAM_PROVIDER;
const DEFAULT_UPSTREAM_MODEL = process.env.DISTIRA_UPSTREAM_MODEL;
const CLIENT_CONTEXT_CMD = process.env.DISTIRA_CLIENT_CONTEXT_CMD;
const CLIENT_APP_CMD = process.env.DISTIRA_CLIENT_APP_CMD;
const UPSTREAM_PROVIDER_CMD = process.env.DISTIRA_UPSTREAM_PROVIDER_CMD;
const UPSTREAM_MODEL_CMD = process.env.DISTIRA_UPSTREAM_MODEL_CMD;

function readDistiraVersion() {
  try {
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);
    const versionPath = path.join(__dirname, "..", "VERSION");
    if (fs.existsSync(versionPath)) {
      const raw = fs.readFileSync(versionPath, "utf8").trim();
      if (raw) return raw;
    }
  } catch {
    // fall through to default
  }
  return "8.0.0";
}

function readCommand(command) {
  if (!command) return undefined;
  try {
    const output = execSync(command, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 1500,
      windowsHide: true,
    }).trim();
    return output || undefined;
  } catch {
    return undefined;
  }
}

function readDynamicContext() {
  if (CLIENT_CONTEXT_CMD) {
    const raw = readCommand(CLIENT_CONTEXT_CMD);
    if (raw) {
      try {
        const parsed = JSON.parse(raw);
        return {
          clientApp: parsed.client_app,
          upstreamProvider: parsed.upstream_provider,
          upstreamModel: parsed.upstream_model,
        };
      } catch {
        return {
          upstreamModel: raw,
        };
      }
    }
  }

  return {
    clientApp: readCommand(CLIENT_APP_CMD),
    upstreamProvider: readCommand(UPSTREAM_PROVIDER_CMD),
    upstreamModel: readCommand(UPSTREAM_MODEL_CMD),
  };
}

function normalizeKeyPath(keyPath) {
  return keyPath.join(".").toLowerCase();
}

function findFirstString(value, predicate, keyPath = []) {
  if (typeof value === "string") {
    return predicate(value, keyPath) ? value : undefined;
  }

  if (Array.isArray(value)) {
    for (const [index, item] of value.entries()) {
      const found = findFirstString(item, predicate, [...keyPath, String(index)]);
      if (found) return found;
    }
    return undefined;
  }

  if (value && typeof value === "object") {
    for (const [key, item] of Object.entries(value)) {
      const found = findFirstString(item, predicate, [...keyPath, key]);
      if (found) return found;
    }
  }

  return undefined;
}

function looksLikeModelName(value) {
  const normalized = value.toLowerCase();
  return ["gpt", "claude", "sonnet", "opus", "gemini", "mistral", "llama", "qwen", "deepseek", "o1", "o3", "o4"].some((token) => normalized.includes(token));
}

function looksLikeProviderName(value) {
  const normalized = value.toLowerCase();
  return ["openai", "github", "copilot", "anthropic", "google", "mistral", "ollama"].some((token) => normalized.includes(token));
}

function readGenericMeta(extra) {
  const keyHintsForModel = [
    "model",
    "modelid",
    "model_id",
    "modelname",
    "selectedmodel",
    "selected_model",
    "chatmodel",
    "chat_model",
    "copilotmodel",
    "copilot_model",
    "languageModel",
    "language_model",
  ].map((value) => value.toLowerCase());
  const keyHintsForProvider = [
    "provider",
    "providerid",
    "provider_id",
    "providername",
    "copilotprovider",
    "copilot_provider",
    "vendor",
    "source",
  ].map((value) => value.toLowerCase());

  const genericModel = findFirstString(extra, (candidate, keyPath) => {
    const normalizedPath = normalizeKeyPath(keyPath);
    return keyHintsForModel.some((hint) => normalizedPath.includes(hint.toLowerCase())) && looksLikeModelName(candidate);
  });

  const genericProvider = findFirstString(extra, (candidate, keyPath) => {
    const normalizedPath = normalizeKeyPath(keyPath);
    return keyHintsForProvider.some((hint) => normalizedPath.includes(hint.toLowerCase())) && looksLikeProviderName(candidate);
  });

  const genericClientApp = findFirstString(extra, (candidate, keyPath) => {
    const normalizedPath = normalizeKeyPath(keyPath);
    return normalizedPath.includes("client") || normalizedPath.includes("application") || normalizedPath.includes("app");
  });

  return {
    clientApp: genericClientApp,
    upstreamProvider: genericProvider,
    upstreamModel: genericModel,
  };
}

function readRequestMeta(extra) {
  const meta = extra?._meta || {};
  const generic = readGenericMeta(extra);
  return {
    clientApp:
      typeof meta["distira/client_app"] === "string"
        ? meta["distira/client_app"]
        : generic.clientApp,
    upstreamProvider:
      typeof meta["distira/upstream_provider"] === "string"
        ? meta["distira/upstream_provider"]
        : generic.upstreamProvider,
    upstreamModel:
      typeof meta["distira/upstream_model"] === "string"
        ? meta["distira/upstream_model"]
        : generic.upstreamModel,
  };
}

function inferUpstreamProvider(model) {
  const normalized = (model || "").toLowerCase();
  if (!normalized) return DEFAULT_UPSTREAM_PROVIDER;
  if (normalized.includes("claude")) return "Anthropic";
  if (normalized.includes("gpt") || normalized.includes("o1") || normalized.includes("o3")) return "OpenAI-family";
  if (normalized.includes("gemini")) return "Google";
  if (normalized.includes("mistral")) return "Mistral";
  if (normalized.includes("llama") || normalized.includes("qwen") || normalized.includes("deepseek")) return "Open-source / local";
  return DEFAULT_UPSTREAM_PROVIDER;
}

async function buildUpstreamMetadata({ clientApp, upstreamProvider, upstreamModel, model }, extra) {
  const requestMeta = readRequestMeta(extra);
  const backendContext = await readBackendContext();
  const dynamicContext = readDynamicContext();
  const resolvedModel = upstreamModel || requestMeta.upstreamModel || backendContext.upstreamModel || dynamicContext.upstreamModel || DEFAULT_UPSTREAM_MODEL;
  return {
    client_app: clientApp || requestMeta.clientApp || backendContext.clientApp || dynamicContext.clientApp || DEFAULT_CLIENT_APP,
    upstream_provider: upstreamProvider || requestMeta.upstreamProvider || backendContext.upstreamProvider || dynamicContext.upstreamProvider || inferUpstreamProvider(resolvedModel),
    upstream_model: resolvedModel,
  };
}

// -- HTTP helper to call DISTIRA backend -----------------

async function callDistira(path, method = "GET", body = undefined) {
  const opts = {
    method,
    headers: { "Content-Type": "application/json" },
  };
  if (body) opts.body = JSON.stringify(body);

  const res = await fetch(`${DISTIRA_URL}${path}`, opts);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`DISTIRA ${res.status}: ${text}`);
  }
  return res.json();
}

async function readBackendContext() {
  try {
    const result = await callDistira("/v1/runtime/client-context");
    return {
      clientApp: result.client_app,
      upstreamProvider: result.upstream_provider,
      upstreamModel: result.upstream_model,
    };
  } catch {
    return {};
  }
}

// -- Auto-compile helper --------------------------------
// Fires a background compile call for session tracking whenever any
// distira tool is invoked. Never blocks the actual tool response.
async function autoCompile(context, extra) {
  try {
    const meta = await buildUpstreamMetadata({}, extra ?? {});
    callDistira("/v1/compile", "POST", { context, ...meta }).catch(() => {});
  } catch {
    // fire-and-forget — ignore all errors
  }
}

// -- Create MCP Server ----------------------------------

const server = new McpServer({
  name: "distira-mcp",
  version: readDistiraVersion(),
});

// Tool: distira_compile
server.tool(
  "distira_compile",
  "Send context through the DISTIRA pipeline (fingerprint, cache, compiler, memory, router) WITHOUT calling the LLM. Returns intent, compiled tokens, routing decision, cache hit status, and efficiency metrics.",
  {
    context: z.string().describe("The raw context/prompt to compile and optimize."),
    sensitive: z.boolean().optional().default(false).describe("If true, forces routing to a local-only provider (sovereign mode)."),
    clientApp: z.string().optional().describe("Optional client application label. Defaults to 'VS Code Copilot Chat'."),
    upstreamProvider: z.string().optional().describe("Optional upstream provider label. Defaults from env or inferred when possible."),
    upstreamModel: z.string().optional().describe("Optional upstream model label selected by the user before DISTIRA routing."),
  },
  async ({ context, sensitive, clientApp, upstreamProvider, upstreamModel }, extra) => {
    const result = await callDistira("/v1/compile", "POST", {
      context,
      sensitive,
      ...await buildUpstreamMetadata({
        clientApp,
        upstreamProvider,
        upstreamModel,
      }, extra),
    });
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] };
  }
);

// Tool: distira_chat  (compile is done server-side inside /v1/chat/completions)
server.tool(
  "distira_chat",
  "Compile context through DISTIRA then forward to the best LLM provider (Ollama local, Mistral cloud, etc.). Returns an OpenAI-compatible chat completion with a distira section showing optimization stats.",
  {
    message: z.string().describe("The user message to send to the LLM via DISTIRA."),
    model: z.string().optional().describe("Optional: force a specific model (e.g. 'llama3:latest', 'mistral-ocr-2512'). If omitted, DISTIRA routes automatically based on intent."),
    sensitive: z.boolean().optional().default(false).describe("If true, forces routing to a local-only provider."),
    clientApp: z.string().optional().describe("Optional client application label. Defaults to 'VS Code Copilot Chat'."),
    upstreamProvider: z.string().optional().describe("Optional upstream provider label. Defaults from env or inferred when possible."),
    upstreamModel: z.string().optional().describe("Optional upstream model label selected by the user before DISTIRA routing."),
  },
  async ({ message, model, sensitive, clientApp, upstreamProvider, upstreamModel }, extra) => {
    const result = await callDistira("/v1/chat/completions", "POST", {
      messages: [{ role: "user", content: message }],
      model: model || undefined,
      sensitive,
      ...await buildUpstreamMetadata({
        clientApp,
        upstreamProvider,
        upstreamModel,
        model,
      }, extra),
    });
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] };
  }
);

server.tool(
  "distira_set_client_context",
  "Update the live upstream client context that Distira should associate with subsequent requests when the client itself cannot send the selected model dynamically.",
  {
    clientApp: z.string().optional().describe("Client application label, for example 'VS Code Copilot Chat'."),
    upstreamProvider: z.string().optional().describe("Upstream provider label, for example 'Anthropic' or 'GitHub Copilot'."),
    upstreamModel: z.string().optional().describe("Upstream model label, for example 'Claude Sonnet 4.6' or 'GPT-5.4'."),
  },
  async ({ clientApp, upstreamProvider, upstreamModel }, extra) => {
    autoCompile(`distira_set_client_context: ${clientApp ?? ""} ${upstreamProvider ?? ""} ${upstreamModel ?? ""}`.trim(), extra);\n    const result = await callDistira("/v1/runtime/client-context", "POST", {
      client_app: clientApp,
      upstream_provider: upstreamProvider,
      upstream_model: upstreamModel,
    });
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] };
  }
);

// Tool: distira_providers
server.tool(
  "distira_providers",
  "List all LLM providers configured in DISTIRA.",
  {},
  async (_args, extra) => {
    autoCompile("distira_providers: list configured providers", extra);
    const result = await callDistira("/v1/providers");
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] };
  }
);

// Tool: distira_metrics
server.tool(
  "distira_metrics",
  "Get the current DISTIRA metrics snapshot: total requests, token counts, efficiency score, cache stats, routing breakdown, and per-intent statistics.",
  {},
  async (_args, extra) => {
    autoCompile("distira_metrics: get current metrics snapshot", extra);
    const result = await callDistira("/v1/metrics");
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] };
  }
);

// -- Start server with stdio transport ------------------

const transport = new StdioServerTransport();
await server.connect(transport);
