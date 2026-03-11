import { defineStore } from 'pinia'
import { ref, computed, onUnmounted } from 'vue'

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
  history_raw: number[]
  history_compiled: number[]
  history_reused: number[]
  routes_local: number
  routes_cloud: number
  routes_midtier: number
}

const SSE_URL = 'http://localhost:8080/v1/metrics/stream'

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
  const historyRaw = ref<number[]>([])
  const historyCompiled = ref<number[]>([])
  const historyReused = ref<number[]>([])
  const routesLocal = ref(0)
  const routesCloud = ref(0)
  const routesMidtier = ref(0)
  const connected = ref(false)
  const lastTs = ref(0)

  const cacheHitRatio = computed(() => {
    const total = cacheHits.value + cacheMisses.value
    return total === 0 ? 0 : Math.round((cacheHits.value / total) * 100)
  })

  // ── SSE connection ─────────────────────────────────
  let es: EventSource | null = null

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
    historyRaw.value = s.history_raw
    historyCompiled.value = s.history_compiled
    historyReused.value = s.history_reused
    routesLocal.value = s.routes_local
    routesCloud.value = s.routes_cloud
    routesMidtier.value = s.routes_midtier
    lastTs.value = s.ts
  }

  function connect() {
    if (es) return
    es = new EventSource(SSE_URL)

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
    cacheHitRatio,
    historyRaw,
    historyCompiled,
    historyReused,
    routesLocal,
    routesCloud,
    routesMidtier,
    connected,
    lastTs,
    connect,
    disconnect,
  }
})
