#!/usr/bin/env node
/**
 * Quick test: spawn katara-server.mjs and send an MCP initialize handshake.
 * Usage:  node mcp/test-mcp.mjs
 */
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const dir = dirname(fileURLToPath(import.meta.url));
const server = spawn("node", [join(dir, "katara-server.mjs")], {
  env: { ...process.env, KATARA_URL: "http://127.0.0.1:8080" },
  stdio: ["pipe", "pipe", "pipe"],
});

server.stderr.on("data", (d) => process.stderr.write("[server stderr] " + d));

// ── Send a JSON-RPC message with Content-Length framing ─────
function send(obj) {
  const body = JSON.stringify(obj);
  const frame = `Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`;
  server.stdin.write(frame);
}

// ── Read Content-Length framed responses ─────────────────────
let buf = Buffer.alloc(0);
const responses = [];

server.stdout.on("data", (chunk) => {
  buf = Buffer.concat([buf, chunk]);
  drain();
});

function drain() {
  while (true) {
    const sep = findSep(buf);
    if (sep === -1) break;
    const header = buf.slice(0, sep).toString("ascii");
    const m = header.match(/Content-Length:\s*(\d+)/i);
    if (!m) { buf = buf.slice(sep + 4); continue; }
    const len = parseInt(m[1], 10);
    const start = sep + 4;
    if (buf.length < start + len) break;
    const body = buf.slice(start, start + len).toString("utf8");
    buf = buf.slice(start + len);
    try {
      const msg = JSON.parse(body);
      responses.push(msg);
      onResponse(msg);
    } catch {}
  }
}

function findSep(b) {
  for (let i = 0; i < b.length - 3; i++) {
    if (b[i] === 13 && b[i+1] === 10 && b[i+2] === 13 && b[i+3] === 10) return i;
  }
  return -1;
}

// ── Test sequence ───────────────────────────────────────────
let step = 0;

function onResponse(msg) {
  if (step === 0) {
    // Response to initialize
    console.log("\n=== INITIALIZE RESPONSE ===");
    console.log(JSON.stringify(msg, null, 2));

    if (msg.result && msg.result.serverInfo) {
      console.log("\n[OK] Server responded to initialize!");
    } else {
      console.log("\n[FAIL] Unexpected initialize response");
    }

    // Send initialized notification
    send({ jsonrpc: "2.0", method: "notifications/initialized" });

    // Now request tools/list
    step = 1;
    send({ jsonrpc: "2.0", id: 2, method: "tools/list", params: {} });
  } else if (step === 1) {
    // Response to tools/list
    console.log("\n=== TOOLS/LIST RESPONSE ===");
    if (msg.result && msg.result.tools) {
      console.log(`[OK] ${msg.result.tools.length} tools available:`);
      for (const t of msg.result.tools) {
        console.log(`  - ${t.name}: ${t.description.slice(0, 60)}...`);
      }
    } else {
      console.log("[FAIL] Unexpected tools/list response");
      console.log(JSON.stringify(msg, null, 2));
    }

    console.log("\n=== ALL TESTS PASSED ===\n");
    server.kill();
    process.exit(0);
  }
}

// Timeout safety
setTimeout(() => {
  console.error("\n[TIMEOUT] No response within 5 seconds.");
  server.kill();
  process.exit(1);
}, 5000);

// Start the handshake
console.log("Sending initialize...");
send({
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: {
    protocolVersion: "2024-11-05",
    capabilities: {},
    clientInfo: { name: "test-harness", version: "1.0.0" },
  },
});
