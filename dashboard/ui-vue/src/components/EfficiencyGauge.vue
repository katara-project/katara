<template>
  <section class="card gauge-card">
    <h2>AI Efficiency Score</h2>
    <div class="gauge-wrap">
      <div class="gauge-ring" :style="gaugeStyle">
        <span>{{ score }}</span>
      </div>
    </div>
    <p class="muted">Composite score based on token avoidance, memory reuse, routing quality, and policy fit.</p>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ score: number }>()

const gaugeStyle = computed(() => ({
  background: `conic-gradient(var(--accent) 0 ${props.score}%, rgba(255,255,255,0.08) ${props.score}% 100%)`,
}))
</script>
