import { defineStore } from 'pinia'
import { ref, computed, onUnmounted } from 'vue'

export interface IntentStat {
  requests: number
  raw_tokens: number
  compiled_tokens: number
}

export interface ModelStat {
  model: string
  provider: string
  requests: number
  raw_tokens: number
  compiled_tokens: number
  memory_reused_tokens: number
  efficiency_score: number
  sovereign_requests: number
  non_sovereign_requests: number
  sovereign_ratio: number
}

export interface UpstreamStat {
  client_app: string
  upstream_provider: string
  upstream_model: string
  requests: number
  last_seen_ts: number
}

export interface RequestLineage {
  client_app?: string
  upstream_provider?: string
  upstream_model?: string
  tenant_id?: string
  project_id?: string
  policy_pack?: string
  routed_provider: string
  routed_model: string
  intent: string
  cache_hit: boolean
  sensitive: boolean
  ts: number
}

export interface VersionInfo {
  version: string
  product: string
}

export interface MetricsSnapshot {
  ts: number
  total_requests: number
  raw_tokens: number
  compiled_tokens: number
  memory_reused_tokens: number
  efficiency_score: number
  local_ratio: number
  cache_hits: number
  cache_misses: number
  cache_saved_tokens: number
  history_raw: number[]
  history_compiled: number[]
  history_reused: number[]
  history_hour_epochs?: number[]
  history_hour_raw?: number[]
  history_hour_compiled?: number[]
  history_hour_reused?: number[]
  routes_local: number
  routes_cloud: number
  routes_midtier: number
  intent_stats: Record<string, IntentStat>
  model_stats: Record<string, ModelStat>
  upstream_stats: Record<string, UpstreamStat>
  last_request?: RequestLineage
  request_history: RequestLineage[]
}

const SSE_URL = 'http://localhost:8080/v1/metrics/stream'
const VERSION_URL = 'http://localhost:8080/version'

export const useMetricsStore = defineStore('metrics', () => {
  // ── reactive state ─────────────────────────────────
  const rawTokens = ref(0)
  const compiledTokens = ref(0)
  const memoryReusedTokens = ref(0)
  const efficiencyScore = ref(0)
  const localRatio = ref(0)
  const cloudRatio = ref(0)
  const totalRequests = ref(0)
  const cacheHits = ref(0)
  const cacheMisses = ref(0)
  const cacheSavedTokens = ref(0)
  const historyRaw = ref<number[]>([])
  const historyCompiled = ref<number[]>([])
  const historyReused = ref<number[]>([])
  const historyHourEpochs = ref<number[]>([])
  const historyHourRaw = ref<number[]>([])
  const historyHourCompiled = ref<number[]>([])
  const historyHourReused = ref<number[]>([])
  const routesLocal = ref(0)
  const routesCloud = ref(0)
  const routesMidtier = ref(0)
  const connected = ref(false)
  const lastTs = ref(0)
  const appVersion = ref('...')
  const intentStats = ref<Record<string, IntentStat>>({})
  const modelStats = ref<Record<string, ModelStat>>({})
  const upstreamStats = ref<Record<string, UpstreamStat>>({})
  const lastRequest = ref<RequestLineage | null>(null)
  const requestHistory = ref<RequestLineage[]>([])

  const cacheHitRatio = computed(() => {
    const total = cacheHits.value + cacheMisses.value
    return total === 0 ? 0 : Math.round((cacheHits.value / total) * 100)
  })

  // ── SSE connection ─────────────────────────────────
  let es: EventSource | null = null
  let versionPollHandle: number | null = null

  async function fetchVersion() {
    try {
      const response = await fetch(VERSION_URL)
      if (!response.ok) return
      const payload: VersionInfo = await response.json()
      appVersion.value = payload.version || 'unknown'
    } catch {
      // Keep the last known version when the backend is temporarily unavailable.
    }
  }

  function applySnapshot(s: MetricsSnapshot) {
    rawTokens.value = s.raw_tokens
    compiledTokens.value = s.compiled_tokens
    memoryReusedTokens.value = s.memory_reused_tokens
    efficiencyScore.value = Math.round(s.efficiency_score)
    localRatio.value = Math.round(s.local_ratio)
    cloudRatio.value = 100 - Math.round(s.local_ratio)
    totalRequests.value = s.total_requests
    cacheHits.value = s.cache_hits
    cacheMisses.value = s.cache_misses
    cacheSavedTokens.value = s.cache_saved_tokens ?? 0
    historyRaw.value = s.history_raw
    historyCompiled.value = s.history_compiled
    historyReused.value = s.history_reused
    historyHourEpochs.value = s.history_hour_epochs ?? []
    historyHourRaw.value = s.history_hour_raw ?? []
    historyHourCompiled.value = s.history_hour_compiled ?? []
    historyHourReused.value = s.history_hour_reused ?? []
    routesLocal.value = s.routes_local
    routesCloud.value = s.routes_cloud
    routesMidtier.value = s.routes_midtier
    intentStats.value = s.intent_stats ?? {}
    modelStats.value = s.model_stats ?? {}
    upstreamStats.value = s.upstream_stats ?? {}
    lastRequest.value = s.last_request ?? null
    requestHistory.value = s.request_history ?? []
    lastTs.value = s.ts
  }

  function connect() {
    if (es) return
    es = new EventSource(SSE_URL)
    void fetchVersion()
    if (versionPollHandle === null) {
      versionPollHandle = window.setInterval(() => {
        void fetchVersion()
      }, 30000)
    }

    es.addEventListener('metrics', (ev) => {
      try {
        const snapshot: MetricsSnapshot = JSON.parse(ev.data)
        applySnapshot(snapshot)
        connected.value = true
      } catch {
        // ignore malformed events
      }
    })

    es.onerror = () => {
      connected.value = false
    }
  }

  function disconnect() {
    if (es) {
      es.close()
      es = null
      connected.value = false
    }
    if (versionPollHandle !== null) {
      window.clearInterval(versionPollHandle)
      versionPollHandle = null
    }
  }

  // Auto‑connect on store creation
  connect()

  return {
    rawTokens,
    compiledTokens,
    memoryReusedTokens,
    efficiencyScore,
    localRatio,
    cloudRatio,
    totalRequests,
    cacheHits,
    cacheMisses,
    cacheSavedTokens,
    cacheHitRatio,
    historyRaw,
    historyCompiled,
    historyReused,
    historyHourEpochs,
    historyHourRaw,
    historyHourCompiled,
    historyHourReused,
    routesLocal,
    routesCloud,
    routesMidtier,
    connected,
    lastTs,
    appVersion,
    intentStats,
    modelStats,
    upstreamStats,
    lastRequest,
    requestHistory,
    connect,
    disconnect,
  }
})
