<template>
  <div class="sparkline-wrap">
    <svg :viewBox="`0 0 ${width} ${height}`" preserveAspectRatio="none" class="sparkline-svg">
      <defs>
        <linearGradient :id="gradientId" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" :stop-color="color" stop-opacity="0.3" />
          <stop offset="100%" :stop-color="color" stop-opacity="0.02" />
        </linearGradient>
      </defs>
      <path :d="areaPath" :fill="`url(#${gradientId})`" />
      <path :d="linePath" fill="none" :stroke="color" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="sparkline-line" />
      <circle v-if="showDot" :cx="dotX" :cy="dotY" r="3.5" :fill="color" class="sparkline-dot" />
    </svg>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  data: number[]
  color?: string
  width?: number
  height?: number
  showDot?: boolean
}>(), {
  color: 'var(--primary)',
  width: 200,
  height: 60,
  showDot: true,
})

const gradientId = computed(() => `spark-grad-${Math.random().toString(36).slice(2, 8)}`)

const points = computed(() => {
  const max = Math.max(...props.data)
  const min = Math.min(...props.data)
  const range = max - min || 1
  const pad = 4
  return props.data.map((v: number, i: number) => ({
    x: pad + (i / (props.data.length - 1)) * (props.width - pad * 2),
    y: pad + (1 - (v - min) / range) * (props.height - pad * 2),
  }))
})

const linePath = computed(() => {
  if (points.value.length < 2) return ''
  const pts = points.value
  let d = `M${pts[0].x},${pts[0].y}`
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(i - 1, 0)]
    const p1 = pts[i]
    const p2 = pts[i + 1]
    const p3 = pts[Math.min(i + 2, pts.length - 1)]
    const cp1x = p1.x + (p2.x - p0.x) / 6
    const cp1y = p1.y + (p2.y - p0.y) / 6
    const cp2x = p2.x - (p3.x - p1.x) / 6
    const cp2y = p2.y - (p3.y - p1.y) / 6
    d += ` C${cp1x},${cp1y} ${cp2x},${cp2y} ${p2.x},${p2.y}`
  }
  return d
})

const areaPath = computed(() => {
  if (!linePath.value) return ''
  const pts = points.value
  return `${linePath.value} L${pts[pts.length - 1].x},${props.height} L${pts[0].x},${props.height} Z`
})

const dotX = computed(() => points.value[points.value.length - 1]?.x ?? 0)
const dotY = computed(() => points.value[points.value.length - 1]?.y ?? 0)
</script>

<style scoped>
.sparkline-wrap {
  width: 100%;
  height: 100%;
}
.sparkline-svg {
  width: 100%;
  height: 100%;
  display: block;
}
.sparkline-line {
  filter: drop-shadow(0 0 4px currentColor);
}
.sparkline-dot {
  filter: drop-shadow(0 0 6px currentColor);
  animation: dot-pulse 2s ease-in-out infinite;
}
@keyframes dot-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}
</style>
