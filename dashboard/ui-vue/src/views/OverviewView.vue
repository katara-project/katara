<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Overview</h2>
        <p class="muted">A visual control plane for AI traffic and context optimization.</p>
      </div>
    </header>
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
      <MetricCard label="Cache Saved Tokens" :value="metrics.cacheSavedTokens.toLocaleString()" hint="Provider tokens avoided via cached responses" accent="primary">
        <SparklineChart :data="cacheSavedHistory" color="var(--primary)" :height="40" />
      </MetricCard>
    </div>
    <div class="two-col">
      <EfficiencyGauge :score="metrics.efficiencyScore" />
      <FlowVisualizer />
    </div>
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
      <TvChart :series="trendSeries" :labels="trendLabels" :height="220" />
    </section>

    <section class="card scope-clarity-section">
      <div class="scope-clarity-header">
        <div>
          <h3>Model Scope Clarity</h3>
          <p class="muted">This dashboard tracks the model routed by Katara after compilation and policy routing. It does not assume that the end-user assistant or client is the same model.</p>
        </div>
      </div>
      <div class="scope-clarity-grid">
        <div class="scope-card assistant-scope">
          <span class="tile-label">Assistant / Client Model</span>
          <strong>{{ leadUpstream.model }}</strong>
          <span class="tile-subtitle">{{ leadUpstream.provider }} · {{ leadUpstream.clientApp }}</span>
        </div>
        <div class="scope-card katara-scope">
          <span class="tile-label">KATARA Routed Model</span>
          <strong>{{ leadModel.model }}</strong>
          <span class="tile-subtitle">{{ leadModel.provider }} · {{ leadModel.routeLabel }}</span>
        </div>
      </div>
      <p class="scope-note">
        Katara metrics, sovereignty ratios, and token savings below always refer to the routed model selected by Katara, not the assistant brand or UI model shown upstream.
      </p>
      <div v-if="upstreamVisibilityWarning.show" class="upstream-warning-banner">
        <strong>{{ upstreamVisibilityWarning.title }}</strong>
        <p>{{ upstreamVisibilityWarning.message }}</p>
      </div>
    </section>

    <section class="card llm-summary-section">
      <div class="llm-summary-header">
        <div>
          <h3>Active LLM Routing</h3>
          <p class="muted">Which model Katara is sending traffic to, how much token reduction it gets, and whether routing stays sovereign.</p>
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

    <section class="card codegen-vs-review-section">
      <div class="section-heading">
        <div>
          <h3>Codegen vs Review</h3>
          <p class="muted">How much traffic is pure code generation vs review/refactor, and how efficiently Katara trims each intent.</p>
        </div>
        <span class="total-badge" v-if="codegenReview.hasData">{{ codegenReview.totalRequests }} req</span>
      </div>
      <div v-if="codegenReview.hasData" class="cvr-grid">
        <div class="cvr-tile">
          <span class="tile-label">Codegen</span>
          <strong class="cvr-count">{{ codegenReview.codegenRequests }}</strong>
          <span class="tile-subtitle">{{ codegenReview.codegenPct }}% of intents</span>
          <span class="cvr-reduction">{{ codegenReview.codegenReduction }}% avg reduction</span>
        </div>
        <div class="cvr-tile">
          <span class="tile-label">Review</span>
          <strong class="cvr-count">{{ codegenReview.reviewRequests }}</strong>
          <span class="tile-subtitle">{{ codegenReview.reviewPct }}% of intents</span>
          <span class="cvr-reduction">{{ codegenReview.reviewReduction }}% avg reduction</span>
        </div>
      </div>
      <p v-else class="muted">No codegen or review traffic yet. Send a few code requests to see this split.</p>
    </section>

    <section class="card request-lineage-section">
      <div class="request-lineage-header">
        <div>
          <h3>Last Request</h3>
          <p class="muted">Live lineage for the most recent request seen by Katara, including upstream client identity, routed target, cache behavior, and sensitivity override.</p>
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
          <p class="muted">What the calling client reports upstream, such as GPT-5.4 in VS Code Copilot. This is separate from Katara's routed target.</p>
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
              <td colspan="5" class="muted">No upstream client model reported yet. Katara can only show GPT-5.4 here if VS Code/Copilot exposes it or if runtime client context is updated live.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section class="card model-efficiency-section">
      <h3>Live AI Efficiency by Routed Model</h3>
      <p class="muted">Per-routed-model efficiency score with Sovereign Routing visibility. This table shows what Katara actually sent downstream, not the model selected in the upstream client UI.</p>
      <div class="model-table-wrap">
        <table class="model-table">
          <thead>
            <tr>
              <th>Model</th>
              <th>Provider</th>
              <th>Route</th>
              <th>Requests</th>
              <th>Saved Tokens</th>
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
              <td>{{ entry.requests }}</td>
              <td>{{ entry.savedTokens.toLocaleString() }}</td>
              <td>
                <strong>{{ entry.efficiency }}%</strong>
              </td>
              <td>
                <span class="sovereign-pill" :class="entry.sovereignClass">{{ entry.sovereignLabel }}</span>
              </td>
            </tr>
            <tr v-if="!modelRows.length">
              <td colspan="7" class="muted">No model data yet. Send a few requests to populate live stats.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section class="card intent-distribution-section">
      <div class="section-heading">
        <div>
          <h3>Intent Distribution</h3>
          <p class="muted">How Katara classified incoming requests. Each intent maps to a different routing target and context reduction strategy.</p>
        </div>
        <span class="total-badge">{{ metrics.totalRequests }} total</span>
      </div>
      <div v-if="intentRows.length" class="intent-grid">
        <div v-for="entry in intentRows" :key="entry.intent" class="intent-tile" :class="`intent-${entry.intent}`">
          <span class="intent-label">{{ entry.intent }}</span>
          <strong class="intent-count">{{ entry.requests }}</strong>
          <span class="intent-pct">{{ entry.pct }}%</span>
          <div class="intent-bar-track">
            <div class="intent-bar-fill" :style="{ width: entry.pct + '%' }"></div>
          </div>
          <span class="intent-reduction">{{ entry.avgReduction }}% reduction</span>
        </div>
      </div>
      <p v-else class="muted">No intent data yet. Send a few requests to see the distribution.</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useMetricsStore } from '../store/metrics'
import MetricCard from '../components/MetricCard.vue'
import EfficiencyGauge from '../components/EfficiencyGauge.vue'
import FlowVisualizer from '../components/FlowVisualizer.vue'
import SparklineChart from '../components/SparklineChart.vue'
import TvChart from '../components/TvChart.vue'

const metrics = useMetricsStore()
const configuredAssistantModelLabel = import.meta.env.VITE_ASSISTANT_MODEL_LABEL || 'External assistant or client model'

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

const cacheSavedHistory = computed(() => {
  if (!metrics.historyRaw.length) return [0]
  const steps = metrics.historyRaw.length
  const current = metrics.cacheSavedTokens
  return Array.from({ length: steps }, (_, idx) => Math.round((current * (idx + 1)) / steps))
})

// TvChart: prefer true hourly buckets when available, fallback to legacy history.
const trendWindowOptions = ['1h', '6h', '24h'] as const
const trendWindow = ref<(typeof trendWindowOptions)[number]>('24h')

const trendWindowLabel = computed(() => trendWindow.value)

const trendLastUpdate = computed(() => {
  if (!metrics.lastTs) return 'waiting for metrics...'
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
    .sort((a, b) => b.requests - a.requests || b.lastSeenTs - a.lastSeenTs)
})

const upstreamTableRows = computed(() => {
  return upstreamRows.value.map((entry) => ({
    ...entry,
    lastSeen: entry.lastSeenTs ? new Date(entry.lastSeenTs * 1000).toLocaleTimeString() : 'n/a',
  }))
})

function classifyRoute(provider: string) {
  const normalized = provider.toLowerCase()
  if (normalized.includes('ollama') || normalized.includes('local')) {
    return { routeLabel: 'Local sovereign', routeClass: 'local' }
  }
  if (normalized.includes('mistral')) {
    return { routeLabel: 'Mid-tier', routeClass: 'midtier' }
  }
  return { routeLabel: 'Cloud', routeClass: 'cloud' }
}

const modelRows = computed(() => {
  return Object.entries(metrics.modelStats)
    .map(([key, stat]) => {
      const savedTokens = Math.max(0, (stat.raw_tokens ?? 0) - (stat.compiled_tokens ?? 0))
      const sovereignRatio = Math.round(stat.sovereign_ratio ?? 0)
      const route = classifyRoute(stat.provider)
      return {
        key,
        model: stat.model,
        provider: stat.provider,
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
    provider: 'Waiting for first routed request',
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
      'Katara can only show the real Copilot model, such as GPT-5.4, when VS Code or the MCP client exposes it in request metadata or runtime client context. The routed-model tables below remain valid, but the upstream client model is currently unknown.',
  }
})

const lastRequestCard = computed(() => {
  const lastRequest = metrics.lastRequest
  if (!lastRequest) {
    return {
      clientApp: 'No request yet',
      upstreamProvider: 'Waiting for upstream metadata',
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
      seenAt: 'Awaiting first request',
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

// Codegen vs Review slice, derived from intentStats
const codegenReview = computed(() => {
  const stats = metrics.intentStats as Record<string, { requests: number; raw_tokens: number; compiled_tokens: number }>
  const codegen = stats['codegen']
  const review = stats['review']

  if (!codegen && !review) {
    return {
      hasData: false,
      totalRequests: 0,
      codegenRequests: 0,
      reviewRequests: 0,
      codegenPct: 0,
      reviewPct: 0,
      codegenReduction: 0,
      reviewReduction: 0,
    }
  }

  const totalRequests = (codegen?.requests ?? 0) + (review?.requests ?? 0)
  const safeTotal = totalRequests || 1

  const calcReduction = (entry: { raw_tokens: number; compiled_tokens: number } | undefined) => {
    if (!entry || !entry.raw_tokens) return 0
    const saved = Math.max(0, entry.raw_tokens - entry.compiled_tokens)
    return Math.round((saved / entry.raw_tokens) * 100)
  }

  return {
    hasData: true,
    totalRequests,
    codegenRequests: codegen?.requests ?? 0,
    reviewRequests: review?.requests ?? 0,
    codegenPct: Math.round(((codegen?.requests ?? 0) / safeTotal) * 100),
    reviewPct: Math.round(((review?.requests ?? 0) / safeTotal) * 100),
    codegenReduction: calcReduction(codegen),
    reviewReduction: calcReduction(review),
  }
})

// Intent distribution — aggregated from intentStats with percentage + avg reduction
const intentRows = computed(() => {
  const stats = metrics.intentStats as Record<string, { requests: number; raw_tokens: number; compiled_tokens: number }>
  const total = Object.values(stats).reduce((s, v) => s + v.requests, 0) || 1
  return Object.entries(stats)
    .map(([intent, stat]) => {
      const saved = Math.max(0, stat.raw_tokens - stat.compiled_tokens)
      const avgReduction = stat.raw_tokens > 0 ? Math.round((saved / stat.raw_tokens) * 100) : 0
      return {
        intent,
        requests: stat.requests,
        pct: Math.round((stat.requests / total) * 100),
        avgReduction,
      }
    })
    .sort((a, b) => b.requests - a.requests)
})
</script>

<style scoped>
.chart-section {
  margin-top: 20px;
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

.katara-scope {
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

/* ── Intent Distribution ──────────────────────────────── */
.intent-distribution-section {
  margin-top: 20px;
}

.intent-distribution-section .section-heading {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.total-badge {
  flex-shrink: 0;
  padding: 4px 12px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.07);
  color: var(--muted);
  font-size: 0.8rem;
  font-weight: 600;
}

.intent-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
  margin-top: 14px;
}

.intent-tile {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px 16px;
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
}

.intent-tile.intent-debug  { border-color: rgba(255, 104, 104, 0.3); background: linear-gradient(135deg, rgba(255, 104, 104, 0.1), rgba(255, 169, 64, 0.06)); }
.intent-tile.intent-review { border-color: rgba(96, 156, 255, 0.3);  background: linear-gradient(135deg, rgba(96, 156, 255, 0.1), rgba(0, 214, 143, 0.06)); }
.intent-tile.intent-summarize { border-color: rgba(0, 214, 143, 0.3); background: linear-gradient(135deg, rgba(0, 214, 143, 0.1), rgba(40, 120, 255, 0.06)); }
.intent-tile.intent-ocr    { border-color: rgba(187, 134, 252, 0.3); background: linear-gradient(135deg, rgba(187, 134, 252, 0.1), rgba(96, 156, 255, 0.06)); }
.intent-tile.intent-general { border-color: rgba(255, 255, 255, 0.1); }

.intent-label {
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--muted);
}

.intent-count {
  font-size: 1.6rem;
  font-weight: 700;
  line-height: 1;
}

.intent-pct {
  font-size: 0.82rem;
  color: var(--muted);
}

.intent-bar-track {
  height: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.08);
  margin: 4px 0 2px;
  overflow: hidden;
}

.intent-bar-fill {
  height: 100%;
  border-radius: 999px;
  background: var(--primary);
  transition: width 0.4s ease;
}

.intent-reduction {
  font-size: 0.77rem;
  color: var(--accent);
}

/* ── Codegen vs Review ──────────────────────────────── */
.codegen-vs-review-section {
  margin-top: 20px;
}

.codegen-vs-review-section .section-heading h3 {
  margin: 0 0 6px;
  font-size: 1rem;
}

.cvr-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-top: 14px;
}

.cvr-tile {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 16px;
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: linear-gradient(135deg, rgba(0, 214, 143, 0.12), rgba(40, 120, 255, 0.08));
}

.cvr-count {
  font-size: 1.6rem;
  font-weight: 700;
  line-height: 1;
}

.cvr-reduction {
  font-size: 0.8rem;
  color: var(--accent);
}

@media (max-width: 640px) {
  .cvr-grid {
    grid-template-columns: 1fr;
  }
}
</style>
