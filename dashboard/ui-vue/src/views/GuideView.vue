<template>
  <div class="guide-view">
    <header class="view-header">
      <div>
        <h2>Guide</h2>
        <p class="muted">Complete documentation for DISTIRA — The AI Context Compiler.</p>
      </div>
    </header>

    <!-- Table of Contents -->
    <nav class="card guide-toc">
      <h3>Table of Contents</h3>
      <ol>
        <li><a href="#what-is-distira" @click.prevent="scrollTo('what-is-distira')">What is DISTIRA?</a></li>
        <li><a href="#architecture" @click.prevent="scrollTo('architecture')">Architecture</a></li>
        <li><a href="#pipeline" @click.prevent="scrollTo('pipeline')">Compilation Pipeline</a></li>
        <li><a href="#intents" @click.prevent="scrollTo('intents')">Intents &amp; Routing</a></li>
        <li><a href="#RCT2I" @click.prevent="scrollTo('RCT2I')">RCT2I Prompt Restructuring</a></li>
        <li><a href="#api-reference" @click.prevent="scrollTo('api-reference')">REST API Reference</a></li>
        <li><a href="#mcp" @click.prevent="scrollTo('mcp')">MCP Integration</a></li>
        <li><a href="#dashboard" @click.prevent="scrollTo('dashboard')">Dashboard Views</a></li>
        <li><a href="#configuration" @click.prevent="scrollTo('configuration')">Configuration</a></li>
        <li><a href="#providers" @click.prevent="scrollTo('providers')">Providers &amp; Compatibility</a></li>
        <li><a href="#troubleshooting" @click.prevent="scrollTo('troubleshooting')">Troubleshooting</a></li>
      </ol>
    </nav>

    <!-- Section 1: What is DISTIRA -->
    <section id="what-is-distira" class="card guide-section">
      <h3><SvgIcon name="zap" :size="20" /> What is DISTIRA?</h3>
      <p>
        <strong>DISTIRA is The AI Context Compiler.</strong> It compiles, minimizes, and governs context
        before every LLM call. Unlike proxies or gateways, DISTIRA touches the context itself — reducing
        tokens, reusing stable blocks, and enforcing sovereignty policies before any model sees the data.
      </p>
      <div class="guide-highlight">
        <p><strong>The problem:</strong> Every LLM call carries too much context — logs, traces, diffs,
        histories, noise. You pay for tokens that don't contribute to the answer.</p>
        <p><strong>DISTIRA's answer:</strong> Compile the context down to signal. Route intelligently.
        Prove every saving.</p>
      </div>
      <h4>The 4 Building Blocks</h4>
      <table class="guide-table">
        <thead><tr><th>Block</th><th>Purpose</th></tr></thead>
        <tbody>
          <tr><td><strong>Context Budget Compiler</strong></td><td>Reduces logs, stack traces, diffs, and conversation histories. Extracts signal, removes noise, cuts tokens before they reach a model.</td></tr>
          <tr><td><strong>Context Memory Lensing</strong></td><td>Builds a structured memory of stable context blocks. Sends only what is new, changed, or still relevant (delta-first forwarding).</td></tr>
          <tr><td><strong>Hybrid Sovereign Routing</strong></td><td>Chooses the right provider — local, private, or cloud — based on confidentiality, cost, latency, policy, and capability. Sensitive data never leaves local.</td></tr>
          <tr><td><strong>AI Flow Visualizer</strong></td><td>Makes every optimization step visible in a live dashboard. Shows before/after, cloud vs local, reused context, and real efficiency gains.</td></tr>
        </tbody>
      </table>
    </section>

    <!-- Section 2: Architecture -->
    <section id="architecture" class="card guide-section">
      <h3><SvgIcon name="layers" :size="20" /> Architecture</h3>
      <p>DISTIRA is a Rust monorepo with specialized crates, a Vue 3 dashboard, and an MCP server for IDE integration.</p>
      <div class="guide-diagram">
        <pre>
Clients / IDE / Agents
        │
  OpenAI-compatible API
        │
      DISTIRA
        │
  ┌─────┴─────┐
  │  Intent    │
  │  Detector  │
  └─────┬─────┘
        │
  ┌─────┴──────────┐
  │ Context Budget  │
  │ Compiler        │
  └─────┬──────────┘
        │
  ┌─────┴──────────┐
  │ Context Memory  │
  │ Lensing         │
  └─────┬──────────┘
        │
  ┌─────┴──────────┐
  │ Semantic Cache  │
  └─────┬──────────┘
        │
  ┌─────┴──────────┐
  │ Hybrid Router   │
  └─────┬──────────┘
        │
  Local / Private / Cloud Providers</pre>
      </div>
      <h4>Monorepo Crates</h4>
      <table class="guide-table">
        <thead><tr><th>Crate</th><th>Purpose</th></tr></thead>
        <tbody>
          <tr><td><code>core/</code></td><td>Axum HTTP server — API entry point and pipeline orchestration</td></tr>
          <tr><td><code>compiler/</code></td><td>Context Budget Compiler — intent detection and context reduction</td></tr>
          <tr><td><code>memory/</code></td><td>Context Memory Lensing — stable block store and delta engine</td></tr>
          <tr><td><code>router/</code></td><td>Hybrid Sovereign Router — provider selection by policy and intent</td></tr>
          <tr><td><code>adapters/</code></td><td>Provider-specific HTTP clients (Ollama, OpenAI, Mistral, OpenRouter)</td></tr>
          <tr><td><code>metrics/</code></td><td>AI Efficiency Score computation and telemetry</td></tr>
          <tr><td><code>cache/</code></td><td>Semantic cache — fingerprint lookup and compiled context store</td></tr>
          <tr><td><code>fingerprint/</code></td><td>Prompt fingerprint graph for deduplication</td></tr>
          <tr><td><code>tokenizer/</code></td><td>BPE-boundary-aware token estimation</td></tr>
        </tbody>
      </table>
      <h4>Frontend &amp; Tooling</h4>
      <table class="guide-table">
        <thead><tr><th>Directory</th><th>Purpose</th></tr></thead>
        <tbody>
          <tr><td><code>dashboard/ui-vue/</code></td><td>AI Flow Visualizer — Vue 3 + Vite dark dashboard</td></tr>
          <tr><td><code>configs/</code></td><td>Provider, routing, policy, and workspace configuration</td></tr>
          <tr><td><code>mcp/</code></td><td>MCP server for VS Code Copilot integration</td></tr>
          <tr><td><code>docs/</code></td><td>Architecture, API reference, implementation notes</td></tr>
          <tr><td><code>deployments/</code></td><td>Docker, Kubernetes, Helm manifests</td></tr>
        </tbody>
      </table>
    </section>

    <!-- Section 3: Pipeline -->
    <section id="pipeline" class="card guide-section">
      <h3><SvgIcon name="git-branch" :size="20" /> Compilation Pipeline</h3>
      <p>Every <code>POST /v1/compile</code> request runs through the full pipeline:</p>
      <div class="guide-pipeline-steps">
        <div class="pipeline-step"><span class="step-num">1</span><span>Fingerprinting — SHA-based prompt deduplication</span></div>
        <div class="pipeline-step"><span class="step-num">2</span><span>Cache lookup — returns cached result if fingerprint matches</span></div>
        <div class="pipeline-step"><span class="step-num">3</span><span>Intent Detection — classifies prompt (debug, review, codegen, summarize, etc.)</span></div>
        <div class="pipeline-step"><span class="step-num">4</span><span>RCT2I Restructuring — reorganizes prompt into Role/Context/Tasks/Instructions/Improvement</span></div>
        <div class="pipeline-step"><span class="step-num">5</span><span>Pass Optimizer — lossless text optimization</span></div>
        <div class="pipeline-step"><span class="step-num">6</span><span>Semantic Reduction — intent-based distillation with tuned keep percentages</span></div>
        <div class="pipeline-step"><span class="step-num">7</span><span>BPE-Boundary Truncation — clean cuts at token boundaries</span></div>
        <div class="pipeline-step"><span class="step-num">8</span><span>Context Memory — delta-first forwarding, reusing stable blocks</span></div>
        <div class="pipeline-step"><span class="step-num">9</span><span>Sovereign Routing — selects the optimal provider</span></div>
        <div class="pipeline-step"><span class="step-num">10</span><span>Metrics Collection — updates efficiency scores and audit trail</span></div>
      </div>

      <h4>The 11 Optimizer Passes</h4>
      <p>The optimizer runs lossless text passes in sequence. Each pass targets a specific noise pattern:</p>
      <table class="guide-table">
        <thead><tr><th>Pass</th><th>Name</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td>1</td><td>Whitespace normalization</td><td>Collapses excessive whitespace, blank lines, indentation</td></tr>
          <tr><td>2</td><td>Numeric separator removal</td><td>Strips formatting separators from numbers</td></tr>
          <tr><td>3</td><td>Verbose-phrase substitution</td><td>Replaces wordy phrases with concise equivalents</td></tr>
          <tr><td>4</td><td>Duplicate-line collapse</td><td>Removes consecutive repeated lines</td></tr>
          <tr><td>5</td><td>Comment stripping</td><td>Removes standalone comments from code blocks</td></tr>
          <tr><td>6</td><td>Compact inline JSON</td><td>Minifies JSON structures inline</td></tr>
          <tr><td>7</td><td>Stopword removal</td><td>Removes low-signal stopwords (intent-dependent)</td></tr>
          <tr><td>8</td><td>URL &amp; file-path compression</td><td>Shortens URLs and file paths to essential segments</td></tr>
          <tr><td>9</td><td>Code boilerplate stripping</td><td>Removes import blocks, package declarations, boilerplate</td></tr>
          <tr><td>10</td><td>Code keyword abbreviation</td><td>Abbreviates common language keywords (codegen only)</td></tr>
          <tr><td>11</td><td>Non-consecutive deduplication</td><td>Commvault-inspired cross-line deduplication</td></tr>
        </tbody>
      </table>
    </section>

    <!-- Section 4: Intents -->
    <section id="intents" class="card guide-section">
      <h3><SvgIcon name="crosshair" :size="20" /> Intents &amp; Routing</h3>
      <p>DISTIRA detects intent from the prompt and routes to the best provider. Slash commands override auto-detection.</p>
      <table class="guide-table">
        <thead><tr><th>Intent</th><th>Keywords</th><th>Default Provider</th></tr></thead>
        <tbody>
          <tr><td><strong>debug</strong></td><td>error, trace, panic, exception, fatal</td><td>ollama-mistral-7b-instruct (on-prem)</td></tr>
          <tr><td><strong>review</strong></td><td>diff, pull request, refactor, review</td><td>ollama-qwen2.5-coder (on-prem)</td></tr>
          <tr><td><strong>codegen</strong></td><td>function, implement, write, typescript, go</td><td>ollama-qwen2.5-coder (on-prem)</td></tr>
          <tr><td><strong>summarize</strong></td><td>summarize, explain, recap, résumé</td><td>openrouter-mistral-small (cloud)</td></tr>
          <tr><td><strong>translate</strong></td><td>translate, traduire, french, german, japanese</td><td>openrouter-mistral-small (cloud)</td></tr>
          <tr><td><strong>ocr</strong></td><td>OCR, image, scan, extract text</td><td>mistral-ocr-2512-cloud (cloud)</td></tr>
          <tr><td><strong>general</strong></td><td>anything else</td><td>openrouter-step-3.5-flash (cloud)</td></tr>
        </tbody>
      </table>
      <h4>Slash Commands</h4>
      <p>Override auto-detection by prefixing your prompt:</p>
      <table class="guide-table">
        <thead><tr><th>Command</th><th>Effect</th></tr></thead>
        <tbody>
          <tr><td><code>/debug</code></td><td>Force debug intent</td></tr>
          <tr><td><code>/review</code></td><td>Force review intent</td></tr>
          <tr><td><code>/codegen</code></td><td>Force codegen intent</td></tr>
          <tr><td><code>/summarize</code></td><td>Force summarize intent</td></tr>
          <tr><td><code>/translate</code></td><td>Force translate intent</td></tr>
          <tr><td><code>/ocr</code></td><td>Force OCR intent</td></tr>
          <tr><td><code>/fast</code></td><td>Prefer fastest provider</td></tr>
          <tr><td><code>/quality</code></td><td>Prefer highest-quality provider</td></tr>
          <tr><td><code>/local</code></td><td>Force local/on-prem only</td></tr>
        </tbody>
      </table>
      <div class="guide-highlight">
        <p><strong>Sovereignty rule:</strong> Requests marked <code>sensitive: true</code> are <em>always</em>
        forced to on-prem (ollama-llama3) regardless of intent or slash command.</p>
      </div>
    </section>

    <!-- Section 5: RCT2I -->
    <section id="RCT2I" class="card guide-section">
      <h3><SvgIcon name="wrench" :size="20" /> RCT2I Prompt Restructuring</h3>
      <p>
        RCT2I (Role / Context / Tasks / Instructions / Improvement) restructures prompts into a
        standardized format that improves LLM comprehension and response quality.
      </p>
      <h4>How it works</h4>
      <ul>
        <li><strong>[R] Role</strong> — Derived from detected intent (e.g., "debug assistant", "code reviewer")</li>
        <li><strong>[C] Context</strong> — Extracted contextual facts from the prompt</li>
        <li><strong>[T] Tasks</strong> — Action items identified from task verbs (65+ verbs in EN/FR)</li>
        <li><strong>[I] Instructions</strong> — Constraints and preferences (e.g., "in TypeScript", "without dependencies")</li>
        <li><strong>[I] Improvement</strong> — Quality hints per intent (e.g., "include root cause" for debug)</li>
      </ul>
      <h4>When RCT2I applies</h4>
      <ul>
        <li>Prompts with 4+ words (short prompts are passed through as-is)</li>
        <li>All intents except <code>ocr</code> and <code>translate</code></li>
        <li>Raw artifact prompts (>50% stack traces/diffs) are preserved without restructuring</li>
      </ul>
    </section>

    <!-- Section 6: API Reference -->
    <section id="api-reference" class="card guide-section">
      <h3><SvgIcon name="database" :size="20" /> REST API Reference</h3>
      <p>DISTIRA exposes an OpenAI-compatible API on <code>localhost:8080</code>.</p>
      <table class="guide-table api-table">
        <thead><tr><th>Method</th><th>Endpoint</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><span class="method-badge post">POST</span></td><td><code>/v1/compile</code></td><td>Compile context through the full pipeline. Returns compiled context, intent, token counts, provider routing.</td></tr>
          <tr><td><span class="method-badge post">POST</span></td><td><code>/v1/chat/completions</code></td><td>OpenAI-compatible chat endpoint. Compiles context, then forwards to the routed provider. Supports streaming.</td></tr>
          <tr><td><span class="method-badge get">GET</span></td><td><code>/v1/metrics</code></td><td>Current metrics snapshot: total requests, tokens saved, efficiency score, per-provider counts.</td></tr>
          <tr><td><span class="method-badge get">GET</span></td><td><code>/v1/metrics/stream</code></td><td>SSE stream — pushes metrics every 2 seconds. Used by the dashboard for live updates.</td></tr>
          <tr><td><span class="method-badge delete">DELETE</span></td><td><code>/v1/metrics/reset</code></td><td>Reset all metrics counters to zero.</td></tr>
          <tr><td><span class="method-badge get">GET</span></td><td><code>/v1/providers</code></td><td>List all configured providers with status and capabilities.</td></tr>
          <tr><td><span class="method-badge get">GET</span></td><td><code>/v1/suggestions</code></td><td>Get optimization suggestions based on recent request patterns.</td></tr>
          <tr><td><span class="method-badge get">GET</span></td><td><code>/healthz</code></td><td>Health check — returns 200 OK when the server is running.</td></tr>
          <tr><td><span class="method-badge get">GET</span></td><td><code>/version</code></td><td>Returns current server version.</td></tr>
        </tbody>
      </table>
      <h4>Example: Compile</h4>
      <div class="guide-code">
        <pre><code>curl -X POST http://localhost:8080/v1/compile \
  -H "Content-Type: application/json" \
  -d '{
    "context": "explain how DISTIRA reduces tokens",
    "sensitive": false
  }'</code></pre>
      </div>
      <h4>Example: Chat Completions</h4>
      <div class="guide-code">
        <pre><code>curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "auto",
    "messages": [
      {"role": "user", "content": "explain RCT2I"}
    ],
    "stream": true
  }'</code></pre>
      </div>
    </section>

    <!-- Section 7: MCP -->
    <section id="mcp" class="card guide-section">
      <h3><SvgIcon name="shield" :size="20" /> MCP Integration</h3>
      <p>
        DISTIRA includes an MCP (Model Context Protocol) server that integrates directly into
        VS Code Copilot Chat. It is configured automatically via <code>.vscode/mcp.json</code>.
      </p>
      <h4>Available MCP Tools</h4>
      <table class="guide-table">
        <thead><tr><th>Tool</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>distira_compile</code></td><td>Compile context through the pipeline — returns intent, token savings, and routing</td></tr>
          <tr><td><code>distira_chat</code></td><td>Full pipeline: compile → cache → route → forward to LLM provider</td></tr>
          <tr><td><code>distira_metrics</code></td><td>Get real-time efficiency metrics and stats</td></tr>
          <tr><td><code>distira_providers</code></td><td>List all configured providers and their status</td></tr>
          <tr><td><code>distira_set_client_context</code></td><td>Set the upstream client model/provider for routing decisions</td></tr>
        </tbody>
      </table>
      <h4>Setup</h4>
      <p>MCP is launched automatically by VS Code when the workspace is opened (via <code>.vscode/mcp.json</code>).
      No manual steps needed. For manual testing:</p>
      <div class="guide-code">
        <pre><code>cd mcp &amp;&amp; node distira-server.mjs</code></pre>
      </div>
    </section>

    <!-- Section 8: Dashboard Views -->
    <section id="dashboard" class="card guide-section">
      <h3><SvgIcon name="chart-bar" :size="20" /> Dashboard Views</h3>
      <p>The dashboard is a Vue 3 SPA connected to DISTIRA via SSE for real-time updates.</p>
      <table class="guide-table">
        <thead><tr><th>View</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><strong>Overview</strong></td><td>KPIs (requests, tokens saved, efficiency score, CO₂ avoided, cost savings), 24-point history chart, request lineage, before/after pipeline example.</td></tr>
          <tr><td><strong>Savings &amp; Impact</strong></td><td>Economic and environmental impact: token cost savings, CO₂ reduction, cloud calls avoided, cumulative gains.</td></tr>
          <tr><td><strong>AI Flow</strong></td><td>Visual pipeline: shows each stage (fingerprint → intent → compiler → memory → router) with real-time data flow.</td></tr>
          <tr><td><strong>Memory</strong></td><td>Context Memory viewer: stable blocks, delta tracking, reuse ratio, memory pool status.</td></tr>
          <tr><td><strong>Insights</strong></td><td>Automated optimization recommendations: high-waste detection, routing improvements, configuration suggestions.</td></tr>
          <tr><td><strong>Benchmarks</strong></td><td>Performance benchmarks: latency percentiles, throughput metrics, provider comparison.</td></tr>
          <tr><td><strong>Runtime Audit</strong></td><td>Full audit trail: every request logged with timestamps, intents, routing decisions, token counts.</td></tr>
          <tr><td><strong>Guide</strong></td><td>This page — integrated documentation and reference.</td></tr>
        </tbody>
      </table>
    </section>

    <!-- Section 9: Configuration -->
    <section id="configuration" class="card guide-section">
      <h3><SvgIcon name="wrench" :size="20" /> Configuration</h3>
      <p>All configuration is in the <code>configs/</code> directory. YAML format, no code changes required.</p>
      <table class="guide-table">
        <thead><tr><th>File</th><th>Purpose</th></tr></thead>
        <tbody>
          <tr><td><code>configs/providers/providers.yaml</code></td><td>Define providers: endpoint, model, type, API key reference. Enable/disable by commenting.</td></tr>
          <tr><td><code>configs/routing/routing.yaml</code></td><td>Map intents to providers. Define fallback chains and priority.</td></tr>
          <tr><td><code>configs/policies/policies.yaml</code></td><td>Sovereignty policies: sensitive data rules, local-only enforcement, cost limits.</td></tr>
          <tr><td><code>configs/workspace/workspace.yaml</code></td><td>Workspace settings: project ID, tenant, default behaviors.</td></tr>
        </tbody>
      </table>
      <h4>Adding a new provider</h4>
      <ol>
        <li>Edit <code>configs/providers/providers.yaml</code></li>
        <li>Add the provider block with key, endpoint, model, and type</li>
        <li>Set the API key in <code>.env</code> if needed</li>
        <li>Map the provider to intents in <code>configs/routing/routing.yaml</code></li>
        <li>Restart the backend — no rebuild required</li>
      </ol>
    </section>

    <!-- Section 10: Providers -->
    <section id="providers" class="card guide-section">
      <h3><SvgIcon name="cloud" :size="20" /> Providers &amp; Compatibility</h3>
      <p>DISTIRA connects to any OpenAI-compatible endpoint. No code changes required.</p>
      <table class="guide-table">
        <thead><tr><th>Runtime</th><th>Default Port</th><th>Notes</th></tr></thead>
        <tbody>
          <tr><td>Ollama</td><td><code>:11434/v1</code></td><td><code>ollama pull &lt;model&gt;</code></td></tr>
          <tr><td>vLLM</td><td><code>:8000/v1</code></td><td>OpenAI-compatible server mode</td></tr>
          <tr><td>LM Studio</td><td><code>:1234/v1</code></td><td>Enable local server in UI</td></tr>
          <tr><td>OpenWebUI</td><td><code>:3000/api</code></td><td>Proxies Ollama or any backend</td></tr>
          <tr><td>OpenAI</td><td><code>api.openai.com/v1</code></td><td>Requires OPENAI_API_KEY</td></tr>
          <tr><td>Anthropic</td><td><code>api.anthropic.com/v1</code></td><td>Requires ANTHROPIC_API_KEY</td></tr>
          <tr><td>Google Gemini</td><td><code>generativelanguage.googleapis.com</code></td><td>Requires GOOGLE_API_KEY</td></tr>
          <tr><td>Mistral AI</td><td><code>api.mistral.ai/v1</code></td><td>Requires MISTRAL_API_KEY</td></tr>
          <tr><td>OpenRouter</td><td><code>openrouter.ai/api/v1</code></td><td>Requires OPENROUTER_API_KEY</td></tr>
          <tr><td>ZhipuAI</td><td><code>open.bigmodel.cn/api/paas/v4</code></td><td>Requires ZHIPU_API_KEY</td></tr>
          <tr><td>DashScope</td><td><code>dashscope.aliyuncs.com</code></td><td>Requires DASHSCOPE_API_KEY</td></tr>
        </tbody>
      </table>
    </section>

    <!-- Section 11: Troubleshooting -->
    <section id="troubleshooting" class="card guide-section">
      <h3><SvgIcon name="shield" :size="20" /> Troubleshooting</h3>
      <table class="guide-table">
        <thead><tr><th>Issue</th><th>Solution</th></tr></thead>
        <tbody>
          <tr><td>Dashboard shows "Offline"</td><td>Ensure the Rust backend is running on port 8080. Check <code>cargo run -p core</code> or <code>.\scripts\start-win.ps1</code>.</td></tr>
          <tr><td>0% token reduction</td><td>Short prompts (&lt;40 tokens) may have minimal reduction. This is expected — DISTIRA avoids over-compressing small inputs.</td></tr>
          <tr><td>Provider unreachable</td><td>Check provider endpoint in <code>providers.yaml</code>. For Ollama: ensure <code>ollama serve</code> is running.</td></tr>
          <tr><td>RCT2I not applied</td><td>RCT2I requires 4+ word prompts and excludes OCR/translate intents. Pure artifact prompts (&gt;50% stack traces) are preserved.</td></tr>
          <tr><td>Sensitive data routed to cloud</td><td>Set <code>sensitive: true</code> in the request. DISTIRA will force on-prem routing.</td></tr>
          <tr><td>Cache not hitting</td><td>Cache uses fingerprint matching — identical prompts hit cache. Minor wording changes generate different fingerprints.</td></tr>
          <tr><td>MCP not connecting</td><td>Ensure <code>.vscode/mcp.json</code> exists and the backend is running. Restart VS Code if needed.</td></tr>
          <tr><td>Build fails (access denied)</td><td>Stop the running server process before rebuilding with <code>cargo build --release</code>.</td></tr>
        </tbody>
      </table>
    </section>

    <footer class="guide-footer">
      <p class="muted">DISTIRA v{{ appVersion }} — The AI Context Compiler. Documentation auto-generated from the live system.</p>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SvgIcon from '../components/SvgIcon.vue'
import { useMetricsStore } from '../store/metrics'

const metrics = useMetricsStore()
const appVersion = computed(() => metrics.appVersion)

function scrollTo(id: string) {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth' })
}
</script>

<style scoped>
.guide-view {

  margin: 0 auto;
}

.guide-toc {
  margin-bottom: 1.5rem;
}
.guide-toc h3 {
  margin-bottom: 0.75rem;
  color: var(--primary);
}
.guide-toc ol {
  columns: 2;
  column-gap: 2rem;
  padding-left: 1.5rem;
  margin: 0;
}
.guide-toc li {
  margin-bottom: 0.4rem;
}
.guide-toc a {
  color: var(--accent);
  text-decoration: none;
  transition: color 0.2s;
}
.guide-toc a:hover {
  color: var(--primary);
  text-decoration: underline;
}

.guide-section {
  margin-bottom: 1.5rem;
}
.guide-section h3 {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--primary);
  margin-bottom: 1rem;
  font-size: 1.25rem;
}
.guide-section h4 {
  color: var(--secondary);
  margin: 1.25rem 0 0.75rem;
  font-size: 1rem;
}
.guide-section p {
  line-height: 1.7;
  margin-bottom: 0.75rem;
}
.guide-section ul,
.guide-section ol {
  padding-left: 1.5rem;
  line-height: 1.8;
}
.guide-section li {
  margin-bottom: 0.3rem;
}

.guide-table {
  width: 100%;
  border-collapse: collapse;
  margin: 0.75rem 0;
  font-size: 0.9rem;
}
.guide-table th {
  text-align: left;
  padding: 0.6rem 0.75rem;
  border-bottom: 2px solid var(--primary);
  color: var(--primary);
  font-weight: 600;
}
.guide-table td {
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  vertical-align: top;
}
.guide-table tr:hover td {
  background: rgba(255, 255, 255, 0.02);
}
.guide-table code {
  background: rgba(255, 255, 255, 0.06);
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  font-size: 0.85em;
}

.guide-highlight {
  background: rgba(var(--primary-rgb, 99, 102, 241), 0.08);
  border-left: 3px solid var(--primary);
  padding: 1rem 1.25rem;
  border-radius: 0 10px 10px 0;
  margin: 1rem 0;
}
.guide-highlight p {
  margin-bottom: 0.4rem;
}
.guide-highlight p:last-child {
  margin-bottom: 0;
}

.guide-diagram {
  background: rgba(0, 0, 0, 0.3);
  border-radius: 12px;
  padding: 1rem 1.5rem;
  overflow-x: auto;
  margin: 1rem 0;
}
.guide-diagram pre {
  margin: 0;
  color: var(--accent);
  font-size: 0.85rem;
  line-height: 1.5;
}

.guide-pipeline-steps {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin: 1rem 0;
}
.pipeline-step {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.6rem 1rem;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 10px;
  border-left: 3px solid var(--primary);
  transition: background 0.2s;
}
.pipeline-step:hover {
  background: rgba(255, 255, 255, 0.06);
}
.step-num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--primary);
  color: #fff;
  font-weight: 700;
  font-size: 0.8rem;
  flex-shrink: 0;
}

.guide-code {
  background: rgba(0, 0, 0, 0.3);
  border-radius: 12px;
  padding: 1rem 1.25rem;
  overflow-x: auto;
  margin: 0.75rem 0;
}
.guide-code pre {
  margin: 0;
}
.guide-code code {
  color: var(--accent);
  font-size: 0.85rem;
  line-height: 1.6;
}

.method-badge {
  display: inline-block;
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}
.method-badge.get { background: #22c55e22; color: #22c55e; }
.method-badge.post { background: #3b82f622; color: #3b82f6; }
.method-badge.delete { background: #ef444422; color: #ef4444; }

.api-table td:first-child { white-space: nowrap; }
.api-table td:nth-child(2) { white-space: nowrap; }

.guide-footer {
  text-align: center;
  padding: 2rem 0 1rem;
}

@media (max-width: 768px) {
  .guide-toc ol {
    columns: 1;
  }
  .guide-table {
    font-size: 0.8rem;
  }
  .guide-table th,
  .guide-table td {
    padding: 0.4rem 0.5rem;
  }
}
</style>
