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
        <RouterLink :to="{ name: 'savings' }" @click="menuOpen = false"><SvgIcon name="leaf" :size="18" /> Savings &amp; Impact</RouterLink>
        <RouterLink :to="{ name: 'flow' }" @click="menuOpen = false"><SvgIcon name="git-branch" :size="18" /> AI Flow</RouterLink>
        <RouterLink :to="{ name: 'memory' }" @click="menuOpen = false"><SvgIcon name="brain" :size="18" /> Memory</RouterLink>
        <RouterLink :to="{ name: 'insights' }" @click="menuOpen = false"><SvgIcon name="lightbulb" :size="18" /> Insights</RouterLink>
        <RouterLink :to="{ name: 'benchmarks' }" @click="menuOpen = false"><SvgIcon name="trending-up" :size="18" /> Benchmarks</RouterLink>
        <RouterLink :to="{ name: 'audit' }" @click="menuOpen = false"><SvgIcon name="radio" :size="18" /> Runtime Audit</RouterLink>
        <RouterLink :to="{ name: 'providers' }" @click="menuOpen = false"><SvgIcon name="server" :size="18" /> Providers</RouterLink>
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
