<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Context Memory</h2>
        <p class="muted">Track stable context blocks, delta tokens, and reuse optimization across sessions.</p>
      </div>
    </header>

    <div class="memory-stats">
      <MetricCard label="Reused Tokens" :value="metrics.memoryReusedTokens.toLocaleString()" hint="From stable blocks" accent="secondary" />
      <MetricCard label="Reuse Ratio" value="48.6%" hint="Context reuse efficiency" accent="accent" />
      <MetricCard label="Stable Blocks" value="14" hint="Identified as reusable" accent="primary" />
    </div>

    <section class="card memory-blocks">
      <h3>Context Block Status</h3>
      <div class="block-grid">
        <div v-for="block in blocks" :key="block.id" class="mem-block" :class="block.status">
          <div class="block-header">
            <span class="block-id">#{{ block.id }}</span>
            <span class="block-badge">{{ block.status }}</span>
          </div>
          <div class="block-tokens">{{ block.tokens }} tokens</div>
          <div class="block-desc">{{ block.label }}</div>
        </div>
      </div>
    </section>

    <section class="card">
      <h3>How Memory Lensing Works</h3>
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
import { useMetricsStore } from '../store/metrics'
import MetricCard from '../components/MetricCard.vue'

const metrics = useMetricsStore()

const blocks = [
  { id: 1, label: 'System prompt', tokens: 820, status: 'stable' },
  { id: 2, label: 'User context (session)', tokens: 1240, status: 'stable' },
  { id: 3, label: 'Previous assistant response', tokens: 640, status: 'stable' },
  { id: 4, label: 'Current query', tokens: 180, status: 'delta' },
  { id: 5, label: 'Tool outputs (last turn)', tokens: 350, status: 'delta' },
  { id: 6, label: 'Deprecated context', tokens: 420, status: 'evicted' },
]

const steps = [
  { title: 'Fingerprint each block', desc: 'Every context segment is hashed to detect duplicates across turns.' },
  { title: 'Classify stability', desc: 'Blocks unchanged for 3+ turns are marked stable and cached.' },
  { title: 'Send only deltas', desc: 'Only new or modified blocks are compiled and sent to the provider.' },
  { title: 'Measure reuse', desc: 'Context reuse ratio becomes a first-class optimization metric.' },
]
</script>

<style scoped>
.memory-stats { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 20px; }
.memory-blocks h3 { margin: 0 0 16px; }
.block-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
.mem-block {
  padding: 14px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  transition: transform 0.2s;
}
.mem-block:hover { transform: translateY(-2px); }
.mem-block.stable { border-color: rgba(44, 255, 179, 0.2); }
.mem-block.delta { border-color: rgba(57, 211, 255, 0.2); }
.mem-block.evicted { border-color: rgba(255, 100, 100, 0.2); opacity: 0.6; }
.block-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.block-id { font-weight: 700; font-size: 0.85rem; color: var(--muted); }
.block-badge {
  padding: 2px 10px;
  border-radius: 12px;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.mem-block.stable .block-badge { background: rgba(44, 255, 179, 0.15); color: var(--accent); }
.mem-block.delta .block-badge { background: rgba(57, 211, 255, 0.15); color: var(--primary); }
.mem-block.evicted .block-badge { background: rgba(255, 100, 100, 0.15); color: #ff6464; }
.block-tokens { font-size: 1.1rem; font-weight: 700; margin-bottom: 4px; }
.block-desc { font-size: 0.82rem; color: var(--muted); }
.lens-steps { display: flex; flex-direction: column; gap: 16px; margin-top: 8px; }
.lens-step { display: flex; gap: 16px; align-items: flex-start; }
.step-num {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--primary), var(--secondary));
  display: grid;
  place-items: center;
  font-weight: 700;
  font-size: 0.85rem;
  flex-shrink: 0;
}
.step-content strong { display: block; margin-bottom: 4px; }
.step-content p { margin: 0; font-size: 0.88rem; line-height: 1.5; }
@media (max-width: 1100px) {
  .memory-stats, .block-grid { grid-template-columns: 1fr; }
}
@media (max-width: 768px) {
  .memory-stats { grid-template-columns: 1fr; }
  .block-grid { grid-template-columns: 1fr 1fr; }
}
@media (max-width: 480px) {
  .block-grid { grid-template-columns: 1fr; }
}
</style>
