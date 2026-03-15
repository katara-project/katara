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
import { computed } from 'vue'
import FlowVisualizer from '../components/FlowVisualizer.vue'
import SvgIcon from '../components/SvgIcon.vue'
import { useMetricsStore } from '../store/metrics'

const metrics = useMetricsStore()

const last = computed(() => metrics.lastRequest)

const perReqReduction = computed(() => {
  const r = last.value?.raw_tokens ?? 0
  const c = last.value?.compiled_tokens ?? 0
  return r > 0 ? Math.round(((r - c) / r) * 1000) / 10 : 0
})

const details = computed(() => [
  {
    icon: 'fingerprint',
    title: 'Fingerprinting',
    description: 'SHA-256 hash of trimmed, lowercased input for cache dedup and request tracking.',
    stat: last.value?.semantic_fingerprint
      ? last.value.semantic_fingerprint.slice(0, 12) + '…'
      : '–',
    statLabel: 'fingerprint',
  },
  {
    icon: 'zap',
    title: 'Semantic Cache',
    description: 'Identical or near-duplicate prompts served from in-memory cache, skipping redundant LLM calls.',
    stat: last.value
      ? (last.value.cache_hit ? (last.value.semantic_cache_hit ? 'Semantic Hit' : 'Hit') : 'Miss')
      : '–',
    statLabel: 'last request',
  },
  {
    icon: 'wrench',
    title: 'Context Compiler',
    description: 'Deduplicates, compresses, and reduces context to the minimal useful token set for the detected intent.',
    stat: last.value
      ? `${(last.value.raw_tokens ?? 0).toLocaleString()} → ${(last.value.compiled_tokens ?? 0).toLocaleString()} (−${perReqReduction.value}%)`
      : '–',
    statLabel: 'tokens: raw → compiled',
  },
  {
    icon: 'brain',
    title: 'Memory Lens',
    description: 'Identifies stable context blocks from prior turns and reuses them, sending only delta tokens.',
    stat: last.value
      ? `${(last.value.tokens_saved ?? 0).toLocaleString()}`
      : '–',
    statLabel: 'tokens saved',
  },
  {
    icon: 'shield',
    title: 'Sovereign Router',
    description: 'Sensitive data routes to local Ollama; general tasks go to cloud; debug to mid-tier — respecting data residency.',
    stat: last.value ? last.value.routed_provider : '–',
    statLabel: last.value?.sensitive ? 'forced local (sensitive)' : 'routed provider',
  },
  {
    icon: 'radio',
    title: 'Provider Dispatch',
    description: 'Routes to Ollama, OpenAI-compatible, or Mistral endpoints with automatic fallback and retry.',
    stat: last.value ? last.value.intent : '–',
    statLabel: 'detected intent',
  },
])
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
