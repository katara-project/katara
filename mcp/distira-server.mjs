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
import { DatabaseSync } from "node:sqlite";
import { homedir, platform } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { z } from "zod";

const DISTIRA_URL = process.env.DISTIRA_URL || "http://127.0.0.1:8080";
const DEFAULT_CLIENT_APP = process.env.DISTIRA_CLIENT_APP || "VS Code Copilot Chat";
const DEFAULT_UPSTREAM_PROVIDER = process.env.DISTIRA_UPSTREAM_PROVIDER;
const DEFAULT_UPSTREAM_MODEL = process.env.DISTIRA_UPSTREAM_MODEL;
const DEFAULT_COPILOT_UPSTREAM_MODEL = process.env.DISTIRA_DEFAULT_COPILOT_MODEL || "GPT-5.3-Codex";
const CLIENT_CONTEXT_CMD = process.env.DISTIRA_CLIENT_CONTEXT_CMD;
const CLIENT_APP_CMD = process.env.DISTIRA_CLIENT_APP_CMD;
const UPSTREAM_PROVIDER_CMD = process.env.DISTIRA_UPSTREAM_PROVIDER_CMD;
const UPSTREAM_MODEL_CMD = process.env.DISTIRA_UPSTREAM_MODEL_CMD;

const MODEL_KEY_HINTS = [
  "model",
  "modelid",
  "model_id",
  "modelname",
  "selectedmodel",
  "selected_model",
  "selectedchatmodel",
  "selected_chat_model",
  "chatmodel",
  "chat_model",
  "copilotmodel",
  "copilot_model",
  "languagemodel",
  "language_model",
  "enginemodel",
  "engine_model",
  "foundationmodel",
  "foundation_model",
  "defaultmodel",
  "active_model",
  "activemodel",
  "sessionmodel",
  "session_model",
  "modelslug",
  "model_slug",
].map((value) => value.toLowerCase());

const PROVIDER_KEY_HINTS = [
  "provider",
  "providerid",
  "provider_id",
  "providername",
  "copilotprovider",
  "copilot_provider",
  "vendor",
  "source",
  "foundationprovider",
  "foundation_provider",
].map((value) => value.toLowerCase());

const CLIENT_KEY_HINTS = [
  "client",
  "clientapp",
  "client_app",
  "application",
  "app",
  "caller",
  "origin",
].map((value) => value.toLowerCase());

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

function keyPathContainsHint(keyPath, hints) {
  const normalizedPath = normalizeKeyPath(keyPath);
  return hints.some((hint) => normalizedPath.includes(hint));
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
  return ["gpt", "claude", "sonnet", "opus", "gemini", "mistral", "llama", "qwen", "deepseek", "o1", "o3", "o4", "codex"].some((token) => normalized.includes(token));
}

function looksLikeProviderName(value) {
  const normalized = value.toLowerCase();
  return ["openai", "github", "copilot", "anthropic", "google", "mistral", "ollama"].some((token) => normalized.includes(token));
}

function looksLikeClientApp(value) {
  const normalized = value.toLowerCase();
  return ["copilot", "vscode", "vs code", "visual studio code", "chat"].some((token) => normalized.includes(token));
}

function collectStringCandidates(value, keyPath = [], out = []) {
  if (typeof value === "string") {
    const trimmed = value.trim();
    if (trimmed && trimmed.length <= 200) {
      out.push({
        path: keyPath.join("."),
        keyPath: [...keyPath],
        value: trimmed,
      });
    }
    return out;
  }

  if (Array.isArray(value)) {
    for (const [index, item] of value.entries()) {
      collectStringCandidates(item, [...keyPath, String(index)], out);
    }
    return out;
  }

  if (value && typeof value === "object") {
    for (const [key, item] of Object.entries(value)) {
      collectStringCandidates(item, [...keyPath, key], out);
    }
  }

  return out;
}

function scoreCandidate(kind, candidate) {
  const value = candidate.value;
  const keyPath = candidate.keyPath;
  const normalizedValue = value.toLowerCase();
  let score = 0;

  if (kind === "model") {
    if (keyPathContainsHint(keyPath, MODEL_KEY_HINTS)) score += 6;
    if (looksLikeModelName(value)) score += 5;
    if (["selected", "active", "current", "default", "session", "chat", "copilot"].some((token) => normalizedValue.includes(token))) score += 1;
  } else if (kind === "provider") {
    if (keyPathContainsHint(keyPath, PROVIDER_KEY_HINTS)) score += 6;
    if (looksLikeProviderName(value)) score += 5;
  } else if (kind === "clientApp") {
    if (keyPathContainsHint(keyPath, CLIENT_KEY_HINTS)) score += 4;
    if (looksLikeClientApp(value)) score += 5;
  }

  return score;
}

function findBestCandidate(extra, kind) {
  const scored = collectStringCandidates(extra)
    .map((candidate) => ({
      ...candidate,
      score: scoreCandidate(kind, candidate),
    }))
    .filter((candidate) => candidate.score > 0)
    .sort((a, b) => b.score - a.score || a.path.length - b.path.length);

  return {
    best: scored[0],
    top: scored.slice(0, 8).map(({ path, value, score }) => ({ path, value, score })),
  };
}

function metaProbePath() {
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = path.dirname(__filename);
  return path.join(__dirname, "..", "cache", "mcp-meta-probe.json");
}

function writeMetaProbe(extra, candidates, requestMeta) {
  try {
    const outputPath = metaProbePath();
    fs.mkdirSync(path.dirname(outputPath), { recursive: true });
    const body = {
      captured_at: new Date().toISOString(),
      meta_keys: Object.keys(extra?._meta || {}),
      request_meta: requestMeta,
      candidates,
    };
    fs.writeFileSync(outputPath, JSON.stringify(body, null, 2), "utf8");
  } catch {
    // best-effort probe only
  }
}

function readGenericMeta(extra) {
  const genericModel = findBestCandidate(extra, "model").best?.value;
  const genericProvider = findBestCandidate(extra, "provider").best?.value;
  const genericClientApp = findBestCandidate(extra, "clientApp").best?.value;

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
  if (normalized.includes("gpt") || normalized.includes("o1") || normalized.includes("o3") || normalized.includes("o4") || normalized.includes("codex")) return "OpenAI";
  if (normalized.includes("gemini")) return "Google";
  if (normalized.includes("mistral")) return "Mistral";
  if (normalized.includes("llama") || normalized.includes("qwen") || normalized.includes("deepseek")) return "Open-source / local";
  return DEFAULT_UPSTREAM_PROVIDER;
}

function nonEmpty(value) {
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed ? trimmed : undefined;
}

function isCopilotClient(clientApp) {
  const normalized = (clientApp || "").toLowerCase();
  return normalized.includes("copilot");
}

// ── VS Code state.vscdb live model detection ───────────────

function resolveVSCodeStateDbPath() {
  const os = platform();
  if (os === "win32")
    return path.join(process.env.APPDATA || path.join(homedir(), "AppData", "Roaming"), "Code", "User", "globalStorage", "state.vscdb");
  if (os === "darwin")
    return path.join(homedir(), "Library", "Application Support", "Code", "User", "globalStorage", "state.vscdb");
  return path.join(homedir(), ".config", "Code", "User", "globalStorage", "state.vscdb");
}

const VSCODE_STATE_DB_PATH = resolveVSCodeStateDbPath();

/** Cache: { value, readAt } — refreshed at most every 3 seconds. */
let _vscodeLmCache = { value: null, readAt: 0 };
const VSCODE_LM_CACHE_TTL_MS = 3_000;

/**
 * Read the currently selected chat model from VS Code's state.vscdb.
 * Returns e.g. "copilot/claude-opus-4.6" or undefined.
 */
function readVSCodeCurrentModel() {
  const now = Date.now();
  if (_vscodeLmCache.value !== null && now - _vscodeLmCache.readAt < VSCODE_LM_CACHE_TTL_MS) {
    return _vscodeLmCache.value || undefined;
  }
  try {
    if (!fs.existsSync(VSCODE_STATE_DB_PATH)) return undefined;
    const db = new DatabaseSync(VSCODE_STATE_DB_PATH, { readOnly: true });
    try {
      const row = db.prepare("SELECT value FROM ItemTable WHERE key = ?").get("chat.currentLanguageModel.panel");
      const raw = row?.value ?? "";
      _vscodeLmCache = { value: raw, readAt: now };
      return raw || undefined;
    } finally {
      db.close();
    }
  } catch {
    return undefined;
  }
}

/**
 * Parse a VS Code model identifier into a human-readable model name and provider.
 * e.g. "copilot/claude-opus-4.6" → { model: "Claude Opus 4.6", provider: "Anthropic" }
 *      "ollama/Ollama/qwen2.5-coder:7b" → { model: "qwen2.5-coder:7b", provider: "Open-source / local" }
 */
function parseVSCodeModelId(raw) {
  if (!raw) return {};
  const parts = raw.split("/");
  const slug = parts[parts.length - 1];
  // "auto" means the user hasn't picked a specific model — treat as unknown.
  if (slug === "auto") return {};
  // Humanize: replace hyphens with spaces, title-case, fix known acronyms.
  const humanized = slug
    .replace(/-/g, " ")
    .replace(/\b\w/g, (c) => c.toUpperCase())
    .replace(/\bGpt\b/g, "GPT");
  return {
    model: humanized,
    provider: inferUpstreamProvider(slug),
  };
}

function maybeSyncRuntimeClientContext(metadata, backendContext) {
  const changedClient = metadata.client_app && metadata.client_app !== backendContext.clientApp;
  const changedProvider = metadata.upstream_provider && metadata.upstream_provider !== backendContext.upstreamProvider;
  const changedModel = metadata.upstream_model && metadata.upstream_model !== backendContext.upstreamModel;

  if (!changedClient && !changedProvider && !changedModel) return;

  callDistira("/v1/runtime/client-context", "POST", {
    client_app: metadata.client_app,
    upstream_provider: metadata.upstream_provider,
    upstream_model: metadata.upstream_model,
  }).catch(() => {
    // best-effort sync only; never block the main tool request
  });
}

async function buildUpstreamMetadata({ clientApp, upstreamProvider, upstreamModel, model }, extra) {
  const requestMeta = readRequestMeta(extra);
  const candidateProbe = {
    model: findBestCandidate(extra, "model").top,
    provider: findBestCandidate(extra, "provider").top,
    clientApp: findBestCandidate(extra, "clientApp").top,
  };
  writeMetaProbe(extra, candidateProbe, requestMeta);
  const backendContext = await readBackendContext();
  const dynamicContext = readDynamicContext();

  const resolvedClientApp =
    nonEmpty(clientApp) ||
    nonEmpty(requestMeta.clientApp) ||
    nonEmpty(dynamicContext.clientApp) ||
    nonEmpty(backendContext.clientApp) ||
    DEFAULT_CLIENT_APP;

  // ── VS Code live model detection (reads state.vscdb) ──
  const vscodeLiveId = readVSCodeCurrentModel();
  const vscodeParsed = parseVSCodeModelId(vscodeLiveId);

  // Precedence: explicit params > request metadata > VS Code live state > dynamic command > backend context > defaults.
  // `model` (distira_chat arg) is treated as an explicit upstream model hint.
  const resolvedModel =
    nonEmpty(upstreamModel) ||
    nonEmpty(model) ||
    nonEmpty(requestMeta.upstreamModel) ||
    nonEmpty(vscodeParsed.model) ||
    nonEmpty(dynamicContext.upstreamModel) ||
    nonEmpty(backendContext.upstreamModel) ||
    nonEmpty(DEFAULT_UPSTREAM_MODEL);

  const inferredProvider = inferUpstreamProvider(resolvedModel);
  const resolvedProvider =
    nonEmpty(upstreamProvider) ||
    nonEmpty(requestMeta.upstreamProvider) ||
    nonEmpty(vscodeParsed.provider) ||
    nonEmpty(dynamicContext.upstreamProvider) ||
    nonEmpty(inferredProvider) ||
    nonEmpty(backendContext.upstreamProvider) ||
    nonEmpty(DEFAULT_UPSTREAM_PROVIDER);

  const normalizedProvider =
    resolvedProvider === "GitHub Copilot" && inferredProvider && inferredProvider !== "GitHub Copilot"
      ? inferredProvider
      : resolvedProvider;

  const metadata = {
    client_app: resolvedClientApp,
    upstream_provider: normalizedProvider,
    upstream_model: resolvedModel,
  };

  maybeSyncRuntimeClientContext(metadata, backendContext);
  return metadata;
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
  "Send context through the DISTIRA pipeline (fingerprint, cache, compiler, memory, router) WITHOUT calling the LLM. Returns intent, compiled tokens, routing decision, cache hit status, and efficiency metrics. Supports slash commands: /debug, /code, /review, /summarize, /translate, /ocr, /dtlr (force local), /fast, /quality, /general — prefix your context to override intent detection.",
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
  "Compile context through DISTIRA then forward to the best LLM provider (Ollama local, Mistral cloud, etc.). Returns an OpenAI-compatible chat completion with a distira section showing optimization stats. Supports slash commands: /debug, /code, /review, /summarize, /translate, /ocr, /dtlr (force local), /fast, /quality — prefix your message to override intent routing.",
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
    autoCompile(`distira_set_client_context: ${clientApp ?? ""} ${upstreamProvider ?? ""} ${upstreamModel ?? ""}`.trim(), extra);
    const result = await callDistira("/v1/runtime/client-context", "POST", {
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

// -- Auto-detect VS Code model polling ------------------

let _lastPolledModelId = null;
setInterval(async () => {
  try {
    const rawId = readVSCodeCurrentModel();
    if (!rawId || rawId === _lastPolledModelId) return;

    _lastPolledModelId = rawId;
    const parsed = parseVSCodeModelId(rawId);
    if (!parsed.model) return;

    const backendContext = await readBackendContext();
    const inferred = inferUpstreamProvider(parsed.model, parsed.provider);
    const resolvedProvider = parsed.provider || inferred;
    const normalizedProvider =
      resolvedProvider === "GitHub Copilot" && inferred && inferred !== "GitHub Copilot"
        ? inferred
        : resolvedProvider;

    maybeSyncRuntimeClientContext(
      {
        client_app: DEFAULT_CLIENT_APP,
        upstream_provider: normalizedProvider,
        upstream_model: parsed.model,
      },
      backendContext
    );
  } catch {
    // best-effort silent loop
  }
}, 3000);

// -- Start server with stdio transport ------------------

const transport = new StdioServerTransport();
await server.connect(transport);
