<template>
  <div class="app-shell">
    <!-- Mobile header -->
    <header class="mobile-header">
      <button class="burger" aria-label="Toggle menu" @click="menuOpen = !menuOpen">
        <span /><span /><span />
      </button>
      <div class="brand-icon">
        <img src="/distira_app_icon.svg" alt="Distira" class="brand-logo" />
      </div>
    </header>
    <div v-if="menuOpen" class="overlay" @click="menuOpen = false" />
    <aside class="sidebar" :class="{ open: menuOpen }">
      <div class="brand-wrap">
        <div class="brand-icon"><img src="/distira_app_icon.svg" alt="Distira" class="brand-logo" /></div>
        <div>
          <h1>DISTIRA</h1>
          <p class="brand-sub">The AI Context Compiler</p>
        </div>
      </div>
      <nav>
        <RouterLink :to="{ name: 'overview' }" @click="menuOpen = false"><SvgIcon name="chart-bar" :size="18" /> Overview</RouterLink>
        <RouterLink :to="{ name: 'flow' }" @click="menuOpen = false"><SvgIcon name="git-branch" :size="18" /> AI Flow</RouterLink>
        <RouterLink :to="{ name: 'memory' }" @click="menuOpen = false"><SvgIcon name="brain" :size="18" /> Memory</RouterLink>
        <RouterLink :to="{ name: 'providers' }" @click="menuOpen = false"><SvgIcon name="server" :size="18" /> Providers</RouterLink>
        <RouterLink :to="{ name: 'savings' }" @click="menuOpen = false"><SvgIcon name="leaf" :size="18" /> Savings &amp; Impact</RouterLink>
        <RouterLink :to="{ name: 'insights' }" @click="menuOpen = false"><SvgIcon name="lightbulb" :size="18" /> Insights</RouterLink>
        <RouterLink :to="{ name: 'benchmarks' }" @click="menuOpen = false"><SvgIcon name="trending-up" :size="18" /> Benchmarks</RouterLink>
        <RouterLink :to="{ name: 'audit' }" @click="menuOpen = false"><SvgIcon name="radio" :size="18" /> Runtime Audit</RouterLink>
        <RouterLink :to="{ name: 'guide' }" @click="menuOpen = false"><SvgIcon name="book" :size="18" /> Guide</RouterLink>
      </nav>
      <div class="sidebar-footer">
        <span class="sse-indicator" :class="{ live: metrics.connected }">
          <span class="dot" /> {{ metrics.connected ? 'Live' : 'Offline' }}
        </span>
        <span class="version-tag">v{{ metrics.appVersion }}</span>
      </div>
    </aside>
    <main class="content">
      <div class="global-top-bar">
        <div class="top-bar-spacer"></div>
        <div class="tb-wrapper" ref="notifRef">
          <button class="tb-btn" aria-label="Notifications" @click="notifOpen = !notifOpen">
            <SvgIcon name="bell" :size="18" />
            <span v-if="metrics.alerts.length" class="tb-badge">{{ metrics.alerts.length }}</span>
          </button>
          <div v-if="notifOpen" class="tb-dropdown notif-dropdown">
            <div class="tb-dropdown-header">
              <span>Notifications</span>
              <span class="tb-count">{{ metrics.alerts.length }}</span>
            </div>
            <ul v-if="metrics.alerts.length" class="notif-list">
              <li v-for="(a, i) in metrics.alerts" :key="i" class="notif-item">
                <span class="notif-type" :class="'notif-type--' + a.type">{{ a.type }}</span>
                <span class="notif-msg">{{ a.message }}</span>
              </li>
            </ul>
            <div v-else class="tb-empty">No alerts — all systems nominal.</div>
          </div>
        </div>
        <div class="tb-wrapper" ref="helpRef">
          <button class="tb-btn" aria-label="Help" @click="helpOpen = !helpOpen">
            <SvgIcon name="help-circle" :size="18" />
          </button>
          <div v-if="helpOpen" class="tb-dropdown help-dropdown">
            <div class="tb-dropdown-header">
              <span>{{ currentHelp.title }}</span>
            </div>
            <div class="help-body">
              <p class="help-desc">{{ currentHelp.description }}</p>
              <ul class="help-features">
                <li v-for="(f, i) in currentHelp.features" :key="i">{{ f }}</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import SvgIcon from './components/SvgIcon.vue'
import { useMetricsStore } from './store/metrics'

const menuOpen = ref(false)
const notifOpen = ref(false)
const helpOpen = ref(false)
const notifRef = ref<HTMLElement | null>(null)
const helpRef = ref<HTMLElement | null>(null)
const metrics = useMetricsStore()
const route = useRoute()

const helpMap: Record<string, { title: string; description: string; features: string[] }> = {
  overview: {
    title: 'Overview',
    description: 'Central dashboard showing real-time AI traffic, efficiency score, and resource usage.',
    features: [
      'AI Efficiency Score — measures compression, memory reuse, and sovereign routing',
      'Token metrics — raw vs compiled tokens with sparkline history',
      'Routing breakdown — local vs cloud request distribution',
      'Live alerts — provider health and system notifications',
    ],
  },
  savings: {
    title: 'Savings & Impact',
    description: 'Track the environmental and cost impact of context compilation.',
    features: [
      'Token savings over time with percentage targets',
      'Estimated CO₂ and cost reduction',
      'Budget tracking per session',
    ],
  },
  flow: {
    title: 'AI Flow',
    description: 'Visualize how requests flow through the DISTIRA pipeline.',
    features: [
      'Step-by-step pipeline visualization (compile → cache → route → forward)',
      'Per-request lineage showing client, intent, and routed provider',
      'Real-time flow updates via SSE',
    ],
  },
  memory: {
    title: 'Memory',
    description: 'Inspect stable context blocks stored and reused across requests.',
    features: [
      'Stable block inventory with token counts',
      'Context reuse ratio tracking',
      'Block stability scores per intent',
    ],
  },
  insights: {
    title: 'Insights',
    description: 'Detailed analytics on intent distribution and model performance.',
    features: [
      'Intent breakdown with request counts and avg latency',
      'Model stats — tokens processed per model',
      'Trend analysis across time windows',
    ],
  },
  benchmarks: {
    title: 'Benchmarks',
    description: 'Compare provider performance side by side.',
    features: [
      'Latency comparison across providers',
      'Quality tier classification (codegen, debug, general)',
      'Error rate tracking',
    ],
  },
  audit: {
    title: 'Runtime Audit',
    description: 'Full audit trail of every request through the system.',
    features: [
      'Request history with timestamps and intent',
      'Client → DISTIRA → Provider lineage chain',
      'Cache hit/miss tracking per request',
    ],
  },
  providers: {
    title: 'Providers',
    description: 'Health observatory for all configured LLM providers.',
    features: [
      'Provider health status (healthy / degraded / down)',
      'Avg latency and error rate per provider',
      'Upstream client models detected from VS Code / other clients',
      'Metrics export for enterprise reporting',
    ],
  },
  guide: {
    title: 'Guide',
    description: 'Documentation and setup instructions for DISTIRA.',
    features: [
      'Installation and configuration guide',
      'Provider setup (Ollama, OpenRouter, etc.)',
      'MCP server integration with VS Code',
    ],
  },
}

const currentHelp = computed(() => {
  const name = (route.name as string) || 'overview'
  return helpMap[name] ?? { title: 'Help', description: 'No help available for this page.', features: [] }
})

function onClickOutside(e: MouseEvent) {
  if (notifRef.value && !notifRef.value.contains(e.target as Node)) notifOpen.value = false
  if (helpRef.value && !helpRef.value.contains(e.target as Node)) helpOpen.value = false
}
onMounted(() => document.addEventListener('click', onClickOutside))
onUnmounted(() => document.removeEventListener('click', onClickOutside))
</script>

<style scoped>
.global-top-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
  padding: 8px 0 4px;
}
.top-bar-spacer { flex: 1; }

.tb-wrapper { position: relative; }
.tb-btn {
  background: none;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: var(--muted);
  cursor: pointer;
  padding: 6px;
  display: flex;
  align-items: center;
  transition: color 0.2s, border-color 0.2s, background 0.2s;
}
.tb-btn:hover {
  color: #fff;
  border-color: rgba(255, 255, 255, 0.18);
  background: rgba(255, 255, 255, 0.04);
}
.tb-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  background: var(--accent);
  color: #111;
  font-size: 0.6rem;
  font-weight: 700;
  min-width: 16px;
  height: 16px;
  border-radius: 99px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 4px;
}

.tb-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  width: 320px;
  background: var(--surface-2, #18212d);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  z-index: 100;
  overflow: hidden;
}
.tb-dropdown-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 0.82rem;
  font-weight: 600;
  color: #fff;
}
.tb-count {
  background: rgba(255, 255, 255, 0.08);
  font-size: 0.68rem;
  padding: 1px 7px;
  border-radius: 99px;
  color: var(--muted);
}
.tb-empty {
  padding: 20px 14px;
  text-align: center;
  font-size: 0.78rem;
  color: var(--muted);
}

/* Notification list */
.notif-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 260px;
  overflow-y: auto;
}
.notif-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  font-size: 0.78rem;
  color: var(--muted);
}
.notif-type {
  font-size: 0.6rem;
  font-weight: 700;
  text-transform: uppercase;
  padding: 1px 5px;
  border-radius: 3px;
  flex-shrink: 0;
  margin-top: 1px;
}
.notif-type--error   { background: rgba(239, 68, 68, 0.15); color: #f87171; }
.notif-type--warning { background: rgba(255, 169, 64, 0.15); color: #ffa940; }
.notif-type--info    { background: rgba(99, 102, 241, 0.15); color: var(--primary); }
.notif-msg { line-height: 1.4; }

/* Help dropdown */
.help-body {
  padding: 12px 14px;
}
.help-desc {
  font-size: 0.82rem;
  color: rgba(255, 255, 255, 0.7);
  line-height: 1.5;
  margin: 0 0 10px;
}
.help-features {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.help-features li {
  font-size: 0.78rem;
  color: var(--muted);
  line-height: 1.4;
  padding-left: 14px;
  position: relative;
}
.help-features li::before {
  content: '';
  position: absolute;
  left: 0;
  top: 7px;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--primary);
}
</style>
