<template>
  <div class="tv-chart" ref="chartWrap">
    <div class="tv-toolbar">
      <div class="tv-legend">
        <span v-for="(s, i) in series" :key="s.name" class="tv-legend-item">
          <span class="tv-legend-dot" :style="{ background: resolvedColors[i] }"></span>
          <span class="tv-legend-name">{{ s.name }}</span>
          <strong class="tv-legend-val" :style="{ color: resolvedColors[i] }">
            {{ hoveredIdx >= 0 ? s.data[hoveredIdx]?.toLocaleString() ?? '—' : s.data[s.data.length - 1]?.toLocaleString() }}
          </strong>
        </span>
      </div>
      <button class="tv-export" @click="exportPng" title="Export PNG" aria-label="Export chart as PNG">
        <SvgIcon name="download" :size="16" />
      </button>
    </div>
    <svg
      ref="svgEl"
      :viewBox="`0 0 ${width} ${height}`"
      preserveAspectRatio="none"
      class="tv-svg"
      @pointermove="onPointer"
      @pointerleave="onLeave"
    >
      <defs>
        <linearGradient v-for="(s, i) in series" :key="'g' + i" :id="`tv-g-${uid}-${i}`" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" :stop-color="resolvedColors[i]" stop-opacity="0.22" />
          <stop offset="100%" :stop-color="resolvedColors[i]" stop-opacity="0.01" />
        </linearGradient>
      </defs>
      <!-- Horizontal grid -->
      <line v-for="y in gridY" :key="y" :x1="padL" :y1="y" :x2="width - padR" :y2="y" stroke="rgba(255,255,255,0.04)" stroke-width="1" />
      <!-- Vertical grid -->
      <line v-for="x in gridX" :key="'vg'+x" :x1="x" :y1="padT" :x2="x" :y2="height - padB" stroke="rgba(255,255,255,0.04)" stroke-width="1" />
      <!-- Areas -->
      <path v-for="(sd, i) in seriesData" :key="'a' + i" :d="areaD(sd)" :fill="`url(#tv-g-${uid}-${i})`" />
      <!-- Lines -->
      <path v-for="(sd, i) in seriesData" :key="'l' + i" :d="lineD(sd)" fill="none" :stroke="resolvedColors[i]" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      <!-- Crosshair -->
      <g v-if="hoveredIdx >= 0">
        <line :x1="crossX" :y1="padT" :x2="crossX" :y2="height - padB" stroke="rgba(255,255,255,0.2)" stroke-width="1" stroke-dasharray="4 3" />
        <circle v-for="(sd, i) in seriesData" :key="'c'+i" :cx="crossX" :cy="sd[hoveredIdx]?.y ?? 0" r="4.5" :fill="resolvedColors[i]" stroke="var(--bg)" stroke-width="2" />
      </g>
    </svg>
    <!-- X labels -->
    <div v-if="visibleLabels.length" class="tv-labels">
      <span v-for="(l, i) in visibleLabels" :key="i" class="tv-label">{{ l }}</span>
    </div>
    <!-- Tooltip -->
    <div v-if="hoveredIdx >= 0" class="tv-tooltip" :style="tooltipStyle">
      <div class="tv-tooltip-head">{{ labels[hoveredIdx] ?? '' }}</div>
      <div v-for="(s, i) in series" :key="s.name" class="tv-tooltip-row">
        <span class="tv-tooltip-dot" :style="{ background: resolvedColors[i] }"></span>
        <span>{{ s.name }}</span>
        <strong :style="{ color: resolvedColors[i] }">{{ s.data[hoveredIdx]?.toLocaleString() }}</strong>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import SvgIcon from './SvgIcon.vue'

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

const chartWrap = ref<HTMLElement | null>(null)
const svgEl = ref<SVGSVGElement | null>(null)
const hoveredIdx = ref(-1)
const pointerX = ref(0)

/* ── resolve CSS vars to hex at render time ── */
const resolvedColors = computed(() =>
  props.series.map((s: Series) => {
    if (!s.color.startsWith('var(')) return s.color
    return s.color // CSS variables work in SVG stroke/fill
  }),
)

const gMax = computed(() => Math.max(...props.series.flatMap((s: Series) => s.data)))
const gMin = computed(() => Math.min(...props.series.flatMap((s: Series) => s.data)))

const gridY = computed(() => {
  const n = 4
  const lines: number[] = []
  for (let i = 0; i <= n; i++) lines.push(padT + (i / n) * (props.height - padT - padB))
  return lines
})

const gridX = computed(() => {
  const len = props.labels.length || (props.series[0]?.data.length ?? 0)
  const pts: number[] = []
  for (let i = 0; i < len; i++) {
    pts.push(padL + (i / Math.max(len - 1, 1)) * (props.width - padL - padR))
  }
  return pts
})
const visibleLabels = computed(() => {
  const labels = props.labels ?? []
  const len = labels.length
  if (!len) return labels

  // Keep labels readable: cap visible ticks to ~7 along the x-axis.
  const step = Math.max(1, Math.ceil((len - 1) / 6))
  return labels.map((label: string, idx: number) =>
    idx === 0 || idx === len - 1 || idx % step === 0 ? label : ''
  )
})

const seriesData = computed(() => {
  const range = (gMax.value - gMin.value) || 1
  return props.series.map((s: Series) =>
    s.data.map((v: number, i: number) => ({
      x: padL + (i / Math.max(s.data.length - 1, 1)) * (props.width - padL - padR),
      y: padT + (1 - (v - gMin.value) / range) * (props.height - padT - padB),
    })),
  )
})

const crossX = computed(() => {
  const pts = seriesData.value[0]
  return pts && hoveredIdx.value >= 0 ? pts[hoveredIdx.value]?.x ?? 0 : 0
})

const tooltipStyle = computed(() => {
  const wrap = chartWrap.value
  if (!wrap || hoveredIdx.value < 0) return { display: 'none' }
  const wrapW = wrap.offsetWidth
  const frac = (hoveredIdx.value / Math.max((props.series[0]?.data.length ?? 1) - 1, 1))
  const px = frac * wrapW
  const left = px > wrapW * 0.65 ? px - 160 : px + 16
  return { left: `${left}px`, top: '8px' }
})

function onPointer(e: PointerEvent) {
  const svg = svgEl.value
  if (!svg) return
  const rect = svg.getBoundingClientRect()
  const xRatio = (e.clientX - rect.left) / rect.width
  const len = props.series[0]?.data.length ?? 0
  const idx = Math.round(xRatio * (len - 1))
  hoveredIdx.value = Math.max(0, Math.min(idx, len - 1))
  pointerX.value = e.clientX - rect.left
}

function onLeave() {
  hoveredIdx.value = -1
}

function lineD(pts: { x: number; y: number }[]) {
  if (pts.length < 2) return ''
  let d = `M${pts[0].x},${pts[0].y}`
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(i - 1, 0)]
    const p1 = pts[i]
    const p2 = pts[i + 1]
    const p3 = pts[Math.min(i + 2, pts.length - 1)]
    d += ` C${p1.x + (p2.x - p0.x) / 6},${p1.y + (p2.y - p0.y) / 6} ${p2.x - (p3.x - p1.x) / 6},${p2.y - (p3.y - p1.y) / 6} ${p2.x},${p2.y}`
  }
  return d
}

function areaD(pts: { x: number; y: number }[]) {
  if (pts.length < 2) return ''
  const bottom = props.height - padB
  return `${lineD(pts)} L${pts[pts.length - 1].x},${bottom} L${pts[0].x},${bottom} Z`
}

/** Export chart + legend to PNG via offscreen canvas — numérique responsable: no lib, uses native API */
function exportPng() {
  const svg = svgEl.value
  if (!svg) return
  const serializer = new XMLSerializer()
  const svgStr = serializer.serializeToString(svg)
  const blob = new Blob([svgStr], { type: 'image/svg+xml;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const img = new Image()
  img.onload = () => {
    const dpr = 2
    const cvs = document.createElement('canvas')
    cvs.width = props.width * dpr
    cvs.height = (props.height + 40) * dpr
    const ctx = cvs.getContext('2d')!
    ctx.scale(dpr, dpr)
    ctx.fillStyle = '#121821'
    ctx.fillRect(0, 0, props.width, props.height + 40)
    ctx.drawImage(img, 0, 0, props.width, props.height)
    // Draw legend
    let lx = 8
    ctx.font = '11px Inter, system-ui, sans-serif'
    for (let i = 0; i < props.series.length; i++) {
      const s = props.series[i]
      ctx.fillStyle = resolvedColors.value[i]
      ctx.beginPath()
      ctx.arc(lx + 5, props.height + 18, 4, 0, Math.PI * 2)
      ctx.fill()
      ctx.fillStyle = '#93a1b2'
      ctx.fillText(s.name, lx + 14, props.height + 22)
      lx += ctx.measureText(s.name).width + 30
    }
    cvs.toBlob((b) => {
      if (!b) return
      const a = document.createElement('a')
      a.href = URL.createObjectURL(b)
      a.download = 'katara-chart.png'
      a.click()
      URL.revokeObjectURL(a.href)
    }, 'image/png')
    URL.revokeObjectURL(url)
  }
  img.src = url
}
</script>

<style scoped>
.tv-chart { position: relative; }
.tv-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  gap: 8px;
  flex-wrap: wrap;
}
.tv-legend { display: flex; flex-wrap: wrap; gap: 14px; }
.tv-legend-item { display: flex; align-items: center; gap: 6px; font-size: 0.8rem; }
.tv-legend-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.tv-legend-name { color: var(--muted); }
.tv-legend-val { font-size: 0.85rem; }
.tv-export {
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 8px;
  padding: 6px 10px;
  color: var(--muted);
  cursor: pointer;
  transition: background 0.2s;
  display: flex;
  align-items: center;
}
.tv-export:hover { background: rgba(255,255,255,0.1); color: var(--text); }
.tv-svg {
  width: 100%;
  height: auto;
  display: block;
  cursor: crosshair;
  touch-action: none;
}
.tv-labels {
  display: flex;
  justify-content: space-between;
  padding: 4px 8px 0;
}
.tv-label {
  font-size: 0.72rem;
  color: var(--muted);
  min-width: 40px;
  text-align: center;
}
/* Tooltip */
.tv-tooltip {
  position: absolute;
  background: rgba(18, 24, 33, 0.95);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 10px;
  padding: 10px 14px;
  pointer-events: none;
  z-index: 10;
  backdrop-filter: blur(8px);
  min-width: 140px;
}
.tv-tooltip-head {
  font-size: 0.75rem;
  color: var(--muted);
  margin-bottom: 6px;
  font-weight: 600;
}
.tv-tooltip-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.82rem;
  padding: 2px 0;
}
.tv-tooltip-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
.tv-tooltip-row span { flex: 1; color: var(--muted); }
.tv-tooltip-row strong { text-align: right; }
@media (max-width: 600px) {
  .tv-legend { gap: 8px; }
  .tv-legend-item { font-size: 0.72rem; }
  .tv-tooltip { min-width: 120px; padding: 8px 10px; }
}
</style>
