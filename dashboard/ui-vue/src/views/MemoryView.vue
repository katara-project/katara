<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Context Memory</h2>
        <p class="muted">DISTIRA remembers context from previous requests so it doesn't re-process what it already knows.</p>
      </div>
    </header>

    <!-- Top KPIs -->
    <div class="memory-stats">
      <MetricCard label="Tokens Saved by Memory" :value="metrics.memoryReusedTokens.toLocaleString()" hint="Not re-sent to the LLM this session" accent="secondary" />
      <MetricCard label="Memory Efficiency" :value="reuseRatioPct + '%'" hint="Share of context recognised from prior requests" accent="accent" />
      <MetricCard label="Topics in Memory" :value="metrics.stableBlocks.toString()" hint="Context blocks actively reused" accent="primary" />
    </div>

    <!-- Session savings bar -->
    <section class="card savings-section">
      <h3>Session Savings</h3>
      <p class="muted savings-label">
        DISTIRA has saved <strong>{{ metrics.memoryReusedTokens.toLocaleString() }} tokens</strong> out of
        {{ metrics.rawTokens.toLocaleString() }} total tokens sent this session.
      </p>
      <div class="savings-bar-bg">
        <div class="savings-bar-fill" :style="{ width: reuseRatioPct + '%' }"></div>
      </div>
      <div class="savings-bar-legend">
        <span class="legend-reused">● Reused ({{ reuseRatioPct }}%)</span>
        <span class="legend-new">● New ({{ 100 - reuseRatioPct }}%)</span>
      </div>
    </section>

    <!-- Topic memory breakdown -->
    <section class="card">
      <h3>What DISTIRA Has in Memory</h3>
      <p class="muted" style="margin-bottom:16px">
        Each row is a topic area DISTIRA has learnt from your requests. The stronger the bar, the more reliably it will be reused.
      </p>
      <div v-if="topicGroups.length" class="topic-list">
        <div v-for="group in topicGroups" :key="group.intent" class="topic-row">
          <div class="topic-meta">
            <span class="topic-label">{{ group.label }}</span>
            <span class="topic-stat">{{ group.count }} block{{ group.count !== 1 ? 's' : '' }} · {{ group.tokens }} tokens</span>
          </div>
          <div class="topic-bar-bg">
            <div class="topic-bar-fill" :class="group.healthClass" :style="{ width: group.healthPct + '%' }"></div>
          </div>
        </div>
      </div>
      <p v-else class="muted">No memory yet — send a few requests and DISTIRA will start building context.</p>
    </section>

    <!-- How it works -->
    <section class="card">
      <h3>How It Works</h3>
      <div class="lens-steps">
        <div class="lens-step" v-for="(step, i) in steps" :key="i">
          <div class="step-num">{{ i + 1 }}</div>
          <div class="step-content">
            <strong>{{ step.title }}</strong>
            <p class="muted">{{ step.desc }}</p>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMetricsStore } from '../store/metrics'
import MetricCard from '../components/MetricCard.vue'

const metrics = useMetricsStore()

const INTENT_LABELS: Record<string, string> = {
  codegen: '💻 Code generation',
  debug: '🐛 Debugging',
  review: '🔍 Code review',
  summarize: '📝 Summarisation',
  translate: '🌍 Translation',
  ocr: '🖼️ OCR / Image',
  general: '💬 General questions',
  '': '💬 General questions',
}

const reuseRatioPct = computed(() => {
  return metrics.rawTokens > 0
    ? Math.round((metrics.memoryReusedTokens / metrics.rawTokens) * 100)
    : 0
})

// Group live blocks by intent, compute aggregate health (avg stability)
const topicGroups = computed(() => {
  const map: Record<string, { count: number; tokens: number; stabilitySum: number }> = {}
  for (const b of metrics.contextBlocksSummary) {
    const key = b.intent || ''
    if (!map[key]) map[key] = { count: 0, tokens: 0, stabilitySum: 0 }
    map[key].count++
    map[key].tokens += b.token_count
    map[key].stabilitySum += b.stability
  }
  return Object.entries(map)
    .map(([intent, g]) => {
      const avgStability = g.count > 0 ? g.stabilitySum / g.count : 0
      const healthPct = Math.round(avgStability * 100)
      const healthClass =
        healthPct >= 70 ? 'health-strong' : healthPct >= 30 ? 'health-mid' : 'health-weak'
      return {
        intent,
        label: INTENT_LABELS[intent] ?? intent,
        count: g.count,
        tokens: g.tokens,
        healthPct,
        healthClass,
      }
    })
    .sort((a, b) => b.tokens - a.tokens)
})

const steps = [
  { title: 'Every request is fingerprinted', desc: 'DISTIRA hashes each incoming context to detect if it has seen similar content before.' },
  { title: 'Stable content is cached', desc: 'Context that appears repeatedly is stored as a memory block and reused in future requests.' },
  { title: 'Only new content is sent', desc: 'On your next request, only the parts DISTIRA has not seen before are compiled and forwarded to the LLM.' },
  { title: 'You save tokens every session', desc: 'The memory efficiency score shows how much context was served from cache rather than re-processed.' },
]
</script>

<style scoped>
.memory-stats { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 20px; }

/* ── Session savings bar ─────────────────────────────────────────────── */
.savings-section h3 { margin: 0 0 8px; }
.savings-label { margin: 0 0 14px; font-size: 0.9rem; }
.savings-bar-bg {
  height: 12px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  overflow: hidden;
  margin-bottom: 8px;
}
.savings-bar-fill {
  height: 100%;
  border-radius: 8px;
  background: linear-gradient(90deg, var(--secondary), var(--accent));
  transition: width 0.6s ease;
}
.savings-bar-legend { display: flex; gap: 20px; font-size: 0.82rem; color: var(--muted); }
.legend-reused { color: var(--accent); }
.legend-new { color: var(--muted); }

/* ── Topic memory list ───────────────────────────────────────────────── */
.topic-list { display: flex; flex-direction: column; gap: 14px; }
.topic-row { display: flex; flex-direction: column; gap: 6px; }
.topic-meta { display: flex; justify-content: space-between; align-items: baseline; }
.topic-label { font-weight: 600; font-size: 0.92rem; }
.topic-stat { font-size: 0.8rem; color: var(--muted); }
.topic-bar-bg {
  height: 8px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  overflow: hidden;
}
.topic-bar-fill {
  height: 100%;
  border-radius: 6px;
  transition: width 0.6s ease;
}
.health-strong { background: var(--accent); }
.health-mid    { background: var(--primary); }
.health-weak   { background: rgba(255, 180, 50, 0.7); }

/* ── How it works steps ──────────────────────────────────────────────── */
.lens-steps { display: flex; flex-direction: column; gap: 16px; margin-top: 8px; }
.lens-step { display: flex; gap: 16px; align-items: flex-start; }
.step-num {
  width: 32px; height: 32px; border-radius: 50%; flex-shrink: 0;
  background: linear-gradient(135deg, var(--primary), var(--secondary));
  display: grid; place-items: center; font-weight: 700; font-size: 0.85rem;
}
.step-content strong { display: block; margin-bottom: 4px; }
.step-content p { margin: 0; font-size: 0.88rem; line-height: 1.5; }

@media (max-width: 768px) {
  .memory-stats { grid-template-columns: 1fr; }
}
</style>
