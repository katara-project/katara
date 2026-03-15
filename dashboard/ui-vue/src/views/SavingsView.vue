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
      <div class="kpi-card accent" :class="{ pulse: pulsing }">
        <div class="kpi-value">{{ savingsData.tokensSaved.toLocaleString() }}</div>
        <div class="kpi-label">Tokens saved by compilation</div>
      </div>
      <div class="kpi-card sovereign" :class="{ pulse: pulsing }">
        <div class="kpi-value">{{ metrics.routesLocal }}</div>
        <div class="kpi-label">Requests routed on-prem</div>
      </div>
      <div class="kpi-card neutral" :class="{ pulse: pulsing }">
        <div class="kpi-value">{{ metrics.totalRequests }}</div>
        <div class="kpi-label">Total requests processed</div>
      </div>
      <div class="kpi-card neutral" :class="{ pulse: pulsing }">
        <div class="kpi-value">{{ avgSavedPerRequest }}</div>
        <div class="kpi-label">Avg tokens saved / request</div>
      </div>
    </div>

    <!-- Last Request Impact -->
    <section v-if="lastImpact" class="card last-impact-section" :class="{ pulse: pulsing }">
      <h3>Last Request Impact</h3>
      <div class="impact-grid">
        <div class="impact-tile">
          <span class="impact-value accent-text">{{ lastImpact.tokensSaved }}</span>
          <span class="impact-label">tokens saved</span>
        </div>
        <div class="impact-tile">
          <span class="impact-value accent-text">{{ lastImpact.costSaved }}</span>
          <span class="impact-label">cost avoided (USD)</span>
        </div>
        <div class="impact-tile">
          <span class="impact-value">{{ lastImpact.intent }}</span>
          <span class="impact-label">intent</span>
        </div>
        <div class="impact-tile">
          <span class="impact-value">{{ lastImpact.provider }}</span>
          <span class="impact-label">provider</span>
        </div>
      </div>
    </section>

    <!-- Savings bar + sparkline -->
    <div class="savings-bar-wrap card savings-bar-compact">
      <div class="savings-bar-header">
        <span class="savings-bar-title">Estimated Session Savings</span>
        <span class="savings-bar-amounts">
          {{ savingsData.tokensSaved.toLocaleString() }} tokens
          <span class="savings-bar-sep">&middot;</span>
          {{ formatCost(savingsData.costSaved) }}
        </span>
      </div>
      <div class="savings-bar-inline">
        <div class="savings-track">
          <div class="savings-fill" :style="{ width: savingsPct + '%' }"></div>
        </div>
        <SparklineChart v-if="savingsHistory.length > 1" :data="savingsHistory" color="var(--accent)" :height="20" />
      </div>
      <span class="savings-bar-note muted">Blended avg ${{ AVG_COST_PER_1K_TOKENS }}/1K tokens</span>
    </div>

    <!-- Session Projections -->
    <section v-if="metrics.totalRequests >= 3" class="card projections-section">
      <div class="section-heading">
        <div>
          <h3>Session Projections</h3>
          <p class="muted">Estimated impact if current request rate continues.</p>
        </div>
      </div>
      <div class="projections-grid">
        <div class="projection-tile">
          <span class="tile-label">Monthly Cost Saved (USD)</span>
          <strong class="projection-value accent-text">{{ formatCost(projections.monthlyCost) }}</strong>
          <span class="tile-subtitle">{{ projections.monthlyTokens.toLocaleString() }} tokens / month</span>
        </div>
        <div class="projection-tile">
          <span class="tile-label">Yearly Cost Saved (USD)</span>
          <strong class="projection-value accent-text">{{ formatCost(projections.yearlyCost) }}</strong>
          <span class="tile-subtitle">{{ projections.yearlyTokens.toLocaleString() }} tokens / year</span>
        </div>
        <div class="projection-tile">
          <span class="tile-label">Monthly CO&#x2082; Avoided</span>
          <strong class="projection-value primary-text">{{ formatCo2(projections.monthlyCo2Kg) }}</strong>
          <span class="tile-subtitle">{{ formatEnergy(projections.monthlyKwh) }} avoided</span>
        </div>
        <div class="projection-tile">
          <span class="tile-label">Yearly Tree Equivalent</span>
          <strong class="projection-value tree-text">&#x1F333; {{ formatTree(projections.yearlyTrees) }}</strong>
          <span class="tile-subtitle">1 tree &#x2248; 22 kg CO&#x2082;/year</span>
        </div>
      </div>
    </section>

    <!-- Environmental Impact tiles -->
    <section class="card savings-impact-section">
      <div class="section-heading">
        <div>
          <h3>Environmental Impact</h3>
          <p class="muted">Estimated environmental benefit from {{ savingsData.tokensSaved.toLocaleString() }} tokens avoided this session.</p>
        </div>
      </div>
      <div class="savings-grid">
        <div class="savings-tile cost-tile" :class="{ pulse: pulsing }">
          <span class="tile-label">Cost Saved (USD)</span>
          <strong class="savings-value">{{ formatCost(savingsData.costSaved) }}</strong>
          <span class="tile-subtitle">@ ${{ AVG_COST_PER_1K_TOKENS }} USD/1K tokens</span>
        </div>
        <div class="savings-tile energy-tile" :class="{ pulse: pulsing }">
          <span class="tile-label">Energy Avoided</span>
          <strong class="savings-value">{{ formatEnergy(savingsData.kwhAvoided) }}</strong>
          <span class="tile-subtitle">{{ formatCo2(savingsData.kgCo2Avoided) }} CO&#x2082; not emitted</span>
        </div>
        <div class="savings-tile tree-tile" :class="{ pulse: pulsing }">
          <span class="tile-label">CO&#x2082; Absorbed (tree equiv.)</span>
          <strong class="savings-value">&#x1F333; {{ formatTree(savingsData.treeFraction) }}</strong>
          <span class="tile-subtitle">1 tree absorbs ~22 kg CO&#x2082;/year</span>
        </div>
        <div class="savings-tile ice-tile" :class="{ pulse: pulsing }">
          <span class="tile-label">Ice Preserved</span>
          <strong class="savings-value">&#x1F9CA; {{ formatIce(savingsData.litresIceSaved) }}</strong>
          <span class="tile-subtitle">ice-melt equivalent avoided</span>
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
import { computed, ref, watch, onMounted, onUnmounted } from 'vue'
import { useMetricsStore } from '../store/metrics'
import SparklineChart from '../components/SparklineChart.vue'

const metrics = useMetricsStore()

// Savings constants
const AVG_COST_PER_1K_TOKENS = 0.006
const KWH_PER_1K_TOKENS = 0.0005
const KG_CO2_PER_KWH = 0.4
const KG_CO2_PER_TREE_YEAR = 22
const LITRES_ICE_PER_KG_CO2 = 5

// Pulse animation trigger — blinks green on any SSE update that changes totals
const pulsing = ref(false)
let pulseTimer: ReturnType<typeof setTimeout> | null = null

// Smart number formatting — dynamic precision, never scientific notation
function formatDynamic(v: number): string {
  if (v === 0) return '0'
  const abs = Math.abs(v)
  if (abs >= 100) return v.toFixed(1)
  if (abs >= 10) return v.toFixed(2)
  if (abs >= 1) return v.toFixed(3)
  if (abs >= 0.01) return v.toFixed(4)
  if (abs >= 0.0001) return v.toFixed(6)
  return v.toFixed(8)
}

// Unit-adaptive formatters for human-readable display
function formatCost(v: number): string {
  if (v === 0) return '$0.00'
  if (v >= 1) return '$' + v.toFixed(2)
  const cents = v * 100
  if (cents >= 1) return cents.toFixed(1) + '¢'
  if (cents >= 0.01) return cents.toFixed(2) + '¢'
  return '< 0.01¢'
}
function formatEnergy(kwh: number): string {
  if (kwh === 0) return '0 Wh'
  if (kwh >= 1) return kwh.toFixed(2) + ' kWh'
  return (kwh * 1000).toFixed(1) + ' Wh'
}
function formatCo2(kg: number): string {
  if (kg === 0) return '0 g'
  if (kg >= 1) return kg.toFixed(2) + ' kg'
  return (kg * 1000).toFixed(1) + ' g'
}
function formatTree(fraction: number): string {
  if (fraction === 0) return '0 g CO\u2082'
  if (fraction >= 1) return fraction.toFixed(1) + ' trees'
  // Convert to grams of CO₂ saved: fraction × 22 kg × 1000
  const gCo2 = fraction * KG_CO2_PER_TREE_YEAR * 1000
  if (gCo2 >= 1000) return (gCo2 / 1000).toFixed(1) + ' kg CO\u2082'
  if (gCo2 >= 1) return gCo2.toFixed(1) + ' g CO\u2082'
  if (gCo2 >= 0.01) return gCo2.toFixed(2) + ' g CO\u2082'
  return '< 0.01 g CO\u2082'
}
function formatIce(litres: number): string {
  if (litres === 0) return '0 mL'
  if (litres >= 1) return litres.toFixed(1) + ' L'
  return (litres * 1000).toFixed(0) + ' mL'
}

const savingsData = computed(() => {
  const tokensSaved = Math.max(0, metrics.rawTokens - metrics.compiledTokens) + metrics.cacheSavedTokens
  const costSaved = (tokensSaved / 1000) * AVG_COST_PER_1K_TOKENS
  const kwhAvoided = (tokensSaved / 1000) * KWH_PER_1K_TOKENS
  const kgCo2Avoided = kwhAvoided * KG_CO2_PER_KWH
  const treeFraction = kgCo2Avoided / KG_CO2_PER_TREE_YEAR
  const litresIceSaved = kgCo2Avoided * LITRES_ICE_PER_KG_CO2
  return { tokensSaved, costSaved, kwhAvoided, kgCo2Avoided, treeFraction, litresIceSaved }
})

// Pulse on every SSE tick that changes total requests
watch(() => metrics.totalRequests, (newVal, oldVal) => {
  if (newVal !== oldVal && newVal > 0) {
    pulsing.value = true
    if (pulseTimer) clearTimeout(pulseTimer)
    pulseTimer = setTimeout(() => { pulsing.value = false }, 700)
  }
})

const avgSavedPerRequest = computed(() => {
  if (!metrics.totalRequests) return '0'
  return Math.round(savingsData.value.tokensSaved / metrics.totalRequests).toLocaleString()
})

// Savings sparkline — derived from historyRaw - historyCompiled (cumulative savings over time)
const savingsHistory = computed(() => {
  const raw = metrics.historyRaw
  const compiled = metrics.historyCompiled
  if (!raw.length) return []
  return raw.map((r: number, i: number) => Math.max(0, r - (compiled[i] ?? r)))
})

const savingsPct = computed(() => {
  const target = 10000
  return Math.min(100, Math.round((savingsData.value.tokensSaved / target) * 100))
})

// Last request impact
const lastImpact = computed(() => {
  const lr = metrics.lastRequest
  if (!lr) return null
  const raw = lr.raw_tokens ?? 0
  const compiled = lr.compiled_tokens ?? 0
  const saved = Math.max(0, raw - compiled)
  return {
    tokensSaved: saved,
    costSaved: formatCost((saved / 1000) * AVG_COST_PER_1K_TOKENS),
    intent: lr.intent ?? 'unknown',
    provider: lr.routed_provider?.replace(/^ollama-|^openrouter-/, '') ?? '—',
  }
})

// Session projections (extrapolate to monthly / yearly at 500 req/day, 22 days/month)
const REQUESTS_PER_DAY = 500
const DAYS_PER_MONTH = 22
const MONTHS_PER_YEAR = 12

const projections = computed(() => {
  const total = metrics.totalRequests || 1
  const saved = savingsData.value.tokensSaved
  const perReq = saved / total
  const monthlyReqs = REQUESTS_PER_DAY * DAYS_PER_MONTH
  const yearlyReqs = monthlyReqs * MONTHS_PER_YEAR
  const monthlyTokens = Math.round(perReq * monthlyReqs)
  const yearlyTokens = Math.round(perReq * yearlyReqs)
  const monthlyCost = (monthlyTokens / 1000) * AVG_COST_PER_1K_TOKENS
  const yearlyCost = (yearlyTokens / 1000) * AVG_COST_PER_1K_TOKENS
  const monthlyKwh = (monthlyTokens / 1000) * KWH_PER_1K_TOKENS
  const monthlyCo2Kg = monthlyKwh * KG_CO2_PER_KWH
  const yearlyKwh = monthlyKwh * MONTHS_PER_YEAR
  const yearlyCo2Kg = yearlyKwh * KG_CO2_PER_KWH
  const yearlyTrees = yearlyCo2Kg / KG_CO2_PER_TREE_YEAR
  return { monthlyTokens, yearlyTokens, monthlyCost, yearlyCost, monthlyKwh, monthlyCo2Kg, yearlyTrees }
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
  if (pulseTimer) clearTimeout(pulseTimer)
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
/* Pulse animation */
@keyframes pulse-glow {
  0%   { box-shadow: 0 0 0 0 rgba(44, 255, 179, 0.4); }
  50%  { box-shadow: 0 0 12px 4px rgba(44, 255, 179, 0.25); }
  100% { box-shadow: 0 0 0 0 rgba(44, 255, 179, 0); }
}
.pulse { animation: pulse-glow 0.7s ease-out; }

/* Savings Bar */
.savings-bar-wrap { padding: 16px 20px; margin-bottom: 20px; }
.savings-bar-compact { padding: 10px 16px; margin-bottom: 14px; }
.savings-bar-header { display: flex; align-items: center; gap: 10px; margin-bottom: 6px; }
.savings-bar-title { font-weight: 600; font-size: 0.82rem; flex: 1; }
.savings-bar-amounts { font-size: 0.8rem; color: var(--muted); }
.savings-bar-sep { margin: 0 3px; }
.savings-bar-inline { display: flex; align-items: center; gap: 12px; }
.savings-bar-inline .savings-track { flex: 1; }
.savings-bar-note { font-size: 0.72rem; margin-top: 4px; display: block; }
.savings-track { height: 6px; border-radius: 4px; background: rgba(255, 255, 255, 0.06); overflow: hidden; }
.savings-fill { height: 100%; border-radius: 6px; background: var(--accent, #2cffb3); transition: width 0.5s ease; }

/* Last Request Impact */
.last-impact-section { margin-bottom: 20px; padding: 16px 20px; }
.last-impact-section h3 { margin: 0 0 12px; font-size: 1rem; }
.impact-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; }
.impact-tile { display: flex; flex-direction: column; gap: 4px; }
.impact-value { font-size: 1.2rem; font-weight: 700; letter-spacing: -0.3px; }
.impact-label { font-size: 0.75rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.5px; }
.accent-text { color: var(--accent, #2cffb3); }
.primary-text { color: var(--primary, #39d3ff); }
.tree-text { color: #66bb6a; }

/* Session Projections */
.projections-section { margin-bottom: 20px; }
.projections-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; margin-top: 14px; }
.projection-tile { display: flex; flex-direction: column; gap: 6px; padding: 16px; border-radius: 16px; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.08); }
.projection-value { font-size: 1.35rem; font-weight: 700; letter-spacing: -0.5px; transition: color 0.3s; }

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
.savings-kpis { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; margin-bottom: 20px; }
.kpi-card { background: var(--card-bg, rgba(255,255,255,0.04)); border: 1px solid rgba(255,255,255,0.08); border-radius: 16px; padding: 18px 20px; }
.kpi-card.accent { border-color: rgba(57, 211, 255, 0.25); }
.kpi-card.sovereign { border-color: rgba(44, 255, 179, 0.25); }
.kpi-card.neutral { border-color: rgba(255, 255, 255, 0.1); }
.kpi-value { font-size: 1.6rem; font-weight: 700; letter-spacing: -0.5px; transition: color 0.3s; }
.kpi-card.accent .kpi-value { color: var(--primary, #39d3ff); }
.kpi-card.sovereign .kpi-value { color: var(--accent, #2cffb3); }
.kpi-card.neutral .kpi-value { color: var(--foreground, #e0e0e0); }
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

@media (max-width: 1100px) { .savings-grid { grid-template-columns: repeat(2, 1fr); } .projections-grid { grid-template-columns: repeat(2, 1fr); } .savings-kpis { grid-template-columns: repeat(2, 1fr); } .impact-grid { grid-template-columns: repeat(2, 1fr); } }
@media (max-width: 640px) { .savings-grid { grid-template-columns: 1fr; } .cvr-grid { grid-template-columns: 1fr; } .projections-grid { grid-template-columns: 1fr; } .savings-kpis { grid-template-columns: 1fr; } .impact-grid { grid-template-columns: repeat(2, 1fr); } }
</style>
