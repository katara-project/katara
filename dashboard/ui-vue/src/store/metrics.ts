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
  semantic_cache_hit?: boolean
  semantic_fingerprint?: string
  cache_hit: boolean
  sensitive: boolean
  cost_usd?: number
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
  session_cost_usd?: number
  last_request_cost_usd?: number
}

const SSE_URL     = '/v1/metrics/stream'
const VERSION_URL = '/version'
const REST_URL    = '/v1/metrics'

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
  const sessionCostUsd = ref(0)
  const lastRequestCostUsd = ref(0)

  const cacheHitRatio = computed(() => {
    const total = cacheHits.value + cacheMisses.value
    return total === 0 ? 0 : Math.round((cacheHits.value / total) * 100)
  })

  // ── SSE connection ─────────────────────────────────
  let es: EventSource | null = null
  let versionPollHandle: number | null = null
  let watchdogHandle: number | null = null
  let restPollHandle: number | null = null   // guaranteed REST poll (always-on)
  let lastEventAt = 0   // wall-clock ms of last successfully parsed SSE event

  const WATCHDOG_INTERVAL_MS = 5_000   // check every 5 s
  const WATCHDOG_STALE_MS    = 10_000  // force reconnect if no event for 10 s

  // ── REST polling fallback ──────────────────────────────────────────────────
  // When the SSE stream is stale, pull a fresh snapshot from the REST endpoint
  // so the dashboard never shows frozen data while the watchdog reconnects.
  async function pollRest() {
    try {
      const res = await fetch(REST_URL)
      if (!res.ok) return
      const snapshot: MetricsSnapshot = await res.json()
      applySnapshot(snapshot)
    } catch {
      // ignore — backend may be momentarily unreachable
    }
  }

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
    sessionCostUsd.value = s.session_cost_usd ?? 0
    lastRequestCostUsd.value = s.last_request_cost_usd ?? 0
    lastTs.value = s.ts
  }

  function connect() {
    // If we already have a live (CONNECTING or OPEN) EventSource, do nothing.
    if (es && es.readyState !== EventSource.CLOSED) return
    // Clean up a stale CLOSED instance before creating a new one.
    if (es) { es.close(); es = null }

    lastEventAt = Date.now()   // reset so watchdog doesn't fire immediately
    es = new EventSource(SSE_URL)
    void pollRest()             // populate UI immediately; don't wait for first SSE event
    void fetchVersion()

    // ── Guaranteed REST poll (always-on) ────────────────────────────────────
    // Runs every 5 s regardless of SSE state — ensures the dashboard always
    // shows live data even when EventSource is silently stuck.
    if (restPollHandle === null) {
      restPollHandle = window.setInterval(() => { void pollRest() }, 5_000)
    }
    if (versionPollHandle === null) {
      versionPollHandle = window.setInterval(() => {
        void fetchVersion()
      }, 30000)
    }

    // ── Watchdog: force reconnect when the stream goes silently stale ────────
    // Covers the case where the browser's EventSource is stuck in CONNECTING
    // with exponential backoff (readyState !== CLOSED so onerror guard misses it),
    // or when a proxy silently drops the connection without sending an error.
    if (watchdogHandle === null) {
      watchdogHandle = window.setInterval(() => {
        const stale = Date.now() - lastEventAt > WATCHDOG_STALE_MS
        if (stale) {
          connected.value = false
          void pollRest()   // pull fresh data via REST while SSE reconnects
          if (es) { es.close(); es = null }
          connect()
        }
      }, WATCHDOG_INTERVAL_MS)
    }

    es.onopen = () => {
      connected.value = true
      lastEventAt = Date.now()
    }

    // Named event emitted by Axum's Sse::new().  Some proxies strip the
    // `event:` line and deliver a generic `message` event instead — we listen
    // for both so the dashboard works transparently behind any proxy.
    function handleSseEvent(ev: MessageEvent) {
      try {
        const snapshot: MetricsSnapshot = JSON.parse(ev.data)
        applySnapshot(snapshot)
        connected.value = true
        lastEventAt = Date.now()
      } catch {
        // ignore malformed events
      }
    }
    es.addEventListener('metrics', handleSseEvent)
    es.addEventListener('message', handleSseEvent)

    es.onerror = () => {
      connected.value = false
      // When the browser gives up retrying (CLOSED state), schedule a manual reconnect.
      if (es?.readyState === EventSource.CLOSED) {
        es = null
        window.setTimeout(connect, 3000)
      }
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
    if (watchdogHandle !== null) {
      window.clearInterval(watchdogHandle)
      watchdogHandle = null
    }
    if (restPollHandle !== null) {
      window.clearInterval(restPollHandle)
      restPollHandle = null
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
    sessionCostUsd,
    lastRequestCostUsd,
    connect,
    disconnect,
  }
})
