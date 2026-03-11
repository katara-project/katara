<template>
  <div class="area-chart">
    <svg :viewBox="`0 0 ${width} ${height}`" preserveAspectRatio="none" class="area-svg">
      <defs>
        <linearGradient v-for="(series, i) in seriesData" :key="'g' + i" :id="`area-grad-${uid}-${i}`" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" :stop-color="series.color" stop-opacity="0.25" />
          <stop offset="100%" :stop-color="series.color" stop-opacity="0.01" />
        </linearGradient>
      </defs>
      <!-- Grid lines -->
      <line v-for="y in gridLines" :key="y" :x1="padL" :y1="y" :x2="width - padR" :y2="y" stroke="rgba(255,255,255,0.04)" stroke-width="1" />
      <!-- Areas -->
      <path v-for="(series, i) in seriesData" :key="'a' + i" :d="areaPath(series.points)" :fill="`url(#area-grad-${uid}-${i})`" />
      <!-- Lines -->
      <path v-for="(series, i) in seriesData" :key="'l' + i" :d="linePath(series.points)" fill="none" :stroke="series.color" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="area-line" />
      <!-- Dots on last point -->
      <circle v-for="(series, i) in seriesData" :key="'d' + i" :cx="series.points[series.points.length - 1].x" :cy="series.points[series.points.length - 1].y" r="4" :fill="series.color" class="area-dot" />
    </svg>
    <div v-if="labels.length" class="area-labels">
      <span v-for="(l, i) in labels" :key="i" class="area-label">{{ l }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Series { name: string; data: number[]; color: string }

const props = withDefaults(defineProps<{
  series: Series[]
  labels?: string[]
  width?: number
  height?: number
}>(), {
  labels: () => [],
  width: 600,
  height: 200,
})

const uid = Math.random().toString(36).slice(2, 8)
const padL = 8
const padR = 8
const padT = 12
const padB = 24

const globalMax = computed(() => Math.max(...props.series.flatMap((s: Series) => s.data)))
const globalMin = computed(() => Math.min(...props.series.flatMap((s: Series) => s.data)))

const gridLines = computed(() => {
  const count = 4
  const lines: number[] = []
  for (let i = 0; i <= count; i++) {
    lines.push(padT + (i / count) * (props.height - padT - padB))
  }
  return lines
})

const seriesData = computed(() => {
  const range = (globalMax.value - globalMin.value) || 1
  return props.series.map((s: Series) => ({
    ...s,
    points: s.data.map((v: number, i: number) => ({
      x: padL + (i / (s.data.length - 1)) * (props.width - padL - padR),
      y: padT + (1 - (v - globalMin.value) / range) * (props.height - padT - padB),
    })),
  }))
})

function linePath(pts: { x: number; y: number }[]) {
  if (pts.length < 2) return ''
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
}

function areaPath(pts: { x: number; y: number }[]) {
  if (pts.length < 2) return ''
  return `${linePath(pts)} L${pts[pts.length - 1].x},${props.height - padB} L${pts[0].x},${props.height - padB} Z`
}
</script>

<style scoped>
.area-chart { position: relative; }
.area-svg { width: 100%; height: auto; display: block; }
.area-line { filter: drop-shadow(0 0 4px currentColor); }
.area-dot { animation: dot-glow 2s ease-in-out infinite; }
.area-labels {
  display: flex;
  justify-content: space-between;
  padding: 4px 8px 0;
}
.area-label {
  font-size: 0.72rem;
  color: var(--muted);
}
@keyframes dot-glow {
  0%, 100% { opacity: 1; r: 4; }
  50% { opacity: 0.7; }
}
</style>
