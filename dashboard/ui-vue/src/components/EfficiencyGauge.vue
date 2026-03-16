<template>
  <section class="card gauge-card">
    <h2>AI Efficiency Score</h2>

    <div class="gauge-wrap">
      <svg viewBox="0 0 200 118" class="gauge-svg" aria-hidden="true">
        <!-- track arc: centre (100,108) r=84, left to right over top, sweep=1 -->
        <path class="gauge-track" d="M 16 108 A 84 84 0 0 1 184 108" />
        <path
          class="gauge-fill"
          d="M 16 108 A 84 84 0 0 1 184 108"
          :stroke="arcColor"
          :stroke-dasharray="ARC_LEN"
          :stroke-dashoffset="arcOffset"
        />
        <text x="100" y="88" class="svg-score" text-anchor="middle" dominant-baseline="auto">{{ displayScore }}<tspan class="svg-unit">%</tspan></text>
        <text x="100" y="110" class="svg-band" text-anchor="middle" dominant-baseline="auto" :class="bandClass">{{ bandLabel }}</text>
      </svg>
    </div>

    <div class="sub-metrics">
      <div class="sub-row">
        <span class="sub-label">Token compression</span>
        <div class="sub-bar-track">
          <div class="sub-bar-fill sub-accent" :style="{ width: tokenCompressionPct + '%' }"></div>
        </div>
        <span class="sub-pct">{{ tokenCompressionPct }}%</span>
      </div>
      <div class="sub-row">
        <span class="sub-label">Memory reuse</span>
        <div class="sub-bar-track">
          <div class="sub-bar-fill sub-secondary" :style="{ width: memoryReusePct + '%' }"></div>
        </div>
        <span class="sub-pct">{{ memoryReusePct }}%</span>
      </div>
      <div class="sub-row">
        <span class="sub-label">On-prem routing</span>
        <div class="sub-bar-track">
          <div class="sub-bar-fill sub-primary" :style="{ width: localRatioPct + '%' }"></div>
        </div>
        <span class="sub-pct">{{ localRatioPct }}%</span>
      </div>
    </div>

    <div class="gauge-footer">
      <div class="footer-section">
        <span class="footer-title">How the score is calculated</span>
        <div class="footer-row"><span class="footer-dot dot-accent"></span><span>Token compression — tokens saved vs raw input</span></div>
        <div class="footer-row"><span class="footer-dot dot-secondary"></span><span>Memory reuse — stable context blocks reused across requests</span></div>
        <div class="footer-row"><span class="footer-dot dot-primary"></span><span>Sovereign bonus — +30 pts for routing through DISTIRA</span></div>
      </div>
      <div class="footer-section">
        <span class="footer-title">Score scale</span>
        <div class="scale-grid">
          <span class="score-badge badge-low">0–49</span><span class="scale-label">Low</span><span class="scale-desc">— little compression or reuse yet</span>
          <span class="score-badge badge-mid">50–74</span><span class="scale-label">High</span><span class="scale-desc">— DISTIRA is actively optimising</span>
          <span class="score-badge badge-top">75–100</span><span class="scale-label">Excellent</span><span class="scale-desc">— heavy compression and context reuse</span>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useMetricsStore } from '../store/metrics'

const props = defineProps<{ score: number; totalRequests?: number }>()
const metrics = useMetricsStore()

const ARC_LEN = Math.PI * 84

const displayScore = ref(0)
let animationTimer: ReturnType<typeof setInterval> | null = null

watch(
  () => props.score,
  (next) => {
    if (animationTimer) { clearInterval(animationTimer); animationTimer = null }
    const target = Math.min(100, Math.max(0, Math.round(next)))
    const direction = target >= displayScore.value ? 1 : -1
    const step = Math.max(1, Math.ceil(Math.abs(target - displayScore.value) / 20))
    animationTimer = setInterval(() => {
      const candidate = displayScore.value + direction * step
      const reached = direction > 0 ? candidate >= target : candidate <= target
      displayScore.value = reached ? target : candidate
      if (reached && animationTimer) { clearInterval(animationTimer); animationTimer = null }
    }, 25)
  },
  { immediate: true }
)

// offset 0 = full arc, offset ARC_LEN = empty
const arcOffset = computed(() => ARC_LEN * (1 - displayScore.value / 100))
const arcColor  = computed(() => {
  const s = displayScore.value
  return s >= 75 ? 'var(--accent)' : s >= 50 ? 'var(--primary)' : '#ffa940'
})

const isWarming = computed(() => (props.totalRequests ?? 0) < 5)
const bandLabel = computed(() => {
  if (isWarming.value) return 'Calibrating'
  const s = displayScore.value
  return s >= 75 ? 'Excellent' : s >= 50 ? 'High' : 'Efficient'
})
const bandClass = computed(() => ({
  'band-warming':   isWarming.value,
  'band-excellent': !isWarming.value && displayScore.value >= 75,
  'band-high':      !isWarming.value && displayScore.value >= 50,
  'band-low':       !isWarming.value && displayScore.value < 50,
}))

const tokenCompressionPct = computed(() => {
  if (!metrics.rawTokens) return 0
  return Math.min(100, Math.round((metrics.rawTokens - metrics.compiledTokens) / metrics.rawTokens * 100))
})
const memoryReusePct = computed(() => {
  if (!metrics.rawTokens) return 0
  return Math.min(100, Math.round(metrics.memoryReusedTokens / metrics.rawTokens * 100))
})
const localRatioPct = computed(() => Math.min(100, Math.round(metrics.localRatio)))
</script>

<style scoped>
.gauge-card h2 { margin: 0 0 8px; }

.gauge-wrap {
  display: flex;
  justify-content: center;
  padding: 4px 0 0;
}
.gauge-svg {
  width: 100%;
  max-width: 210px;
}

.gauge-track {
  fill: none;
  stroke: rgba(255, 255, 255, 0.07);
  stroke-width: 14;
  stroke-linecap: round;
}
.gauge-fill {
  fill: none;
  stroke-width: 14;
  stroke-linecap: round;
  transition: stroke-dashoffset 0.5s ease-out, stroke 0.4s ease;
}

.svg-score {
  font-size: 34px;
  font-weight: 700;
  fill: #fff;
  font-family: inherit;
}
.svg-unit {
  font-size: 16px;
  fill: rgba(255,255,255,0.5);
}
.svg-band {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  font-family: inherit;
}
.band-warming   { fill: rgba(255,255,255,0.4); }
.band-excellent { fill: var(--accent); }
.band-high      { fill: var(--primary); }
.band-low       { fill: #ffa940; }

.sub-metrics {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 14px;
}
.sub-row {
  display: grid;
  grid-template-columns: 130px 1fr 36px;
  align-items: center;
  gap: 8px;
}
.sub-label {
  font-size: 0.78rem;
  color: var(--muted);
  white-space: nowrap;
}
.sub-bar-track {
  height: 4px;
  border-radius: 99px;
  background: rgba(255,255,255,0.06);
  overflow: hidden;
}
.sub-bar-fill {
  height: 100%;
  border-radius: 99px;
  transition: width 0.5s ease-out;
}
.sub-accent    { background: var(--accent); }
.sub-secondary { background: var(--secondary); }
.sub-primary   { background: var(--primary); }
.sub-pct {
  font-size: 0.75rem;
  color: var(--muted);
  text-align: right;
}
.gauge-footer {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid rgba(255,255,255,0.06);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.footer-section {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.footer-title {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: rgba(255,255,255,0.35);
  margin-bottom: 2px;
}
.footer-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
  color: var(--muted);
  line-height: 1.4;
}
.footer-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot-accent    { background: var(--accent); }
.dot-secondary { background: var(--secondary); }
.dot-primary   { background: var(--primary); }
.score-badge {
  font-size: 0.68rem;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  flex-shrink: 0;
  min-width: 38px;
  text-align: center;
}
.badge-low { background: rgba(255,169,64,0.15); color: #ffa940; }
.badge-mid { background: rgba(99,102,241,0.15); color: var(--primary); }
.badge-top { background: rgba(52,211,153,0.15); color: var(--accent); }
.scale-grid {
  display: grid;
  grid-template-columns: 50px auto 1fr;
  gap: 4px 8px;
  align-items: center;
  font-size: 0.75rem;
  color: var(--muted);
  line-height: 1.4;
}
.scale-label {
  font-weight: 600;
  color: rgba(255,255,255,0.65);
  white-space: nowrap;
}
.scale-desc {
  color: var(--muted);
}
</style>
