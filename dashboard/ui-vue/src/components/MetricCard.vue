<template>
  <section class="card metric-card" :class="accentClass">
    <div class="metric-accent-bar"></div>
    <span class="label">{{ label }}</span>
    <strong class="value">{{ value }}</strong>
    <small class="hint" v-if="hint">{{ hint }}</small>
    <div class="metric-sparkline" v-if="$slots.default">
      <slot />
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ label: string; value: string | number; hint?: string; accent?: string }>()

const accentClass = computed(() => props.accent ? `metric-${props.accent}` : '')
</script>

<style scoped>
.metric-card {
  position: relative;
  overflow: hidden;
}
.metric-accent-bar {
  position: absolute;
  top: 0;
  left: 0;
  width: 3px;
  height: 100%;
  border-radius: 0 3px 3px 0;
  background: var(--muted);
  opacity: 0.4;
}
.metric-primary .metric-accent-bar { background: var(--primary); opacity: 1; }
.metric-secondary .metric-accent-bar { background: var(--secondary); opacity: 1; }
.metric-accent .metric-accent-bar { background: var(--accent); opacity: 1; }
.metric-warn .metric-accent-bar { background: #ffa940; opacity: 1; }
.metric-sparkline {
  margin-top: 8px;
  height: 40px;
}
</style>
