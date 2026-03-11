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
    </div>
    <div class="two-col">
      <EfficiencyGauge :score="metrics.efficiencyScore" />
      <FlowVisualizer />
    </div>
    <section class="card chart-section">
      <h3>Token Trends (24h)</h3>
      <TvChart :series="trendSeries" :labels="trendLabels" :height="220" />
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMetricsStore } from '../store/metrics'
import MetricCard from '../components/MetricCard.vue'
import EfficiencyGauge from '../components/EfficiencyGauge.vue'
import FlowVisualizer from '../components/FlowVisualizer.vue'
import SparklineChart from '../components/SparklineChart.vue'
import TvChart from '../components/TvChart.vue'

const metrics = useMetricsStore()

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

// TvChart: label each history point as #1, #2, …
const trendLabels = computed(() =>
  metrics.historyRaw.length
    ? metrics.historyRaw.map((_: number, i: number) => `#${i + 1}`)
    : ['—']
)
const trendSeries = computed(() => [
  { name: 'Raw', data: metrics.historyRaw.length ? [...metrics.historyRaw] : [0], color: '#ffa940' },
  { name: 'Compiled', data: metrics.historyCompiled.length ? [...metrics.historyCompiled] : [0], color: 'var(--primary)' },
  { name: 'Reused', data: metrics.historyReused.length ? [...metrics.historyReused] : [0], color: 'var(--secondary)' },
])
</script>

<style scoped>
.chart-section {
  margin-top: 20px;
}
.chart-section h3 {
  margin: 0 0 16px;
  font-size: 1rem;
}
</style>
