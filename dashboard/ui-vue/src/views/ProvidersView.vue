<template>
  <section class="providers-view">
    <h2><SvgIcon name="server" :size="22" /> Provider Health Observatory</h2>
    <p class="subtitle">Real-time health, latency, and reliability of all configured LLM providers.</p>

    <!-- Summary cards -->
    <div class="summary-row">
      <div class="summary-card">
        <span class="summary-value">{{ totalProviders }}</span>
        <span class="summary-label">Providers tracked</span>
      </div>
      <div class="summary-card">
        <span class="summary-value healthy">{{ healthyCount }}</span>
        <span class="summary-label">Healthy</span>
      </div>
      <div class="summary-card">
        <span class="summary-value degraded">{{ degradedCount }}</span>
        <span class="summary-label">Degraded</span>
      </div>
      <div class="summary-card">
        <span class="summary-value down">{{ downCount }}</span>
        <span class="summary-label">Down</span>
      </div>
    </div>

    <!-- Provider health table -->
    <div class="table-wrap">
      <table class="health-table">
        <thead>
          <tr>
            <th>Provider</th>
            <th>Status</th>
            <th class="num">Requests</th>
            <th class="num">Errors</th>
            <th class="num">Error Rate</th>
            <th class="num">Avg Latency</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="p in providers" :key="p.provider">
            <td class="provider-name">{{ friendlyName(p.provider) }}<span class="provider-key">{{ p.provider }}</span></td>
            <td><span class="status-badge" :class="p.status">{{ p.status }}</span></td>
            <td class="num">{{ p.requests.toLocaleString() }}</td>
            <td class="num">{{ p.errors }}</td>
            <td class="num">{{ (p.error_rate * 100).toFixed(1) }}%</td>
            <td class="num">{{ p.avg_latency_ms > 0 ? p.avg_latency_ms.toFixed(0) + ' ms' : '–' }}</td>
          </tr>
          <tr v-if="providers.length === 0">
            <td colspan="6" class="empty">No provider activity yet. Send a request through <code>/v1/chat/completions</code> to start tracking.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Export section -->
    <div class="export-section">
      <h3><SvgIcon name="download" :size="18" /> Metrics Export</h3>
      <p>Download cumulative metrics for enterprise reporting — includes per-provider and per-intent breakdowns.</p>
      <button class="export-btn" @click="exportMetrics" :disabled="exporting">
        <SvgIcon name="download" :size="16" /> {{ exporting ? 'Exporting…' : 'Export JSON' }}
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import SvgIcon from '../components/SvgIcon.vue'
import { useMetricsStore } from '../store/metrics'
import { friendlyProvider } from '../utils/providers'

interface ProviderHealth {
  provider: string
  requests: number
  errors: number
  error_rate: number
  avg_latency_ms: number
  status: string
}

const metrics = useMetricsStore()
const exporting = ref(false)

const providers = computed<ProviderHealth[]>(() => {
  const raw = metrics.providerHealth
  if (!Array.isArray(raw)) return []
  return [...raw].sort((a, b) => b.requests - a.requests)
})

const totalProviders = computed(() => providers.value.length)
const healthyCount = computed(() => providers.value.filter(p => p.status === 'healthy').length)
const degradedCount = computed(() => providers.value.filter(p => p.status === 'degraded').length)
const downCount = computed(() => providers.value.filter(p => p.status === 'down').length)

function friendlyName(key: string): string {
  return friendlyProvider(key)
}

async function exportMetrics() {
  exporting.value = true
  try {
    const resp = await fetch('http://localhost:8080/v1/metrics/export')
    const data = await resp.json()
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'distira-metrics-export-' + new Date().toISOString().slice(0, 10) + '.json'
    a.click()
    URL.revokeObjectURL(url)
  } finally {
    exporting.value = false
  }
}
</script>

<style scoped>
.providers-view { max-width: 960px; margin: 0 auto; }
h2 { display: flex; align-items: center; gap: 8px; margin-bottom: 4px; }
.subtitle { color: var(--text-muted, #888); margin-bottom: 24px; font-size: 0.95rem; }

.summary-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 24px; }
.summary-card {
  background: var(--card-bg, #1a1a2e);
  border-radius: 10px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}
.summary-value { font-size: 2rem; font-weight: 700; color: var(--text, #eee); }
.summary-value.healthy { color: #4ade80; }
.summary-value.degraded { color: #facc15; }
.summary-value.down { color: #f87171; }
.summary-label { font-size: 0.8rem; color: var(--text-muted, #888); text-transform: uppercase; letter-spacing: 0.5px; }

.table-wrap { overflow-x: auto; margin-bottom: 32px; }
.health-table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
.health-table th {
  text-align: left;
  padding: 10px 12px;
  border-bottom: 2px solid var(--border, #333);
  color: var(--text-muted, #888);
  font-weight: 600;
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.health-table th.num, .health-table td.num { text-align: right; }
.health-table td {
  padding: 10px 12px;
  border-bottom: 1px solid var(--border, #222);
  color: var(--text, #eee);
}
.health-table tbody tr:hover { background: rgba(255,255,255,0.03); }

.provider-name { font-weight: 600; }
.provider-key { display: block; font-size: 0.75rem; color: var(--text-muted, #666); font-weight: 400; font-family: monospace; }

.status-badge {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 12px;
  font-size: 0.78rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}
.status-badge.healthy { background: rgba(74,222,128,0.15); color: #4ade80; }
.status-badge.degraded { background: rgba(250,204,21,0.15); color: #facc15; }
.status-badge.down { background: rgba(248,113,113,0.15); color: #f87171; }

.empty { text-align: center; color: var(--text-muted, #666); padding: 32px; }

.export-section {
  background: var(--card-bg, #1a1a2e);
  border-radius: 10px;
  padding: 20px;
}
.export-section h3 { display: flex; align-items: center; gap: 6px; margin-bottom: 6px; }
.export-section p { color: var(--text-muted, #888); font-size: 0.9rem; margin-bottom: 14px; }
.export-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 18px;
  background: var(--accent, #6366f1);
  color: #fff;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.9rem;
  transition: opacity 0.2s;
}
.export-btn:hover { opacity: 0.85; }
.export-btn:disabled { opacity: 0.5; cursor: not-allowed; }

@media (max-width: 640px) {
  .summary-row { grid-template-columns: repeat(2, 1fr); }
}
</style>
