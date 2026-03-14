<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Runtime Audit</h2>
        <p class="muted">Rolling audit trail of the latest routed requests, including upstream lineage, routed target, cache behavior, and sensitive overrides.</p>
      </div>
      <span v-if="metrics.connected" class="live-badge">● Live</span>
      <span v-else class="live-badge offline">○ Offline</span>
    </header>

    <section class="card" v-if="auditRows.length">
      <h3>Recent Request History</h3>
      <div class="audit-controls">
        <label class="control-field">
          <span>Time window</span>
          <select v-model="timeFilter">
            <option value="24h">24h</option>
            <option value="7d">7d</option>
            <option value="custom">Custom</option>
          </select>
        </label>
        <label v-if="timeFilter === 'custom'" class="control-field">
          <span>From</span>
          <input v-model="customFrom" type="datetime-local" />
        </label>
        <label v-if="timeFilter === 'custom'" class="control-field">
          <span>To</span>
          <input v-model="customTo" type="datetime-local" />
        </label>
        <label class="control-field">
          <span>Tenant</span>
          <select v-model="tenantFilter">
            <option value="all">All</option>
            <option v-for="tenant in tenantOptions" :key="tenant" :value="tenant">{{ tenant }}</option>
          </select>
        </label>
        <label class="control-field">
          <span>Project</span>
          <select v-model="projectFilter">
            <option value="all">All</option>
            <option v-for="project in projectOptions" :key="project" :value="project">{{ project }}</option>
          </select>
        </label>
        <button type="button" class="export-btn" @click="exportScopedCsv">Export CSV</button>
      </div>
      <div class="audit-table">
        <div class="audit-row audit-header">
          <span>Time</span>
          <span>Scope</span>
          <span>Client</span>
          <span>Upstream</span>
          <span>Routed</span>
          <span>Intent</span>
          <span>Flags</span>
        </div>
        <div v-for="row in auditRows" :key="row.key" class="audit-row">
          <span>{{ row.time }}</span>
          <span>
            <strong>{{ row.tenantId }}</strong>
            <small>{{ row.projectId }}</small>
          </span>
          <span>
            <strong>{{ row.clientApp }}</strong>
            <small>{{ row.upstreamProvider }}</small>
          </span>
          <span>
            <strong>{{ row.upstreamModel }}</strong>
            <small>reported by client</small>
          </span>
          <span>
            <strong>{{ row.routedModel }}</strong>
            <small>{{ row.routedProvider }}</small>
          </span>
          <span>{{ row.intent }}</span>
          <span class="flag-stack">
            <span class="route-pill" :class="row.routeClass">{{ row.routeLabel }}</span>
            <span class="status-pill" :class="row.cacheClass">{{ row.cacheLabel }}</span>
            <span class="status-pill" :class="row.sensitiveClass">{{ row.sensitiveLabel }}</span>
          </span>
        </div>
      </div>
    </section>

    <section class="card" v-else>
      <h3>Recent Request History</h3>
      <p class="muted">No routed requests yet. Send a few compile or chat requests to populate the runtime audit trail.</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useMetricsStore } from '../store/metrics'
import { classifyRoute } from '../utils/providers'

const metrics = useMetricsStore()
const STORAGE_KEY = 'distira.runtimeAudit.filters.v1'

type TimeFilter = '24h' | '7d' | 'custom'

interface StoredAuditFilters {
  tenantFilter: string
  projectFilter: string
  timeFilter: TimeFilter
  customFrom: string
  customTo: string
}

function readStoredFilters(): StoredAuditFilters {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) {
      return {
        tenantFilter: 'all',
        projectFilter: 'all',
        timeFilter: '24h',
        customFrom: '',
        customTo: '',
      }
    }

    const parsed = JSON.parse(raw) as Partial<StoredAuditFilters>
    const timeFilter = parsed.timeFilter === '7d' || parsed.timeFilter === 'custom' ? parsed.timeFilter : '24h'

    return {
      tenantFilter: parsed.tenantFilter || 'all',
      projectFilter: parsed.projectFilter || 'all',
      timeFilter,
      customFrom: parsed.customFrom || '',
      customTo: parsed.customTo || '',
    }
  } catch {
    return {
      tenantFilter: 'all',
      projectFilter: 'all',
      timeFilter: '24h',
      customFrom: '',
      customTo: '',
    }
  }
}

const stored = readStoredFilters()
const tenantFilter = ref(stored.tenantFilter)
const projectFilter = ref(stored.projectFilter)
const timeFilter = ref<TimeFilter>(stored.timeFilter)
const customFrom = ref(stored.customFrom)
const customTo = ref(stored.customTo)

watch([tenantFilter, projectFilter, timeFilter, customFrom, customTo], () => {
  const payload: StoredAuditFilters = {
    tenantFilter: tenantFilter.value,
    projectFilter: projectFilter.value,
    timeFilter: timeFilter.value,
    customFrom: customFrom.value,
    customTo: customTo.value,
  }
  localStorage.setItem(STORAGE_KEY, JSON.stringify(payload))
})

// classifyRoute imported from ../utils/providers

const tenantOptions = computed(() => {
  const values = new Set(
    metrics.requestHistory.map((entry) => entry.tenant_id || 'default-tenant')
  )
  return [...values].sort((a, b) => a.localeCompare(b))
})

const projectOptions = computed(() => {
  const values = new Set(
    metrics.requestHistory.map((entry) => entry.project_id || 'default-project')
  )
  return [...values].sort((a, b) => a.localeCompare(b))
})

const filteredHistory = computed(() => {
  const nowTs = Math.floor(Date.now() / 1000)
  const customFromTs = customFrom.value
    ? Math.floor(new Date(customFrom.value).getTime() / 1000)
    : null
  const customToTs = customTo.value
    ? Math.floor(new Date(customTo.value).getTime() / 1000)
    : null

  const inTimeWindow = (ts: number) => {
    if (timeFilter.value === '24h') {
      return ts >= nowTs - 24 * 60 * 60
    }
    if (timeFilter.value === '7d') {
      return ts >= nowTs - 7 * 24 * 60 * 60
    }

    if (customFromTs && ts < customFromTs) return false
    if (customToTs && ts > customToTs) return false
    return true
  }

  return metrics.requestHistory.filter((entry) => {
    const tenant = entry.tenant_id || 'default-tenant'
    const project = entry.project_id || 'default-project'
    const tenantOk = tenantFilter.value === 'all' || tenant === tenantFilter.value
    const projectOk = projectFilter.value === 'all' || project === projectFilter.value
    const timeOk = inTimeWindow(entry.ts)
    return tenantOk && projectOk && timeOk
  })
})

const auditRows = computed(() => {
  return [...filteredHistory.value]
    .reverse()
    .map((entry, index) => {
      const route = classifyRoute(entry.routed_provider)
      return {
        key: `${entry.ts}-${index}`,
        time: new Date(entry.ts * 1000).toLocaleTimeString(),
        clientApp: entry.client_app || 'Unknown client app',
        upstreamProvider: entry.upstream_provider || 'Not supplied',
        upstreamModel: entry.upstream_model || 'Unknown upstream model',
        routedProvider: entry.routed_provider,
        routedModel: entry.routed_model,
        intent: entry.intent,
        tenantId: entry.tenant_id || 'default-tenant',
        projectId: entry.project_id || 'default-project',
        routeLabel: route.routeLabel,
        routeClass: route.routeClass,
        cacheLabel: entry.cache_hit ? 'Cache hit' : 'Cache miss',
        cacheClass: entry.cache_hit ? 'hit' : 'miss',
        sensitiveLabel: entry.sensitive ? 'Sensitive override' : 'Standard routing',
        sensitiveClass: entry.sensitive ? 'warn' : 'neutral',
      }
    })
})

function exportScopedCsv() {
  const rows = [...filteredHistory.value].reverse()
  if (!rows.length) return

  const esc = (v: unknown) => {
    const s = String(v ?? '')
    const escaped = s.replace(/"/g, '""')
    return `"${escaped}"`
  }

  const header = [
    'ts',
    'tenant_id',
    'project_id',
    'policy_pack',
    'client_app',
    'upstream_provider',
    'upstream_model',
    'routed_provider',
    'routed_model',
    'intent',
    'cache_hit',
    'sensitive',
  ]

  const lines = [header.join(',')]
  for (const entry of rows) {
    lines.push(
      [
        entry.ts,
        entry.tenant_id || 'default-tenant',
        entry.project_id || 'default-project',
        entry.policy_pack || 'baseline',
        entry.client_app || '',
        entry.upstream_provider || '',
        entry.upstream_model || '',
        entry.routed_provider,
        entry.routed_model,
        entry.intent,
        entry.cache_hit,
        entry.sensitive,
      ]
        .map(esc)
        .join(',')
    )
  }

  const blob = new Blob([lines.join('\n')], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `distira-runtime-audit-${timeFilter.value}-${tenantFilter.value}-${projectFilter.value}.csv`
  a.click()
  URL.revokeObjectURL(url)
}
</script>

<style scoped>
.audit-table {
  display: flex;
  flex-direction: column;
  margin-top: 12px;
}

.audit-controls {
  display: flex;
  align-items: end;
  gap: 10px;
  margin-top: 12px;
  flex-wrap: wrap;
}

.control-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.control-field span {
  font-size: 0.75rem;
  color: var(--muted);
}

.control-field select {
  min-width: 170px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: var(--text);
  border-radius: 8px;
  padding: 6px 8px;
}

.control-field input {
  min-width: 190px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: var(--text);
  border-radius: 8px;
  padding: 6px 8px;
}

.export-btn {
  border: 1px solid rgba(57, 211, 255, 0.3);
  background: rgba(57, 211, 255, 0.1);
  color: var(--primary);
  border-radius: 8px;
  padding: 6px 10px;
  font-weight: 600;
  cursor: pointer;
}

.audit-row {
  display: grid;
  grid-template-columns: 0.9fr 1.1fr 1.1fr 1.2fr 1.2fr 0.8fr 1.4fr;
  gap: 12px;
  align-items: start;
  padding: 14px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  font-size: 0.88rem;
}

.audit-header {
  color: var(--muted);
  font-size: 0.78rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  border-bottom-color: rgba(255, 255, 255, 0.08);
}

.audit-row strong {
  display: block;
  font-size: 0.92rem;
}

.audit-row small {
  display: block;
  margin-top: 4px;
  color: var(--muted);
}

.flag-stack {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.live-badge {
  font-size: 0.82rem;
  color: var(--accent);
  font-weight: 600;
}

.live-badge.offline {
  color: var(--muted);
}

.route-pill,
.status-pill {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.route-pill.local,
.status-pill.hit {
  background: rgba(44, 255, 179, 0.15);
  color: var(--accent);
}

.route-pill.cloud,
.status-pill.warn {
  background: rgba(255, 96, 96, 0.15);
  color: #ff8b8b;
}

.route-pill.midtier {
  background: rgba(96, 156, 255, 0.15);
  color: #8ab6ff;
}

.status-pill.miss {
  background: rgba(255, 169, 64, 0.15);
  color: #ffa940;
}

.status-pill.neutral {
  background: rgba(255, 255, 255, 0.08);
  color: var(--muted);
}

@media (max-width: 1120px) {
  .audit-row {
    grid-template-columns: 1fr 1fr;
  }

  .audit-header {
    display: none;
  }
}
</style>