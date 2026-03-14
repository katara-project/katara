<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Benchmarks</h2>
        <p class="muted">Live token reduction measurements per intent category.</p>
      </div>
      <span v-if="metrics.connected" class="live-badge">● Live</span>
      <span v-else class="live-badge offline">○ Offline</span>
    </header>

    <section class="card" v-if="benchmarks.length">
      <h3>Token Reduction by Intent</h3>
      <div class="bench-table">
        <div class="bench-row bench-header">
          <span>Intent</span>
          <span>Requests</span>
          <span>Raw</span>
          <span>Compiled</span>
          <span>Saved</span>
          <span>Reduction</span>
        </div>
        <div v-for="row in benchmarks" :key="row.intent" class="bench-row">
          <span class="bench-category">{{ row.intent }}</span>
          <span>{{ row.requests.toLocaleString() }}</span>
          <span>{{ row.raw.toLocaleString() }}</span>
          <span>{{ row.compiled.toLocaleString() }}</span>
          <span class="bench-saved">{{ row.saved > 0 ? '−' + row.saved.toLocaleString() : '—' }}</span>
          <span class="bench-pct">
            <span class="bench-bar" :style="{ width: row.reduction + '%' }"></span>
            {{ row.reduction }}%
          </span>
        </div>
      </div>
    </section>
    <section class="card" v-else>
      <h3>Token Reduction by Intent</h3>
      <p class="muted">No requests yet. Send a <code>POST /v1/compile</code> or <code>/v1/chat/completions</code> to see live data.</p>
    </section>

    <div class="bench-summary-grid">
      <section class="card bench-summary">
        <h3>Aggregate Results</h3>
        <div class="summary-items">
          <div class="summary-item">
            <span class="muted">Average reduction</span>
            <strong>{{ avgReduction }}%</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Best case</span>
            <strong>{{ bestReduction }}%</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Worst case</span>
            <strong>{{ worstReduction }}%</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Total tokens saved</span>
            <strong>{{ totalSaved.toLocaleString() }}</strong>
          </div>
        </div>
      </section>
      <section class="card bench-summary">
        <h3>Cumulative Overview</h3>
        <div class="summary-items">
          <div class="summary-item">
            <span class="muted">Total requests</span>
            <strong>{{ metrics.totalRequests.toLocaleString() }}</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Cache hit ratio</span>
            <strong>{{ metrics.cacheHitRatio }}%</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Efficiency score</span>
            <strong>{{ metrics.efficiencyScore }}%</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Intent categories</span>
            <strong>{{ benchmarks.length }}</strong>
          </div>
        </div>
      </section>
    </div>

    <section class="card chart-section">
      <h3>Token Trends (live)</h3>
      <TvChart :series="chartSeries" :labels="chartLabels" :height="220" />
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMetricsStore } from '../store/metrics'
import TvChart from '../components/TvChart.vue'

const metrics = useMetricsStore()

const INTENT_LABELS: Record<string, string> = {
  debug: 'Debug / Trace',
  summarize: 'Summarization',
  review: 'Code Review',
  general: 'General',
  ocr: 'OCR',
}

const benchmarks = computed(() => {
  const stats = metrics.intentStats
  return Object.entries(stats)
    .map(([intent, s]) => {
      const saved = Math.max(0, s.raw_tokens - s.compiled_tokens)
      const reduction = s.raw_tokens > 0
        ? Math.max(0, Math.round((saved / s.raw_tokens) * 1000) / 10)
        : 0
      return {
        intent: INTENT_LABELS[intent] ?? intent,
        requests: s.requests,
        raw: s.raw_tokens,
        compiled: s.compiled_tokens,
        saved,
        reduction,
      }
    })
    .sort((a, b) => b.requests - a.requests)
})

const avgReduction = computed(() => {
  if (!benchmarks.value.length) return 0
  const sum = benchmarks.value.reduce((acc, r) => acc + r.reduction, 0)
  return Math.round(sum / benchmarks.value.length * 10) / 10
})
const bestReduction = computed(() =>
  benchmarks.value.length ? Math.max(...benchmarks.value.map(r => r.reduction)) : 0
)
const worstReduction = computed(() =>
  benchmarks.value.length ? Math.min(...benchmarks.value.map(r => r.reduction)) : 0
)
const totalSaved = computed(() =>
  benchmarks.value.reduce((acc, r) => acc + r.saved, 0)
)

const chartLabels = computed(() =>
  metrics.historyRaw.length
    ? metrics.historyRaw.map((_: number, i: number) => `#${i + 1}`)
    : ['—']
)
const chartSeries = computed(() => {
  const raw = metrics.historyRaw.length ? [...metrics.historyRaw] : [0]
  const compiled = metrics.historyCompiled.length ? [...metrics.historyCompiled] : [0]
  const saved = raw.map((r: number, i: number) => r - (compiled[i] ?? 0))
  return [
    { name: 'Raw tokens', data: raw, color: '#ffa940' },
    { name: 'Compiled', data: compiled, color: 'var(--primary)' },
    { name: 'Saved', data: saved, color: 'var(--accent)' },
  ]
})
</script>

<style scoped>
.bench-table { display: flex; flex-direction: column; margin-top: 12px; }
.bench-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr 1fr 2fr;
  gap: 12px;
  padding: 12px 16px;
  align-items: center;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  font-size: 0.9rem;
}
.bench-header {
  color: var(--muted);
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom-color: rgba(255, 255, 255, 0.08);
}
.bench-category { font-weight: 600; }
.bench-saved { color: var(--accent); font-weight: 600; }
.bench-pct {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 700;
  color: var(--primary);
}
.bench-bar {
  height: 6px;
  border-radius: 3px;
  background: linear-gradient(90deg, var(--primary), var(--accent));
  flex-shrink: 0;
}
.bench-summary-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-top: 20px; }
.bench-summary h3 { margin: 0 0 16px; }
.summary-items { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.summary-item { display: flex; flex-direction: column; gap: 4px; }
.summary-item span { font-size: 0.82rem; }
.summary-item strong { font-size: 1.3rem; color: var(--primary); }
.chart-section { margin-top: 20px; }
.chart-section h3 { margin: 0 0 16px; font-size: 1rem; }
.live-badge {
  font-size: 0.82rem;
  color: var(--accent);
  font-weight: 600;
}
.live-badge.offline { color: var(--muted); }
@media (max-width: 1100px) {
  .bench-summary-grid { grid-template-columns: 1fr; }
  .bench-row { grid-template-columns: 1.5fr repeat(5, 1fr); font-size: 0.82rem; }
}
@media (max-width: 600px) {
  .bench-row { grid-template-columns: 1fr 1fr; gap: 6px; font-size: 0.78rem; padding: 10px 12px; }
  .bench-header { display: none; }
  .bench-category { grid-column: 1 / -1; }
  .summary-items { grid-template-columns: 1fr; }
}
</style>
