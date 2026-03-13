<template>
  <div class="app-shell">
    <!-- Mobile header -->
    <header class="mobile-header">
      <button class="burger" aria-label="Toggle menu" @click="menuOpen = !menuOpen">
        <span /><span /><span />
      </button>
      <div class="brand-icon">K</div>
      <span class="mobile-title">KATARA</span>
    </header>
    <div v-if="menuOpen" class="overlay" @click="menuOpen = false" />
    <aside class="sidebar" :class="{ open: menuOpen }">
      <div class="brand-wrap">
        <div class="brand-icon">K</div>
        <div>
          <h1>KATARA</h1>
          <p class="brand-sub">Sovereign AI Context OS</p>
        </div>
      </div>
      <nav>
        <RouterLink :to="{ name: 'overview' }" @click="menuOpen = false"><SvgIcon name="chart-bar" :size="18" /> Overview</RouterLink>
        <RouterLink :to="{ name: 'flow' }" @click="menuOpen = false"><SvgIcon name="git-branch" :size="18" /> AI Flow</RouterLink>
        <RouterLink :to="{ name: 'memory' }" @click="menuOpen = false"><SvgIcon name="brain" :size="18" /> Memory</RouterLink>
        <RouterLink :to="{ name: 'insights' }" @click="menuOpen = false"><SvgIcon name="lightbulb" :size="18" /> Insights</RouterLink>
        <RouterLink :to="{ name: 'benchmarks' }" @click="menuOpen = false"><SvgIcon name="trending-up" :size="18" /> Benchmarks</RouterLink>
        <RouterLink :to="{ name: 'audit' }" @click="menuOpen = false"><SvgIcon name="radio" :size="18" /> Runtime Audit</RouterLink>
      </nav>
      <div class="sidebar-footer">
        <span class="sse-indicator" :class="{ live: metrics.connected }">
          <span class="dot" /> {{ metrics.connected ? 'Live' : 'Offline' }}
        </span>
        <span class="version-tag">v{{ metrics.appVersion }}</span>
      </div>
    </aside>
    <main class="content">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import SvgIcon from './components/SvgIcon.vue'
import { useMetricsStore } from './store/metrics'

const menuOpen = ref(false)
const metrics = useMetricsStore()
</script>
