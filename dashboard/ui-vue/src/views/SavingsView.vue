<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Savings &amp; Impact</h2>
        <p class="muted">Economic and environmental benefit of AI context optimisation by Distira.</p>
      </div>
    </header>

    <!-- KPI cards -->
    <div class="savings-kpis">
      <div class="kpi-card accent">
        <div class="kpi-value">{{ savingsData.tokensSaved.toLocaleString() }}</div>
        <div class="kpi-label">Tokens saved by compilation</div>
      </div>
      <div class="kpi-card sovereign">
        <div class="kpi-value">{{ metrics.routesLocal }}</div>
        <div class="kpi-label">Requests routed on-prem</div>
      </div>
    </div>

    <!-- Savings bar -->
    <div class="savings-bar-wrap card">
      <div class="savings-bar-header">
        <span class="savings-bar-title">Estimated Session Savings</span>
        <span class="savings-bar-amounts">
          {{ savingsData.tokensSaved.toLocaleString() }} tokens saved
          <span class="savings-bar-sep">&middot;</span>
          ${{ savingsData.costSaved.toFixed(4) }}
        </span>
      </div>
      <div class="savings-track">
        <div class="savings-fill" :style="{ width: savingsPct + '%' }"></div>
      </div>
      <span class="savings-bar-note muted">Based on blended avg ${{ AVG_COST_PER_1K_TOKENS }}/1K tokens across major LLM providers</span>
    </div>

    <!-- Environmental Impact tiles -->
    <section class="card savings-impact-section">
      <div class="section-heading">
        <div>
          <h3>Environmental Impact</h3>
          <p class="muted">Estimated environmental benefit from {{ savingsData.tokensSaved.toLocaleString() }} tokens avoided.</p>
        </div>
      </div>
      <div class="savings-grid">
        <div class="savings-tile cost-tile">
          <span class="tile-label">Cost Saved</span>
          <strong class="savings-value">${{ savingsData.costSaved.toFixed(4) }}</strong>
          <span class="tile-subtitle">@ ${{ AVG_COST_PER_1K_TOKENS }}/1K tokens</span>
        </div>
        <div class="savings-tile energy-tile">
          <span class="tile-label">Energy Avoided</span>
          <strong class="savings-value">{{ savingsData.kwhAvoided.toFixed(4) }} kWh</strong>
          <span class="tile-subtitle">{{ savingsData.kgCo2Avoided.toFixed(4) }} kg CO&#x2082; not emitted</span>
        </div>
        <div class="savings-tile tree-tile">
          <span class="tile-label">Tree Equivalent</span>
          <strong class="savings-value">&#x1F333; {{ savingsData.treeFraction.toFixed(4) }}</strong>
          <span class="tile-subtitle">of a tree's yearly CO&#x2082; absorption</span>
        </div>
        <div class="savings-tile ice-tile">
          <span class="tile-label">Ice Preserved</span>
          <strong class="savings-value">&#x1F9CA; {{ savingsData.litresIceSaved.toFixed(1) }} L</strong>
          <span class="tile-subtitle">of ice-melt equivalent avoided</span>
        </div>
      </div>
    </section>

    <!-- Codegen vs Review -->
    <section class="card codegen-vs-review-section">
      <div class="section-heading">
        <div>
          <h3>Codegen vs Review</h3>
          <p class="muted">How much traffic is pure code generation vs review/refactor, and how efficiently Distira trims each intent.</p>
        </div>
        <span class="total-badge" v-if="codegenReview.hasData">{{ codegenReview.totalRequests }} req</span>
      </div>
      <div v-if="codegenReview.hasData" class="cvr-grid">
        <div class="cvr-tile">
          <span class="tile-label">Codegen</span>
          <strong class="cvr-count">{{ codegenReview.codegenRequests }}</strong>
          <span class="tile-subtitle">{{ codegenReview.codegenPct }}% of intents</span>
          <span class="cvr-reduction">{{ codegenReview.codegenReduction }}% avg reduction</span>
        </div>
        <div class="cvr-tile">
          <span class="tile-label">Review</span>
          <strong class="cvr-count">{{ codegenReview.reviewRequests }}</strong>
          <span class="tile-subtitle">{{ codegenReview.reviewPct }}% of intents</span>
          <span class="cvr-reduction">{{ codegenReview.reviewReduction }}% avg reduction</span>
        </div>
      </div>
      <p v-else class="muted">No codegen or review traffic yet.</p>
    </section>

    <!-- Intent Distribution -->
    <section class="card intent-distribution-section">
      <div class="section-heading">
        <div>
          <h3>Intent Distribution</h3>
          <p class="muted">How Distira classified incoming requests.</p>
        </div>
        <span class="total-badge">{{ metrics.totalRequests }} total</span>
      </div>
      <div v-if="intentRows.length" class="intent-grid">
        <div v-for="entry in intentRows" :key="entry.intent" class="intent-tile" :class="'intent-' + entry.intent">
          <span class="intent-label">{{ entry.intent }}</span>
          <strong class="intent-count">{{ entry.requests }}</strong>
          <span class="intent-pct">{{ entry.pct }}%</span>
          <div class="intent-bar-track">
            <div class="intent-bar-fill" :style="{ width: entry.pct + '%' }"></div>
          </div>
          <span class="intent-reduction">{{ entry.avgReduction }}% reduction</span>
        </div>
      </div>
      <p v-else class="muted">No intent data yet.</p>
    </section>

    <!-- Optimization Suggestions -->
    <section class="card suggestions-section">
      <div class="section-heading">
        <div>
          <h3>Optimization Suggestions</h3>
          <p class="muted">Active routing map, session efficiency, and reactive alerts &mdash; refreshes every 30 s.</p>
        </div>
        <button class="refresh-btn" @click="fetchSuggestions" :disabled="loadingSuggestions">
          {{ loadingSuggestions ? 'Loading\u2026' : 'Refresh' }}
        </button>
      </div>
      <div v-if="suggestions.length" class="suggestions-list">
        <div v-for="(s, idx) in suggestions" :key="idx" class="suggestion-item" :class="s.severity">
          <span class="suggestion-icon">{{ suggestionIcon(s) }}</span>
          <div class="suggestion-body">
            <span class="suggestion-message">{{ s.message }}</span>
            <span class="suggestion-meta" v-if="s.provider">{{ s.provider }} &middot; {{ s.metric }}: {{ s.value }}</span>
            <span class="suggestion-meta" v-else>{{ s.metric }}: {{ s.value }}</span>
          </div>
        </div>
      </div>
      <div v-else-if="loadingSuggestions" class="suggestions-loading">
        <span class="loading-spinner"></span> Loading suggestions&hellip;
      </div>
      <p v-else class="muted">No suggestions available yet.</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useMetricsStore } from '../store/metrics'

const metrics = useMetricsStore()

// Savings constants
const AVG_COST_PER_1K_TOKENS = 0.006
const KWH_PER_1K_TOKENS = 0.0005
const KG_CO2_PER_KWH = 0.4
const KG_CO2_PER_TREE_YEAR = 22
const LITRES_ICE_PER_KG_CO2 = 5

const savingsData = computed(() => {
  const tokensSaved = Math.max(0, metrics.rawTokens - metrics.compiledTokens) + metrics.cacheSavedTokens
  const costSaved = (tokensSaved / 1000) * AVG_COST_PER_1K_TOKENS
  const kwhAvoided = (tokensSaved / 1000) * KWH_PER_1K_TOKENS
  const kgCo2Avoided = kwhAvoided * KG_CO2_PER_KWH
  const treeFraction = kgCo2Avoided / KG_CO2_PER_TREE_YEAR
  const litresIceSaved = kgCo2Avoided * LITRES_ICE_PER_KG_CO2
  return { tokensSaved, costSaved, kwhAvoided, kgCo2Avoided, treeFraction, litresIceSaved }
})

const savingsPct = computed(() => {
  const target = 10000
  return Math.min(100, Math.round((savingsData.value.tokensSaved / target) * 100))
})

// Suggestions
interface Suggestion {
  severity: 'warning' | 'info'
  code: string
  provider: string
  metric: string
  value: number
  message: string
}
const suggestions = ref<Suggestion[]>([])
const loadingSuggestions = ref(false)

async function fetchSuggestions() {
  loadingSuggestions.value = true
  try {
    const res = await fetch('/v1/suggestions')
    if (res.ok) {
      const data = await res.json()
      suggestions.value = data.suggestions ?? []
    }
  } catch {
    // Server not yet available
  } finally {
    loadingSuggestions.value = false
  }
}

const SUGGESTION_ICONS: Record<string, string> = {
  routing_active: '\u21CC',
  session_efficiency: '\u26A1',
  cache_performance: '\u25C8',
  concise_mode_active: '\u2726',
  high_error_rate: '\u26A0',
  high_latency: '\u2139',
}
function suggestionIcon(s: Suggestion): string {
  if (s.code in SUGGESTION_ICONS) return SUGGESTION_ICONS[s.code]
  return s.severity === 'warning' ? '\u26A0' : '\u2139'
}

let suggestionTimer: ReturnType<typeof setInterval> | null = null
onMounted(() => {
  fetchSuggestions()
  suggestionTimer = setInterval(fetchSuggestions, 30_000)
})
onUnmounted(() => {
  if (suggestionTimer !== null) clearInterval(suggestionTimer)
})

// Codegen vs Review
const codegenReview = computed(() => {
  const stats = metrics.intentStats as Record<string, { requests: number; raw_tokens: number; compiled_tokens: number }>
  const codegen = stats['codegen']
  const review = stats['review']
  if (!codegen && !review) {
    return { hasData: false, totalRequests: 0, codegenRequests: 0, reviewRequests: 0, codegenPct: 0, reviewPct: 0, codegenReduction: 0, reviewReduction: 0 }
  }
  const totalRequests = (codegen?.requests ?? 0) + (review?.requests ?? 0)
  const safeTotal = totalRequests || 1
  const calcReduction = (entry: { raw_tokens: number; compiled_tokens: number } | undefined) => {
    if (!entry || !entry.raw_tokens) return 0
    return Math.round((Math.max(0, entry.raw_tokens - entry.compiled_tokens) / entry.raw_tokens) * 100)
  }
  return {
    hasData: true,
    totalRequests,
    codegenRequests: codegen?.requests ?? 0,
    reviewRequests: review?.requests ?? 0,
    codegenPct: Math.round(((codegen?.requests ?? 0) / safeTotal) * 100),
    reviewPct: Math.round(((review?.requests ?? 0) / safeTotal) * 100),
    codegenReduction: calcReduction(codegen),
    reviewReduction: calcReduction(review),
  }
})

// Intent Distribution
const intentRows = computed(() => {
  const stats = metrics.intentStats as Record<string, { requests: number; raw_tokens: number; compiled_tokens: number }>
  const total = Object.values(stats).reduce((s, v) => s + v.requests, 0) || 1
  return Object.entries(stats)
    .map(([intent, stat]) => {
      const saved = Math.max(0, stat.raw_tokens - stat.compiled_tokens)
      const avgReduction = stat.raw_tokens > 0 ? Math.round((saved / stat.raw_tokens) * 100) : 0
      return { intent, requests: stat.requests, pct: Math.round((stat.requests / total) * 100), avgReduction }
    })
    .sort((a, b) => b.requests - a.requests)
})
</script>

<style scoped>
/* Savings Bar */
.savings-bar-wrap { padding: 16px 20px; margin-bottom: 20px; }
.savings-bar-header { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
.savings-bar-title { font-weight: 600; font-size: 0.9rem; flex: 1; }
.savings-bar-amounts { font-size: 0.88rem; color: var(--muted); }
.savings-bar-sep { margin: 0 4px; }
.savings-bar-note { font-size: 0.78rem; margin-top: 6px; display: block; }
.savings-track { height: 8px; border-radius: 6px; background: rgba(255, 255, 255, 0.06); overflow: hidden; }
.savings-fill { height: 100%; border-radius: 6px; background: var(--accent, #2cffb3); transition: width 0.5s ease; }

/* Savings Impact */
.savings-impact-section { margin-top: 20px; margin-bottom: 20px; }
.savings-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 14px; margin-top: 14px; }
.savings-tile { display: flex; flex-direction: column; gap: 6px; padding: 16px; border-radius: 16px; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.08); }
.savings-tile.cost-tile  { border-color: rgba(44, 255, 179, 0.22); }
.savings-tile.energy-tile { border-color: rgba(57, 211, 255, 0.22); }
.savings-tile.tree-tile  { border-color: rgba(76, 175, 80, 0.28); }
.savings-tile.ice-tile   { border-color: rgba(120, 200, 255, 0.28); }
.savings-value { font-size: 1.35rem; font-weight: 700; letter-spacing: -0.5px; transition: color 0.3s, transform 0.3s; }
.cost-tile .savings-value  { color: var(--accent, #2cffb3); }
.energy-tile .savings-value { color: var(--primary, #39d3ff); }
.tree-tile .savings-value  { color: #66bb6a; }
.ice-tile .savings-value   { color: #78c8ff; }
/* KPI Cards */
.savings-kpis { display: grid; grid-template-columns: repeat(2, 1fr); gap: 14px; margin-bottom: 20px; }
.kpi-card { background: var(--card-bg, rgba(255,255,255,0.04)); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px; padding: 18px 20px; }
.kpi-card.accent { border-color: rgba(57, 211, 255, 0.25); }
.kpi-card.sovereign { border-color: rgba(44, 255, 179, 0.25); }
.kpi-value { font-size: 1.6rem; font-weight: 700; letter-spacing: -0.5px; transition: color 0.3s; }
.kpi-card.accent .kpi-value { color: var(--primary, #39d3ff); }
.kpi-card.sovereign .kpi-value { color: var(--accent, #2cffb3); }
.kpi-label { font-size: 0.78rem; color: var(--muted); margin-top: 4px; }

/* Codegen vs Review */
.codegen-vs-review-section { margin-top: 20px; }
.codegen-vs-review-section .section-heading h3 { margin: 0 0 6px; font-size: 1rem; }
.cvr-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; margin-top: 14px; }
.cvr-tile { display: flex; flex-direction: column; gap: 6px; padding: 14px 16px; border-radius: 16px; border: 1px solid rgba(255, 255, 255, 0.08); background: linear-gradient(135deg, rgba(0, 214, 143, 0.12), rgba(40, 120, 255, 0.08)); }
.cvr-count { font-size: 1.6rem; font-weight: 700; line-height: 1; }
.cvr-reduction { font-size: 0.8rem; color: var(--accent); }

/* Intent Distribution */
.intent-distribution-section { margin-top: 20px; }
.intent-distribution-section .section-heading { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; }
.total-badge { flex-shrink: 0; padding: 4px 12px; border-radius: 999px; background: rgba(255, 255, 255, 0.07); color: var(--muted); font-size: 0.8rem; font-weight: 600; }
.intent-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 12px; margin-top: 14px; }
.intent-tile { display: flex; flex-direction: column; gap: 4px; padding: 14px 16px; border-radius: 16px; border: 1px solid rgba(255, 255, 255, 0.08); background: rgba(255, 255, 255, 0.03); }
.intent-tile.intent-debug  { border-color: rgba(255, 104, 104, 0.3); background: linear-gradient(135deg, rgba(255, 104, 104, 0.1), rgba(255, 169, 64, 0.06)); }
.intent-tile.intent-review { border-color: rgba(96, 156, 255, 0.3);  background: linear-gradient(135deg, rgba(96, 156, 255, 0.1), rgba(0, 214, 143, 0.06)); }
.intent-tile.intent-summarize { border-color: rgba(0, 214, 143, 0.3); background: linear-gradient(135deg, rgba(0, 214, 143, 0.1), rgba(40, 120, 255, 0.06)); }
.intent-tile.intent-ocr    { border-color: rgba(187, 134, 252, 0.3); background: linear-gradient(135deg, rgba(187, 134, 252, 0.1), rgba(96, 156, 255, 0.06)); }
.intent-tile.intent-general { border-color: rgba(255, 255, 255, 0.1); }
.intent-tile.intent-codegen { border-color: rgba(44, 255, 179, 0.3); background: linear-gradient(135deg, rgba(44, 255, 179, 0.1), rgba(40, 120, 255, 0.06)); }
.intent-label { font-size: 0.75rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.5px; color: var(--muted); }
.intent-count { font-size: 1.6rem; font-weight: 700; line-height: 1; }
.intent-pct { font-size: 0.82rem; color: var(--muted); }
.intent-bar-track { height: 4px; border-radius: 999px; background: rgba(255, 255, 255, 0.08); margin: 4px 0 2px; overflow: hidden; }
.intent-bar-fill { height: 100%; border-radius: 999px; background: var(--primary); transition: width 0.4s ease; }
.intent-reduction { font-size: 0.77rem; color: var(--accent); }

/* Suggestions */
.suggestions-section .section-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 14px; }
.refresh-btn { padding: 5px 14px; border-radius: 8px; border: 1px solid rgba(255, 255, 255, 0.12); background: rgba(255, 255, 255, 0.05); color: var(--muted); font-size: 0.82rem; cursor: pointer; flex-shrink: 0; transition: background 0.2s; }
.refresh-btn:hover { background: rgba(255, 255, 255, 0.10); }
.refresh-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.suggestions-list { display: flex; flex-direction: column; gap: 8px; }
.suggestion-item { display: flex; align-items: flex-start; gap: 10px; padding: 10px 14px; border-radius: 10px; border: 1px solid; font-size: 0.87rem; }
.suggestion-item.warning { background: rgba(255, 169, 64, 0.08); border-color: rgba(255, 169, 64, 0.28); }
.suggestion-item.info { background: rgba(40, 120, 255, 0.07); border-color: rgba(40, 120, 255, 0.22); }
.suggestion-icon { font-size: 1rem; flex-shrink: 0; margin-top: 1px; }
.suggestion-body { display: flex; flex-direction: column; gap: 3px; }
.suggestion-message { color: var(--foreground); line-height: 1.4; }
.suggestion-meta { font-size: 0.78rem; color: var(--muted); font-family: monospace; }
.suggestions-loading { display: flex; align-items: center; gap: 8px; color: var(--muted); font-size: 0.87rem; }
.loading-spinner { display: inline-block; width: 14px; height: 14px; border: 2px solid rgba(255,255,255,0.15); border-top-color: var(--primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

/* Shared */
.section-heading { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; }

@media (max-width: 1100px) { .savings-grid { grid-template-columns: repeat(2, 1fr); } }
@media (max-width: 640px) { .savings-grid { grid-template-columns: 1fr; } .cvr-grid { grid-template-columns: 1fr; } }
</style>
