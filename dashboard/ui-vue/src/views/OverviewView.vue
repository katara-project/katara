<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Overview</h2>
        <p class="muted">A visual control plane for AI traffic and context optimization.</p>
      </div>
    </header>
    <div class="metrics-grid">
      <MetricCard label="Raw Tokens" :value="metrics.rawTokens" hint="Estimated before compilation" />
      <MetricCard label="Compiled Tokens" :value="metrics.compiledTokens" hint="Estimated after compiler" />
      <MetricCard label="Memory Reused" :value="metrics.memoryReusedTokens" hint="Tokens not re-sent" />
      <MetricCard label="Local Routing" :value="metrics.localRatio + '%'" hint="Requests kept local" />
    </div>
    <div class="two-col">
      <EfficiencyGauge :score="metrics.efficiencyScore" />
      <FlowVisualizer />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useMetricsStore } from '../store/metrics'
import MetricCard from '../components/MetricCard.vue'
import EfficiencyGauge from '../components/EfficiencyGauge.vue'
import FlowVisualizer from '../components/FlowVisualizer.vue'
const metrics = useMetricsStore()
</script>
