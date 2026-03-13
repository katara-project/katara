<template>
  <section class="flow-vis card">
    <div class="flow-vis-header">
      <h2>AI Flow Pipeline</h2>
      <div class="flow-live-badge">
        <span class="pulse-dot"></span> Live
      </div>
    </div>

    <div class="pipeline">
      <div v-for="(stage, i) in stages" :key="stage.id" class="pipeline-segment">
        <div class="pipeline-node" :class="stage.variant">
          <SvgIcon :name="stage.icon" :size="22" class="node-icon" />
          <div class="node-body">
            <span class="node-label">{{ stage.label }}</span>
            <span class="node-metric">{{ stage.metric }}</span>
          </div>
        </div>
        <div v-if="i < stages.length - 1" class="pipeline-edge">
          <svg viewBox="0 0 80 24" class="edge-svg">
            <line x1="0" y1="12" x2="80" y2="12" class="edge-track" />
            <line x1="0" y1="12" x2="80" y2="12" class="edge-flow" />
            <polygon points="72,6 80,12 72,18" class="edge-arrow" />
          </svg>
        </div>
      </div>
    </div>

    <div class="routing-panel">
      <h3>LLM Routing</h3>
      <div class="route-branches">
        <div
          v-for="branch in branches"
          :key="branch.provider"
          class="route-branch"
          :class="[branch.variant, { active: branch.active }]"
        >
          <div class="branch-bar" :style="{ width: branch.ratio + '%' }"></div>
          <div class="branch-content">
            <SvgIcon :name="branch.icon" :size="20" class="branch-icon" />
            <span class="branch-name">{{ branch.provider }}</span>
            <span class="branch-tag">{{ branch.tag }}</span>
            <span class="branch-pct">{{ branch.ratio }}%</span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import { useMetricsStore } from '../store/metrics'
import SvgIcon from './SvgIcon.vue'

const metrics = useMetricsStore()

function classifyRouteProvider(provider: string): 'local' | 'cloud' | 'midtier' {
  const lower = provider.toLowerCase()
  if (lower.includes('ollama') || lower.includes('local')) return 'local'
  if (lower.includes('mistral')) return 'midtier'
  return 'cloud'
}

const activeRouteClass = computed<null | 'local' | 'cloud' | 'midtier'>(() => {
  const last = metrics.lastRequest
  if (!last) return null
  return classifyRouteProvider(last.routed_provider)
})

const nowEpoch = ref(Math.floor(Date.now() / 1000))
const clock = window.setInterval(() => {
  nowEpoch.value = Math.floor(Date.now() / 1000)
}, 1000)

onUnmounted(() => {
  window.clearInterval(clock)
})

const fingerprintIsRunning = computed(() => {
  const ts = metrics.lastRequest?.ts
  if (!ts) return false
  return nowEpoch.value - ts <= 8
})

const semanticFingerprintLabel = computed(() => {
  const fp = metrics.lastRequest?.semantic_fingerprint
  if (!fp) return 'pending'
  if (fingerprintIsRunning.value) return 'running'
  const short = fp.length > 10 ? fp.slice(0, 10) : fp
  return `${short}…`
})

const semanticCacheLabel = computed(() => {
  const last = metrics.lastRequest
  if (!last) return `${metrics.cacheHitRatio}% hit`
  const semantic = last.semantic_cache_hit ? 'semantic hit' : 'semantic miss'
  return `${metrics.cacheHitRatio}% hit · ${semantic}`
})

const stages = computed(() => [
  { id: 'request', icon: 'inbox', label: 'Request', metric: `${metrics.rawTokens.toLocaleString()} tok`, variant: 'default' },
  { id: 'fingerprint', icon: 'fingerprint', label: 'Fingerprint', metric: semanticFingerprintLabel.value, variant: 'default' },
  { id: 'cache', icon: 'zap', label: 'Cache', metric: semanticCacheLabel.value, variant: 'accent' },
  { id: 'compiler', icon: 'wrench', label: 'Compiler', metric: `${metrics.compiledTokens.toLocaleString()} tok`, variant: 'primary' },
  { id: 'memory', icon: 'brain', label: 'Memory Lens', metric: `${metrics.memoryReusedTokens.toLocaleString()} reused`, variant: 'secondary' },
  { id: 'router', icon: 'shield', label: 'Router', metric: `${metrics.localRatio}% local`, variant: 'good' },
])

const branches = computed(() => {
  const total = Math.max(1, metrics.routesLocal + metrics.routesCloud + metrics.routesMidtier)
  const localPct = Math.round((metrics.routesLocal / total) * 100)
  const cloudPct = Math.round((metrics.routesCloud / total) * 100)
  const midPct = Math.round((metrics.routesMidtier / total) * 100)

  return [
    {
      provider: 'Local LLM',
      icon: 'home',
      tag: `${metrics.routesLocal} req`,
      ratio: localPct,
      variant: 'local',
      active: activeRouteClass.value === 'local',
    },
    {
      provider: 'Cloud Providers',
      icon: 'cloud',
      tag: `${metrics.routesCloud} req`,
      ratio: cloudPct,
      variant: 'cloud',
      active: activeRouteClass.value === 'cloud',
    },
    {
      provider: 'Mid-tier Providers',
      icon: 'globe',
      tag: `${metrics.routesMidtier} req`,
      ratio: midPct,
      variant: 'midtier',
      active: activeRouteClass.value === 'midtier',
    },
  ]
})
</script>

<style scoped>
.flow-vis-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}
.flow-vis-header h2 { margin: 0; }

.flow-live-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  border-radius: 20px;
  background: rgba(44, 255, 179, 0.1);
  color: var(--accent);
  font-size: 0.85rem;
  font-weight: 600;
}
.pulse-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
  animation: pulse 2s ease-in-out infinite;
}

/* ── Pipeline ── */
.pipeline {
  display: flex;
  align-items: center;
  overflow-x: auto;
  padding: 16px 0 24px;
}
.pipeline-segment {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}
.pipeline-node {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  border-radius: 14px;
  background: var(--surface-2);
  border: 1px solid rgba(255, 255, 255, 0.06);
  transition: transform 0.2s, box-shadow 0.2s;
  z-index: 1;
}
.pipeline-node:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}
.pipeline-node.primary {
  border-color: rgba(57, 211, 255, 0.3);
  background: linear-gradient(135deg, rgba(57, 211, 255, 0.08), rgba(57, 211, 255, 0.02));
}
.pipeline-node.secondary {
  border-color: rgba(140, 109, 255, 0.3);
  background: linear-gradient(135deg, rgba(140, 109, 255, 0.08), rgba(140, 109, 255, 0.02));
}
.pipeline-node.accent {
  border-color: rgba(255, 214, 0, 0.3);
  background: linear-gradient(135deg, rgba(255, 214, 0, 0.06), rgba(255, 214, 0, 0.01));
}
.pipeline-node.good {
  border-color: rgba(44, 255, 179, 0.3);
  background: linear-gradient(135deg, rgba(44, 255, 179, 0.08), rgba(44, 255, 179, 0.02));
}
.node-icon { color: var(--muted); }
.node-body { display: flex; flex-direction: column; gap: 2px; }
.node-label { font-weight: 600; font-size: 0.9rem; white-space: nowrap; }
.node-metric { font-size: 0.78rem; color: var(--muted); white-space: nowrap; }

/* ── Edge connectors ── */
.pipeline-edge { width: 60px; height: 24px; flex-shrink: 0; }
.edge-svg { width: 100%; height: 100%; }
.edge-track { stroke: rgba(255, 255, 255, 0.08); stroke-width: 2; }
.edge-flow { stroke: var(--primary); stroke-width: 2; stroke-dasharray: 8 6; animation: flow-dash 1.5s linear infinite; }
.edge-arrow { fill: var(--primary); opacity: 0.6; }

/* ── Routing Panel ── */
.routing-panel {
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}
.routing-panel h3 { font-size: 1rem; font-weight: 600; margin: 0 0 16px; color: var(--muted); }
.route-branches { display: flex; flex-direction: column; gap: 10px; }
.route-branch {
  position: relative;
  overflow: hidden;
  border-radius: 10px;
  background: var(--surface-2);
  border: 1px solid rgba(255, 255, 255, 0.04);
}
.branch-bar {
  position: absolute;
  top: 0; left: 0; bottom: 0;
  border-radius: 10px;
  opacity: 0.15;
  transition: width 1s ease-out;
}
.route-branch.local .branch-bar { background: var(--accent); }
.route-branch.cloud .branch-bar { background: var(--primary); }
.route-branch.midtier .branch-bar { background: var(--secondary); }
.branch-content {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  z-index: 1;
}
.branch-icon { color: var(--muted); flex-shrink: 0; }
.branch-name { font-weight: 600; font-size: 0.9rem; flex: 1; }
.branch-tag {
  padding: 2px 10px;
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.route-branch.local .branch-tag { background: rgba(44, 255, 179, 0.15); color: var(--accent); }
.route-branch.cloud .branch-tag { background: rgba(57, 211, 255, 0.15); color: var(--primary); }
.route-branch.midtier .branch-tag { background: rgba(140, 109, 255, 0.15); color: var(--secondary); }
.branch-pct { font-size: 1.1rem; font-weight: 700; min-width: 48px; text-align: right; }

.route-branch.active .branch-bar {
  opacity: 0.28;
  animation: branchPulse 1.4s ease-in-out infinite;
}

@keyframes branchPulse {
  0%, 100% { transform: scaleX(1); opacity: 0.22; }
  50% { transform: scaleX(1.05); opacity: 0.35; }
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.8); }
}
@keyframes flow-dash {
  to { stroke-dashoffset: -28; }
}
/* ── Responsive ── */
@media (max-width: 768px) {
  .pipeline { flex-direction: column; align-items: stretch; }
  .pipeline-segment { flex-direction: column; align-items: center; }
  .pipeline-edge { width: 24px; height: 40px; transform: rotate(90deg); }
  .pipeline-node { width: 100%; }
  .route-branches { gap: 8px; }
}
</style>
