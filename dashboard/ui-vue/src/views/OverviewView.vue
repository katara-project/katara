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
import { useMetricsStore } from '../store/metrics'
import MetricCard from '../components/MetricCard.vue'
import EfficiencyGauge from '../components/EfficiencyGauge.vue'
import FlowVisualizer from '../components/FlowVisualizer.vue'
import SparklineChart from '../components/SparklineChart.vue'
import TvChart from '../components/TvChart.vue'

const metrics = useMetricsStore()

const rawHistory = [14200, 15800, 16100, 17400, 16900, 18100, 17600, 18420]
const compiledHistory = [4100, 4600, 4400, 5100, 4800, 5300, 5000, 5210]
const memoryHistory = [3200, 4100, 4800, 5200, 5500, 5800, 5900, 6030]
const localHistory = [52, 54, 56, 58, 57, 59, 60, 61]

const trendLabels = ['00:00', '04:00', '08:00', '12:00', '16:00', '20:00', '24:00']
const trendSeries = [
  { name: 'Raw', data: [12400, 14800, 18200, 21600, 19400, 17800, 18420], color: '#ffa940' },
  { name: 'Compiled', data: [3600, 4200, 5400, 6100, 5600, 5100, 5210], color: 'var(--primary)' },
  { name: 'Reused', data: [2800, 3400, 4600, 5800, 5200, 5900, 6030], color: 'var(--secondary)' },
]
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
