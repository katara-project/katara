<template>
  <div>
    <header class="view-header">
      <div>
        <h2>AI Flow</h2>
        <p class="muted">Request lifecycle — from ingestion through sovereign routing to provider.</p>
      </div>
    </header>
    <FlowVisualizer />
    <div class="flow-detail-grid">
      <section class="card flow-detail-card" v-for="detail in details" :key="detail.title">
        <SvgIcon :name="detail.icon" :size="28" class="detail-icon" />
        <h3>{{ detail.title }}</h3>
        <p class="muted">{{ detail.description }}</p>
        <div class="detail-stat">
          <strong>{{ detail.stat }}</strong>
          <span class="muted">{{ detail.statLabel }}</span>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import FlowVisualizer from '../components/FlowVisualizer.vue'
import SvgIcon from '../components/SvgIcon.vue'

const details = [
  {
    icon: 'fingerprint',
    title: 'Fingerprinting',
    description: 'SHA-256 hash of trimmed, lowercased input for cache dedup and request tracking.',
    stat: '< 1ms',
    statLabel: 'avg latency',
  },
  {
    icon: 'zap',
    title: 'Semantic Cache',
    description: 'Identical or near-duplicate prompts served from in-memory cache, skipping redundant LLM calls.',
    stat: '42%',
    statLabel: 'cache hit rate',
  },
  {
    icon: 'wrench',
    title: 'Context Compiler',
    description: 'Deduplicates, compresses, and reduces context to the minimal useful token set for the detected intent.',
    stat: '71.7%',
    statLabel: 'token reduction',
  },
  {
    icon: 'brain',
    title: 'Memory Lens',
    description: 'Identifies stable context blocks from prior turns and reuses them, sending only delta tokens.',
    stat: '6,030',
    statLabel: 'reused tokens',
  },
  {
    icon: 'shield',
    title: 'Sovereign Router',
    description: 'Sensitive data routes to local Ollama; general tasks go to cloud; debug to mid-tier — respecting data residency.',
    stat: '61%',
    statLabel: 'local routing',
  },
  {
    icon: 'radio',
    title: 'Provider Dispatch',
    description: 'Routes to Ollama, OpenAI-compatible, or Mistral endpoints with automatic fallback and retry.',
    stat: '3',
    statLabel: 'active providers',
  },
]
</script>

<style scoped>
.flow-detail-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-top: 20px;
}
.flow-detail-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  transition: transform 0.2s;
}
.flow-detail-card:hover {
  transform: translateY(-2px);
}
.detail-icon { color: var(--primary); }
.flow-detail-card h3 { margin: 0; font-size: 1rem; }
.flow-detail-card p { margin: 0; font-size: 0.85rem; line-height: 1.5; }
.detail-stat {
  margin-top: auto;
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.detail-stat strong { font-size: 1.3rem; color: var(--primary); }
.detail-stat span { font-size: 0.8rem; }
@media (max-width: 1100px) {
  .flow-detail-grid { grid-template-columns: 1fr 1fr; }
}
@media (max-width: 600px) {
  .flow-detail-grid { grid-template-columns: 1fr; }
}
</style>
