<template>
  <section class="card gauge-card">
    <div class="gauge-header">
      <h2>AI Efficiency Score</h2>
      <button type="button" class="help-icon" @click="showHelp = !showHelp" aria-label="Aide sur l'AI Efficiency Score">
        ?
      </button>
    </div>
    <div class="gauge-wrap">
      <div class="gauge-ring" :style="gaugeStyle">
        <span class="gauge-inner">
          <span class="gauge-value">{{ displayScore }}</span>
          <span class="gauge-unit">%</span>
          <span class="gauge-band">{{ bandLabel }}</span>
        </span>
      </div>
    </div>
    <div class="gauge-legend">
      <div class="legend-item">
        <span class="legend-dot" style="background: var(--accent)"></span>
        Token avoidance
      </div>
      <div class="legend-item">
        <span class="legend-dot" style="background: var(--secondary)"></span>
        Memory reuse
      </div>
      <div class="legend-item">
        <span class="legend-dot" style="background: var(--primary)"></span>
        Routing quality
      </div>
    </div>
    <p class="gauge-helper muted">
      Entre 30 et 50 %, Katara est déjà très efficace. 100 % reste un plafond théorique.
    </p>
    <div v-if="showHelp" class="gauge-help-panel">
      <p><strong>Comment lire ce score&nbsp;?</strong></p>
      <p>0&nbsp;% signifie qu'aucun token n'est évité par rapport à l'envoi brut.</p>
      <p>Autour de 30–50&nbsp;%, Katara réutilise et réduit déjà fortement le contexte.</p>
      <p>100&nbsp;% est théorique&nbsp;: dans la pratique, un bon objectif est de rester au‑dessus de 30&nbsp;% sur vos gros scénarios.</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

const props = defineProps<{ score: number }>()

const displayScore = ref(0)
let animationTimer: ReturnType<typeof setInterval> | null = null

const showHelp = ref(false)

watch(
  () => props.score,
  (next) => {
    if (animationTimer) {
      clearInterval(animationTimer)
      animationTimer = null
    }
    const target = Math.max(0, Math.round(next))
    const direction = target >= displayScore.value ? 1 : -1
    const delta = Math.abs(target - displayScore.value)
    const step = Math.max(1, Math.ceil(delta / 20))

    animationTimer = setInterval(() => {
      const candidate = displayScore.value + direction * step
      const reached = direction > 0 ? candidate >= target : candidate <= target
      displayScore.value = reached ? target : candidate
      if (reached && animationTimer) {
        clearInterval(animationTimer)
        animationTimer = null
      }
    }, 25)
  },
  { immediate: true }
)

const gaugeStyle = computed(() => {
  const pct = displayScore.value
  const color = pct >= 60 ? 'var(--accent)' : pct >= 30 ? 'var(--primary)' : '#ffa940'
  return {
    background: `conic-gradient(${color} 0 ${pct}%, rgba(255,255,255,0.06) ${pct}% 100%)`,
  }
})

const bandLabel = computed(() => {
  const s = displayScore.value
  if (s < 15) return 'À optimiser'
  if (s < 30) return 'Correct'
  return 'Efficace'
})
</script>

<style scoped>
.gauge-card h2 { margin: 0 0 8px; }
.gauge-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.help-icon {
  border: none;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 999px;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.9rem;
  color: var(--muted);
  cursor: pointer;
  padding: 0;
}
.help-icon:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
}
.gauge-wrap { display: flex; align-items: center; justify-content: center; padding: 20px 0 16px; }
.gauge-ring {
  width: 180px;
  height: 180px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  transition: background 0.3s ease-out;
}
.gauge-inner {
  width: 140px;
  height: 140px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface);
  gap: 2px;
}
.gauge-value { font-size: 2.6rem; font-weight: 700; line-height: 1; }
.gauge-unit { font-size: 1.2rem; color: var(--muted); margin-top: 6px; }
.gauge-band { font-size: 0.8rem; color: var(--muted); margin-top: 4px; }
.gauge-legend { display: flex; gap: 16px; justify-content: center; flex-wrap: wrap; }
.legend-item { display: flex; align-items: center; gap: 6px; font-size: 0.8rem; color: var(--muted); }
.legend-dot { width: 8px; height: 8px; border-radius: 50%; }
.gauge-helper { margin-top: 10px; font-size: 0.8rem; }
.gauge-help-panel {
  margin-top: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.08);
  font-size: 0.78rem;
  line-height: 1.5;
}
</style>
