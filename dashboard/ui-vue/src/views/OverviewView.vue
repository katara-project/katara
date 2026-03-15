<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Overview</h2>
        <p class="muted">A visual control plane for AI traffic and context optimization.</p>
      </div>
      <button class="reset-btn" :disabled="resetting" @click="resetMetrics">
        {{ resetting ? 'Resetting…' : 'Reset metrics' }}
      </button>
    </header>
    <div v-if="metrics.alerts && metrics.alerts.length" class="alert-banner">
      <div
        v-for="(alert, idx) in metrics.alerts"
        :key="idx"
        class="alert-item"
        :class="alert.type"
      >
        <span class="alert-icon">&#9888;</span>
        <span>{{ alert.message }}</span>
      </div>
    </div>
    <div class="metrics-grid">
      <MetricCard label="Raw Tokens" :value="metrics.rawTokens.toLocaleString()" hint="Estimated before compilation" accent="warn">
        <SparklineChart :data="rawHistory" color="#ffa940" :height="40" />
      </MetricCard>
      <MetricCard label="Compiled Tokens" :value="metrics.compiledTokens.toLocaleString()" hint="Estimated after compiler" accent="primary">
        <SparklineChart :data="compiledHistory" color="var(--primary)" :height="40" />
      </MetricCard>
      <MetricCard label="Memory Reused" :value="metrics.memoryReusedTokens.toLocaleString()" hint="Tokens not re-sent" accent="secondary">
        <SparklineChart :data="memoryHistory" color="var(--secondary)" :height="40" />
      </MetricCard>
      <MetricCard label="Local Routing" :value="metrics.localRatio + '%'" hint="Requests kept local" accent="accent">
        <SparklineChart :data="localHistory" color="var(--accent)" :height="40" />
      </MetricCard>
      <MetricCard label="RCT2I Structured" :value="RCT2ILabel" hint="Prompts restructured by RCT2I" accent="secondary" />
    </div>

    <!-- V10.7 — Session Savings bar (estimated $ saved from tokens avoided) -->
    <div class="budget-bar-wrap card">
      <div class="budget-bar-header">
        <span class="budget-bar-title">Estimated Session Savings</span>
        <span class="budget-bar-amounts">
          {{ savingsData.tokensSaved.toLocaleString() }} tokens saved
          <span class="budget-bar-sep">·</span>
          {{ formatCost(savingsData.costSaved) }} USD
        </span>
        <span class="budget-bar-pct" :class="savingsPctClass">{{ savingsPct }}%</span>
      </div>
      <div class="budget-track">
        <div class="budget-fill" :class="savingsPctClass" :style="{ width: Math.min(savingsPct, 100) + '%' }"></div>
      </div>
    </div>
    <div class="two-col">
      <EfficiencyGauge :score="metrics.efficiencyScore" />
      <FlowVisualizer />
    </div>

    <div class="charts-row">
      <section class="card chart-section">
        <div class="chart-heading">
          <h3>Token Trends ({{ trendWindowLabel }})</h3>
          <span class="chart-timestamp">Last update: {{ trendLastUpdate }}</span>
          <div class="window-switch" role="group" aria-label="Token trends window">
            <button
              v-for="w in trendWindowOptions"
              :key="w"
              type="button"
              class="window-btn"
              :class="{ active: trendWindow === w }"
              @click="trendWindow = w"
            >
              {{ w }}
            </button>
          </div>
        </div>
        <TvChart :series="trendSeries" :labels="trendLabels" :height="220" :hideXLabels="true" />
      </section>

      <section class="card chart-section">
        <div class="chart-heading">
          <h3>Token Trends (live)</h3>
        </div>
        <TvChart :series="liveSeries" :labels="liveLabels" :height="220" :hideXLabels="true" />
      </section>
    </div>

    <section class="card scope-clarity-section">
      <div class="scope-clarity-header">
        <div>
          <h3>Model Scope Clarity</h3>
          <p class="muted">This dashboard tracks the model routed by Distira after compilation and policy routing. It does not assume that the end-user assistant or client is the same model.</p>
        </div>
      </div>
      <div class="scope-clarity-grid">
        <div class="scope-card assistant-scope">
          <span class="tile-label">Assistant / Client Model</span>
          <strong>{{ leadUpstream.model }}</strong>
          <span class="tile-subtitle">{{ leadUpstream.provider }} · {{ leadUpstream.clientApp }}</span>
        </div>
        <div class="scope-card distira-scope">
          <span class="tile-label">DISTIRA Routed Model</span>
          <strong>{{ leadModel.model }}</strong>
          <span class="tile-subtitle">{{ leadModel.provider }} · {{ leadModel.routeLabel }}</span>
        </div>
      </div>
      <p class="scope-note">
        Distira metrics, sovereignty ratios, and token savings below always refer to the routed model selected by Distira, not the assistant brand or UI model shown upstream.
      </p>
      <div v-if="upstreamVisibilityWarning.show" class="upstream-warning-banner">
        <strong>{{ upstreamVisibilityWarning.title }}</strong>
        <p>{{ upstreamVisibilityWarning.message }}</p>
      </div>
    </section>

    <!-- V10.16 — Before/After Pipeline Example -->
    <section class="card pipeline-example-section">
      <h3>AI Flow Pipeline — Before &amp; After</h3>
      <p class="muted">A real example of how DISTIRA compiles a raw prompt before forwarding it to the LLM.</p>
      <div class="pipeline-ba-grid">
        <div class="pipeline-ba-card before-card">
          <span class="ba-badge before">Before</span>
          <pre class="ba-code">User: Please can you explain to me in detail
what is the error that I am seeing in
src/main.rs at line 42? The error says
"mismatched types expected i32 found &amp;str".
I would really appreciate a detailed
explanation of why this happens and how
to fix it. Thank you very much!</pre>
          <span class="ba-tokens">~68 tokens · intent unknown</span>
        </div>
        <div class="pipeline-ba-arrow">
          <span class="arrow-label">DISTIRA</span>
          <span class="arrow-icon">→</span>
          <span class="arrow-steps">11 passes · RCT2I · dedup</span>
        </div>
        <div class="pipeline-ba-card after-card">
          <span class="ba-badge after">After</span>
          <pre class="ba-code">[k:debug]|explain error src/main.rs:42
mismatched types expected i32 found &amp;str
why + fix</pre>
          <span class="ba-tokens">~21 tokens · debug · 69% saved</span>
        </div>
      </div>
    </section>

    <section class="card llm-summary-section">
      <div class="llm-summary-header">
        <div>
          <h3>Active LLM Routing</h3>
          <p class="muted">Which model Distira is sending traffic to, how much token reduction it gets, and whether routing stays sovereign.</p>
        </div>
      </div>
      <div class="llm-summary-grid">
        <div class="llm-summary-tile emphasis">
          <span class="tile-label">Most Used LLM</span>
          <strong>{{ leadModel.model }}</strong>
          <span class="tile-subtitle">{{ leadModel.provider }}</span>
        </div>
        <div class="llm-summary-tile">
          <span class="tile-label">Route Class</span>
          <strong>{{ leadModel.routeLabel }}</strong>
          <span class="tile-subtitle">{{ leadModel.requests }} requests</span>
        </div>
        <div class="llm-summary-tile">
          <span class="tile-label">Token Reduction</span>
          <strong>{{ leadModel.reduction }}%</strong>
          <span class="tile-subtitle">{{ leadModel.savedTokens.toLocaleString() }} tokens saved</span>
        </div>
        <div class="llm-summary-tile">
          <span class="tile-label">Sovereign Usage</span>
          <strong>{{ leadModel.sovereignLabel }}</strong>
          <span class="tile-subtitle">{{ leadModel.sovereignRequests }} sovereign / {{ leadModel.nonSovereignRequests }} non-sovereign</span>
        </div>
      </div>
    </section>


    <section class="card request-lineage-section">
      <div class="request-lineage-header">
        <div>
          <h3>Last Request</h3>
          <p class="muted">Live lineage for the most recent request seen by Distira, including upstream client identity, routed target, cache behavior, and sensitivity override.</p>
        </div>
        <span class="request-time">{{ lastRequestCard.seenAt }}</span>
      </div>
      <div class="request-lineage-grid">
        <div class="request-cell">
          <span class="tile-label">Client App</span>
          <strong>{{ lastRequestCard.clientApp }}</strong>
          <span class="tile-subtitle">{{ lastRequestCard.upstreamProvider }}</span>
        </div>
        <div class="request-cell">
          <span class="tile-label">Upstream Model</span>
          <strong>{{ lastRequestCard.upstreamModel }}</strong>
          <span class="tile-subtitle">What the user-facing client reported</span>
        </div>
        <div class="request-cell">
          <span class="tile-label">Routed Target</span>
          <strong>{{ lastRequestCard.routedModel }}</strong>
          <span class="tile-subtitle">{{ lastRequestCard.routedProvider }}</span>
        </div>
        <div class="request-cell">
          <span class="tile-label">Execution Flags</span>
          <div class="request-flags">
            <span class="route-pill" :class="lastRequestCard.routeClass">{{ lastRequestCard.routeLabel }}</span>
            <span class="status-pill" :class="lastRequestCard.cacheClass">{{ lastRequestCard.cacheLabel }}</span>
            <span class="status-pill" :class="lastRequestCard.sensitiveClass">{{ lastRequestCard.sensitiveLabel }}</span>
          </div>
          <span class="tile-subtitle">Intent: {{ lastRequestCard.intent }}</span>
        </div>
      </div>
    </section>

    <section class="card upstream-models-section">
      <div class="section-heading">
        <div>
          <h3>Upstream Client Models</h3>
          <p class="muted">What the calling client reports upstream, such as GPT-5.4 in VS Code Copilot. This is separate from Distira's routed target.</p>
        </div>
      </div>
      <div class="model-table-wrap">
        <table class="model-table">
          <thead>
            <tr>
              <th>Upstream Model</th>
              <th>Provider</th>
              <th>Client App</th>
              <th>Requests</th>
              <th>Last Seen</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="entry in upstreamTableRows" :key="entry.key">
              <td>{{ entry.model }}</td>
              <td>{{ entry.provider }}</td>
              <td>{{ entry.clientApp }}</td>
              <td>{{ entry.requests }}</td>
              <td>{{ entry.lastSeen }}</td>
            </tr>
            <tr v-if="!upstreamTableRows.length">
              <td colspan="5" class="muted">No upstream client model reported yet. Distira can only show GPT-5.4 here if VS Code/Copilot exposes it or if runtime client context is updated live.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section class="card model-efficiency-section">
      <h3>Live AI Efficiency by Routed Model</h3>
      <p class="muted">Per-routed-model efficiency score with Sovereign Routing visibility. This table shows what Distira actually sent downstream, not the model selected in the upstream client UI.</p>
      <div class="model-table-wrap">
        <table class="model-table">
          <thead>
            <tr>
              <th>Model</th>
              <th>Provider</th>
              <th>Route</th>
              <th>Quality</th>
              <th>Requests</th>
              <th>Saved Tokens</th>
              <th>Latency</th>
              <th>Efficiency</th>
              <th>Sovereign</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="entry in modelRows" :key="entry.key">
              <td>{{ entry.model }}</td>
              <td>{{ entry.provider }}</td>
              <td>
                <span class="route-pill" :class="entry.routeClass">{{ entry.routeLabel }}</span>
              </td>
              <td>
                <span class="quality-pill" :class="entry.qualityTier">{{ entry.qualityTier }}</span>
              </td>
              <td>{{ entry.requests }}</td>
              <td>{{ entry.savedTokens.toLocaleString() }}</td>
              <td>
                <span v-if="entry.avgLatencyMs > 0" class="latency-badge">{{ entry.avgLatencyMs }} ms</span>
                <span v-else class="muted">—</span>
              </td>
              <td>
                <strong>{{ entry.efficiency }}%</strong>
              </td>
              <td>
                <span class="sovereign-pill" :class="entry.sovereignClass">{{ entry.sovereignLabel }}</span>
              </td>
            </tr>
            <tr v-if="!modelRows.length">
              <td colspan="9" class="muted">No model data yet. Send a few requests to populate live stats.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>


  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useMetricsStore } from '../store/metrics'
import { classifyRoute, friendlyProvider, qualityTier } from '../utils/providers'
import MetricCard from '../components/MetricCard.vue'
import EfficiencyGauge from '../components/EfficiencyGauge.vue'
import FlowVisualizer from '../components/FlowVisualizer.vue'
import SparklineChart from '../components/SparklineChart.vue'
import TvChart from '../components/TvChart.vue'

const metrics = useMetricsStore()

const resetting = ref(false)
async function resetMetrics() {
  if (resetting.value) return
  resetting.value = true
  try {
    await fetch('/v1/metrics/reset', { method: 'DELETE' })
  } finally {
    resetting.value = false
  }
}

const configuredAssistantModelLabel = import.meta.env.VITE_ASSISTANT_MODEL_LABEL || 'External assistant or client model'

// ── V10.7 — Savings bar constants ────────────
const AVG_COST_PER_1K_TOKENS = 0.006
function formatCost(v: number): string {
  if (v === 0) return '$0.00'
  if (v >= 1) return '$' + v.toFixed(2)
  const cents = v * 100
  if (cents >= 1) return cents.toFixed(1) + '¢'
  if (cents >= 0.01) return cents.toFixed(2) + '¢'
  return '< 0.01¢'
}

const savingsData = computed(() => {
  const tokensSaved = Math.max(0, metrics.rawTokens - metrics.compiledTokens) + metrics.cacheSavedTokens
  const costSaved = (tokensSaved / 1000) * AVG_COST_PER_1K_TOKENS
  return { tokensSaved, costSaved }
})

// Sparklines: direct SSE history arrays (reactive)
const rawHistory = computed(() => metrics.historyRaw.length ? metrics.historyRaw : [0])
const compiledHistory = computed(() => metrics.historyCompiled.length ? metrics.historyCompiled : [0])
const memoryHistory = computed(() => metrics.historyReused.length ? metrics.historyReused : [0])
const localHistory = computed(() => {
  const total = metrics.routesLocal + metrics.routesCloud + metrics.routesMidtier
  if (!total || !metrics.historyRaw.length) return [0]
  // Derive a local-ratio sparkline from history length (same cardinality)
  return metrics.historyRaw.map((_: number, i: number) =>
    Math.round((metrics.routesLocal / Math.max(1, total)) * 100)
  )
})

const RCT2ILabel = computed(() => {
  const count = metrics.rct2iAppliedCount
  if (!metrics.totalRequests) return '0'
  const pct = Math.round((count / metrics.totalRequests) * 100)
  return `${count} (${pct}%)`
})

// V10.7 — Session savings bar (estimated $ saved from tokens avoided)
const savingsPct = computed(() => {
  const target = 10000
  const tokensSaved = Math.max(0, metrics.rawTokens - metrics.compiledTokens) + metrics.cacheSavedTokens
  return Math.min(100, Math.round((tokensSaved / target) * 100))
})
const savingsPctClass = computed(() => {
  const pct = savingsPct.value
  if (pct >= 80) return 'budget-ok'
  if (pct >= 40) return 'budget-warning'
  return 'budget-exhausted'
})

// TvChart: prefer true hourly buckets when available, fallback to legacy history.
const trendWindowOptions = ['1h', '6h', '24h'] as const
const trendWindow = ref<(typeof trendWindowOptions)[number]>('24h')

const trendWindowLabel = computed(() => trendWindow.value)

const trendLastUpdate = computed(() => {
  if (!metrics.lastTs) return 'pending metrics...'
  return new Date(metrics.lastTs * 1000).toLocaleString()
})

function sliceByWindow<T>(arr: T[]) {
  const points = trendWindow.value === '1h' ? 2 : trendWindow.value === '6h' ? 6 : 24
  if (!arr.length) return arr
  return arr.slice(-Math.min(points, arr.length))
}

const trendLabels = computed(() => {
  const base = metrics.historyHourEpochs.length
    ? metrics.historyHourEpochs
    : Array.from({ length: metrics.historyRaw.length }, (_, i) => i)
  const points = sliceByWindow(base)
  const len = points.length
  if (!len) return ['—']

  if (metrics.historyHourEpochs.length) {
    return points.map((epoch: number) =>
      new Date(epoch * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    )
  }

  return Array.from({ length: len }, (_, i) => `H-${len - 1 - i}`)
})
const trendSeries = computed(() => [
  {
    name: 'Raw',
    data: metrics.historyHourRaw.length
      ? sliceByWindow([...metrics.historyHourRaw])
      : (metrics.historyRaw.length ? sliceByWindow([...metrics.historyRaw]) : [0]),
    color: '#ffa940',
  },
  {
    name: 'Compiled',
    data: metrics.historyHourCompiled.length
      ? sliceByWindow([...metrics.historyHourCompiled])
      : (metrics.historyCompiled.length ? sliceByWindow([...metrics.historyCompiled]) : [0]),
    color: 'var(--primary)',
  },
  {
    name: 'Reused',
    data: metrics.historyHourReused.length
      ? sliceByWindow([...metrics.historyHourReused])
      : (metrics.historyReused.length ? sliceByWindow([...metrics.historyReused]) : [0]),
    color: 'var(--secondary)',
  },
])

// Live per-request chart (moved from BenchmarksView)
const liveLabels = computed(() =>
  metrics.historyRaw.length
    ? metrics.historyRaw.map((_: number, i: number) => `#${i + 1}`)
    : ['—']
)
const liveSeries = computed(() => {
  const raw = metrics.historyRaw.length ? [...metrics.historyRaw] : [0]
  const compiled = metrics.historyCompiled.length ? [...metrics.historyCompiled] : [0]
  const saved = raw.map((r: number, i: number) => r - (compiled[i] ?? 0))
  return [
    { name: 'Raw tokens', data: raw, color: '#ffa940' },
    { name: 'Compiled', data: compiled, color: 'var(--primary)' },
    { name: 'Saved', data: saved, color: 'var(--accent)' },
  ]
})

const TEST_PATTERNS = /^(t|test|unknown-model)$/i

const upstreamRows = computed(() => {
  return Object.entries(metrics.upstreamStats)
    .map(([key, stat]) => ({
      key,
      model: stat.upstream_model,
      provider: stat.upstream_provider,
      clientApp: stat.client_app,
      requests: stat.requests,
      lastSeenTs: stat.last_seen_ts,
    }))
    .filter((r) => !TEST_PATTERNS.test(r.model) && !TEST_PATTERNS.test(r.provider))
    .sort((a, b) => b.requests - a.requests || b.lastSeenTs - a.lastSeenTs)
})

const upstreamTableRows = computed(() => {
  return upstreamRows.value.map((entry) => ({
    ...entry,
    lastSeen: entry.lastSeenTs ? new Date(entry.lastSeenTs * 1000).toLocaleTimeString() : 'n/a',
  }))
})

// classifyRoute and friendlyProvider are imported from ../utils/providers

const modelRows = computed(() => {
  return Object.entries(metrics.modelStats)
    .map(([key, stat]) => {
      const savedTokens = Math.max(0, (stat.raw_tokens ?? 0) - (stat.compiled_tokens ?? 0))
      const sovereignRatio = Math.round(stat.sovereign_ratio ?? 0)
      const route = classifyRoute(stat.provider)
      return {
        key,
        model: stat.model,
        provider: friendlyProvider(stat.provider),
        providerRaw: stat.provider,
        requests: stat.requests,
        savedTokens,
        efficiency: Math.round(stat.efficiency_score ?? 0),
        reduction: stat.raw_tokens > 0 ? Math.round((savedTokens / stat.raw_tokens) * 100) : 0,
        sovereignRequests: stat.sovereign_requests ?? 0,
        nonSovereignRequests: stat.non_sovereign_requests ?? 0,
        sovereignLabel: sovereignRatio >= 100 ? 'Sovereign' : `${sovereignRatio}% sovereign`,
        sovereignClass: sovereignRatio >= 100 ? 'sovereign' : 'mixed',
        routeLabel: route.routeLabel,
        routeClass: route.routeClass,
        qualityTier: qualityTier(stat.provider),
        avgLatencyMs: Math.round(stat.avg_latency_ms ?? 0),
      }
    })
    .sort((a, b) => b.requests - a.requests)
})

const leadModel = computed(() => {
  const first = modelRows.value[0]
  if (first) return first

  return {
    key: 'none',
    model: 'No traffic yet',
    provider: 'Pending first routed request',
    requests: 0,
    savedTokens: 0,
    efficiency: 0,
    reduction: 0,
    sovereignRequests: 0,
    nonSovereignRequests: 0,
    sovereignLabel: 'No data',
    sovereignClass: 'mixed',
    routeLabel: 'No route',
    routeClass: 'cloud',
  }
})

const leadUpstream = computed(() => {
  const first = upstreamRows.value[0]
  if (first) return first

  return {
    key: 'upstream-none',
    model: metrics.lastRequest?.upstream_model || configuredAssistantModelLabel,
    provider: metrics.lastRequest?.upstream_provider || 'Not supplied by client',
    clientApp: metrics.lastRequest?.client_app || 'Unknown client app',
    requests: 0,
    lastSeenTs: 0,
  }
})

const upstreamVisibilityWarning = computed(() => {
  const lastRequest = metrics.lastRequest
  const missingUpstreamModel = !upstreamRows.value.length && !lastRequest?.upstream_model

  if (!missingUpstreamModel) {
    return {
      show: false,
      title: '',
      message: '',
    }
  }

  return {
    show: true,
    title: 'Upstream model not reported by client',
    message:
      'Distira can only show the real Copilot model, such as GPT-5.4, when VS Code or the MCP client exposes it in request metadata or runtime client context. The routed-model tables below remain valid, but the upstream client model is currently unknown.',
  }
})

const lastRequestCard = computed(() => {
  const lastRequest = metrics.lastRequest
  if (!lastRequest) {
    return {
      clientApp: 'No request yet',
      upstreamProvider: 'Pending upstream metadata',
      upstreamModel: configuredAssistantModelLabel,
      routedProvider: 'No routed provider yet',
      routedModel: 'No routed model yet',
      routeClass: 'cloud',
      routeLabel: 'No route',
      cacheClass: 'neutral',
      cacheLabel: 'No cache data',
      sensitiveClass: 'neutral',
      sensitiveLabel: 'No sensitivity data',
      intent: 'n/a',
      seenAt: 'Pending first request',
    }
  }

  const route = classifyRoute(lastRequest.routed_provider)

  return {
    clientApp: lastRequest.client_app || 'Unknown client app',
    upstreamProvider: lastRequest.upstream_provider || 'Not supplied by client',
    upstreamModel: lastRequest.upstream_model || 'Not reported by client',
    routedProvider: lastRequest.routed_provider,
    routedModel: lastRequest.routed_model,
    routeClass: route.routeClass,
    routeLabel: route.routeLabel,
    cacheClass: lastRequest.cache_hit ? 'hit' : 'miss',
    cacheLabel: lastRequest.cache_hit ? 'Cache hit' : 'Cache miss',
    sensitiveClass: lastRequest.sensitive ? 'warn' : 'neutral',
    sensitiveLabel: lastRequest.sensitive ? 'Sensitive override' : 'Standard routing',
    intent: lastRequest.intent,
    seenAt: new Date(lastRequest.ts * 1000).toLocaleTimeString(),
  }
})
</script>

<style scoped>
.reset-btn {
  align-self: flex-start;
  padding: 6px 16px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(255, 80, 80, 0.12);
  color: #ff6b6b;
  font-size: 0.85rem;
  cursor: pointer;
  transition: background 0.2s, opacity 0.2s;
}
.reset-btn:hover:not(:disabled) {
  background: rgba(255, 80, 80, 0.22);
}
.reset-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.alert-banner {
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.alert-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 0.88rem;
  border: 1px solid;
}
.alert-item.budget_exhausted {
  background: rgba(255, 80, 80, 0.10);
  border-color: rgba(255, 80, 80, 0.30);
  color: #ff6b6b;
}
.alert-item.budget_warning {
  background: rgba(255, 169, 64, 0.10);
  border-color: rgba(255, 169, 64, 0.30);
  color: #ffa940;
}
.alert-icon {
  font-size: 1rem;
  flex-shrink: 0;
}

.charts-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
  margin-top: 20px;
}

.chart-section {
  margin-top: 0;
  min-width: 0;
  overflow: hidden;
}

.chart-heading {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.chart-section h3 {
  margin: 0;
  font-size: 1rem;
}

.window-switch {
  display: inline-flex;
  gap: 6px;
  padding: 4px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.window-btn {
  border: 0;
  background: transparent;
  color: var(--muted);
  padding: 4px 8px;
  border-radius: 8px;
  font-size: 0.75rem;
  cursor: pointer;
}

.window-btn.active {
  background: rgba(57, 211, 255, 0.16);
  color: var(--primary);
}

.chart-timestamp {
  margin-left: auto;
  font-size: 0.75rem;
  color: var(--muted);
  white-space: nowrap;
}

.scope-clarity-section {
  margin-top: 20px;
}

.scope-clarity-header h3 {
  margin: 0 0 6px;
  font-size: 1rem;
}

.scope-clarity-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-top: 14px;
}

.scope-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 16px;
  border-radius: 18px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.assistant-scope {
  background: linear-gradient(135deg, rgba(255, 173, 64, 0.16), rgba(255, 104, 104, 0.1));
}

.distira-scope {
  background: linear-gradient(135deg, rgba(0, 214, 143, 0.16), rgba(40, 120, 255, 0.12));
}

.scope-note {
  margin: 14px 0 0;
  color: var(--muted);
  font-size: 0.88rem;
  line-height: 1.5;
}

.upstream-warning-banner {
  margin-top: 14px;
  padding: 14px 16px;
  border-radius: 16px;
  border: 1px solid rgba(255, 169, 64, 0.35);
  background: linear-gradient(135deg, rgba(255, 169, 64, 0.18), rgba(255, 104, 104, 0.12));
}

.upstream-warning-banner strong {
  display: block;
  color: #ffd28b;
  font-size: 0.92rem;
}

.upstream-warning-banner p {
  margin: 6px 0 0;
  color: #ffe9c7;
  font-size: 0.86rem;
  line-height: 1.5;
}

.llm-summary-section {
  margin-top: 20px;
}

.upstream-models-section {
  margin-top: 20px;
}

.section-heading h3 {
  margin: 0 0 6px;
  font-size: 1rem;
}

.llm-summary-header h3 {
  margin: 0 0 6px;
  font-size: 1rem;
}

.llm-summary-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-top: 14px;
}

.llm-summary-tile {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.llm-summary-tile.emphasis {
  background: linear-gradient(135deg, rgba(0, 214, 143, 0.16), rgba(40, 120, 255, 0.12));
}

.request-lineage-section {
  margin-top: 20px;
}

.request-lineage-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.request-lineage-header h3 {
  margin: 0 0 6px;
  font-size: 1rem;
}

.request-time {
  color: var(--muted);
  font-size: 0.8rem;
  white-space: nowrap;
}

.request-lineage-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-top: 14px;
}

.request-cell {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.request-flags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tile-label {
  color: var(--muted);
  font-size: 0.76rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.llm-summary-tile strong {
  font-size: 1.05rem;
}

.tile-subtitle {
  color: var(--muted);
  font-size: 0.82rem;
}

.model-efficiency-section {
  margin-top: 20px;
}

.model-efficiency-section h3 {
  margin: 0 0 6px;
  font-size: 1rem;
}

.model-table-wrap {
  margin-top: 12px;
  overflow-x: auto;
}

.model-table {
  width: 100%;
  border-collapse: collapse;
  min-width: 620px;
}

.model-table th,
.model-table td {
  text-align: left;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  font-size: 0.88rem;
}

.model-table th {
  color: var(--muted);
  font-weight: 600;
}

.sovereign-pill {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.sovereign-pill.sovereign {
  background: rgba(44, 255, 179, 0.15);
  color: var(--accent);
}

.sovereign-pill.mixed {
  background: rgba(255, 169, 64, 0.15);
  color: #ffa940;
}

.route-pill {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.route-pill.local {
  background: rgba(44, 255, 179, 0.15);
  color: var(--accent);
}

.route-pill.cloud {
  background: rgba(255, 96, 96, 0.15);
  color: #ff8b8b;
}

.route-pill.midtier {
  background: rgba(96, 156, 255, 0.15);
  color: #8ab6ff;
}

.quality-pill {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.3px;
  text-transform: uppercase;
}

.quality-pill.high {
  background: rgba(44, 255, 179, 0.12);
  color: var(--accent);
}

.quality-pill.standard {
  background: rgba(255, 196, 0, 0.12);
  color: #ffd84d;
}

.quality-pill.low {
  background: rgba(150, 150, 150, 0.12);
  color: #999;
}

.latency-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 0.7rem;
  font-weight: 600;
  background: rgba(96, 156, 255, 0.10);
  color: #8ab6ff;
  font-variant-numeric: tabular-nums;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.status-pill.hit {
  background: rgba(44, 255, 179, 0.15);
  color: var(--accent);
}

.status-pill.miss {
  background: rgba(255, 169, 64, 0.15);
  color: #ffa940;
}

.status-pill.warn {
  background: rgba(255, 96, 96, 0.15);
  color: #ff8b8b;
}

.status-pill.neutral {
  background: rgba(255, 255, 255, 0.08);
  color: var(--muted);
}

@media (max-width: 980px) {
  .charts-row {
    grid-template-columns: 1fr;
  }

  .scope-clarity-grid {
    grid-template-columns: 1fr;
  }

  .request-lineage-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .llm-summary-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 640px) {
  .request-lineage-header {
    flex-direction: column;
  }

  .request-lineage-grid {
    grid-template-columns: 1fr;
  }

  .llm-summary-grid {
    grid-template-columns: 1fr;
  }
}

/* ── V10.3 Session Cost Budget bar ────────────────────── */
.budget-bar-wrap {
  padding: 16px 20px;
  margin-bottom: 20px;
}

.budget-bar-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.budget-bar-title {
  font-weight: 600;
  font-size: 0.9rem;
  flex: 1;
}

.budget-bar-amounts {
  font-size: 0.88rem;
  color: var(--muted);
}

.budget-bar-sep { margin: 0 4px; }

.budget-bar-pct {
  font-weight: 700;
  font-size: 0.88rem;
  min-width: 44px;
  text-align: right;
}

.budget-track {
  height: 8px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  overflow: hidden;
}

.budget-fill {
  height: 100%;
  border-radius: 6px;
  transition: width 0.5s ease;
}

.budget-fill.budget-ok        { background: var(--accent); }
.budget-fill.budget-warning   { background: #ffa940; }
.budget-fill.budget-exhausted { background: #ff6060; }

.budget-bar-pct.budget-ok        { color: var(--accent); }
.budget-bar-pct.budget-warning   { color: #ffa940; }
.budget-bar-pct.budget-exhausted { color: #ff6060; }

/* V10.16 — Pipeline Before/After */
.pipeline-example-section { margin-top: 20px; }
.pipeline-example-section h3 { margin: 0 0 6px; font-size: 1rem; }
.pipeline-ba-grid {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: 16px;
  align-items: stretch;
  margin-top: 14px;
}
.pipeline-ba-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
}
.before-card { border-color: rgba(255, 169, 64, 0.25); }
.after-card  { border-color: rgba(44, 255, 179, 0.25); }
.ba-badge {
  display: inline-block;
  width: fit-content;
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.ba-badge.before { background: rgba(255, 169, 64, 0.18); color: #ffa940; }
.ba-badge.after  { background: rgba(44, 255, 179, 0.18); color: var(--accent); }
.ba-code {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.78rem;
  line-height: 1.6;
  white-space: pre-wrap;
  padding: 12px;
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.25);
  flex: 1;
}
.ba-tokens {
  font-size: 0.75rem;
  color: var(--muted);
  margin-top: auto;
}
.pipeline-ba-arrow {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
.arrow-label {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--primary);
}
.arrow-icon {
  font-size: 1.8rem;
  color: var(--primary);
}
.arrow-steps {
  font-size: 0.65rem;
  color: var(--muted);
  text-align: center;
}
@media (max-width: 768px) {
  .pipeline-ba-grid { grid-template-columns: 1fr; }
  .pipeline-ba-arrow { flex-direction: row; gap: 8px; padding: 4px 0; }
  .arrow-icon { font-size: 1.2rem; transform: rotate(90deg); }
}

</style>
