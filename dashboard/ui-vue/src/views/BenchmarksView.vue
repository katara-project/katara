<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Benchmarks</h2>
        <p class="muted">Token reduction and latency measurements across different prompt categories.</p>
      </div>
    </header>

    <section class="card">
      <h3>Token Reduction Benchmarks</h3>
      <div class="bench-table">
        <div class="bench-row bench-header">
          <span>Prompt Category</span>
          <span>Raw</span>
          <span>Compiled</span>
          <span>Saved</span>
          <span>Reduction</span>
        </div>
        <div v-for="row in benchmarks" :key="row.category" class="bench-row">
          <span class="bench-category">{{ row.category }}</span>
          <span>{{ row.raw.toLocaleString() }}</span>
          <span>{{ row.compiled.toLocaleString() }}</span>
          <span class="bench-saved">&minus;{{ (row.raw - row.compiled).toLocaleString() }}</span>
          <span class="bench-pct">
            <span class="bench-bar" :style="{ width: row.reduction + '%' }"></span>
            {{ row.reduction }}%
          </span>
        </div>
      </div>
    </section>

    <div class="bench-summary-grid">
      <section class="card bench-summary">
        <h3>Aggregate Results</h3>
        <div class="summary-items">
          <div class="summary-item">
            <span class="muted">Average reduction</span>
            <strong>63.8%</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Best case</span>
            <strong>72.4%</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Worst case</span>
            <strong>49.1%</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Total tokens saved</span>
            <strong>18,640</strong>
          </div>
        </div>
      </section>
      <section class="card bench-summary">
        <h3>Latency Impact</h3>
        <div class="summary-items">
          <div class="summary-item">
            <span class="muted">Avg compile time</span>
            <strong>&lt; 2ms</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Cache lookup</span>
            <strong>&lt; 0.5ms</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Net latency saved</span>
            <strong>&minus;340ms/req</strong>
          </div>
          <div class="summary-item">
            <span class="muted">Overhead ratio</span>
            <strong>0.4%</strong>
          </div>
        </div>
      </section>
    </div>

    <section class="card chart-section">
      <h3>Reduction Over Time</h3>
      <TvChart :series="chartSeries" :labels="chartLabels" :height="220" />
    </section>
  </div>
</template>

<script setup lang="ts">
import TvChart from '../components/TvChart.vue'

const benchmarks = [
  { category: 'Code review', raw: 2100, compiled: 760, reduction: 63.8 },
  { category: 'Debug trace', raw: 3400, compiled: 1180, reduction: 65.3 },
  { category: 'Summarization', raw: 4200, compiled: 1160, reduction: 72.4 },
  { category: 'Chat (general)', raw: 1800, compiled: 920, reduction: 49.1 },
  { category: 'System prompt', raw: 1100, compiled: 380, reduction: 65.5 },
  { category: 'Multi-turn', raw: 5600, compiled: 1720, reduction: 69.3 },
]

const chartLabels = ['Week 1', 'Week 2', 'Week 3', 'Week 4', 'Week 5', 'Week 6']
const chartSeries = [
  { name: 'Raw tokens', data: [22000, 24800, 27400, 28600, 26800, 29200], color: '#ffa940' },
  { name: 'Compiled', data: [9200, 9600, 8700, 8400, 7900, 8120], color: 'var(--primary)' },
  { name: 'Saved', data: [12800, 15200, 18700, 20200, 18900, 21080], color: 'var(--accent)' },
]
</script>

<style scoped>
.bench-table { display: flex; flex-direction: column; margin-top: 12px; }
.bench-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr 2fr;
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
@media (max-width: 1100px) {
  .bench-summary-grid { grid-template-columns: 1fr; }
  .bench-row { grid-template-columns: 1.5fr repeat(4, 1fr); font-size: 0.82rem; }
}
@media (max-width: 600px) {
  .bench-row { grid-template-columns: 1fr 1fr; gap: 6px; font-size: 0.78rem; padding: 10px 12px; }
  .bench-header { display: none; }
  .bench-category { grid-column: 1 / -1; }
  .summary-items { grid-template-columns: 1fr; }
}
</style>
