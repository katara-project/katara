<template>
  <div>
    <header class="view-header">
      <div>
        <h2>Optimization Insights</h2>
        <p class="muted">Automated recommendations to reduce cost, improve latency, and strengthen sovereignty.</p>
      </div>
    </header>

    <div class="insights-grid">
      <section v-for="insight in insights" :key="insight.title" class="card insight-card" :class="insight.severity">
        <div class="insight-header">
          <span class="insight-badge">{{ insight.severity }}</span>
          <span class="insight-category">{{ insight.category }}</span>
        </div>
        <h3>{{ insight.title }}</h3>
        <p class="muted">{{ insight.description }}</p>
        <div class="insight-impact">
          <span class="impact-label">Estimated impact:</span>
          <strong class="impact-value">{{ insight.impact }}</strong>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
const insights = [
  {
    title: 'Increase local routing for PII queries',
    description: 'Analysis shows 12% of PII-flagged requests still route to cloud providers. Tightening policy rules could redirect these locally.',
    severity: 'high',
    category: 'Sovereignty',
    impact: '\u221212% cloud PII exposure',
  },
  {
    title: 'Enable semantic cache for code review',
    description: 'Code review prompts show 38% duplication across sessions. Enabling cache for the \"review\" intent would reduce redundant LLM calls.',
    severity: 'medium',
    category: 'Cost',
    impact: '\u2212$420/mo estimated',
  },
  {
    title: 'Compress system prompts above 800 tokens',
    description: 'System prompts average 1,100 tokens. Applying compiler reduction to system context could save 27% per request.',
    severity: 'medium',
    category: 'Performance',
    impact: '\u2212300 tokens/req avg',
  },
  {
    title: 'Upgrade Ollama to v0.4 for faster inference',
    description: 'Local Ollama instance runs v0.3.2. Upgrading to v0.4 provides 18% faster token generation with quantized models.',
    severity: 'low',
    category: 'Performance',
    impact: '+18% local throughput',
  },
  {
    title: 'Add Mistral fallback for code generation',
    description: 'Debug intent currently routes only to Mistral Cloud. Adding a fallback to OpenAI-compatible improves resilience.',
    severity: 'low',
    category: 'Reliability',
    impact: '99.9% \u2192 99.99% uptime',
  },
  {
    title: 'Memory lens coverage below target',
    description: 'Only 48.6% context reuse \u2014 target is 65%. Increasing stable block TTL from 3 to 5 turns may help.',
    severity: 'medium',
    category: 'Efficiency',
    impact: '+16.4% reuse potential',
  },
]
</script>

<style scoped>
.insights-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 16px; }
.insight-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: transform 0.2s;
}
.insight-card:hover { transform: translateY(-2px); }
.insight-header { display: flex; align-items: center; gap: 10px; }
.insight-badge {
  padding: 3px 10px;
  border-radius: 12px;
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.insight-card.high .insight-badge { background: rgba(255, 100, 100, 0.15); color: #ff6464; }
.insight-card.medium .insight-badge { background: rgba(255, 169, 64, 0.15); color: #ffa940; }
.insight-card.low .insight-badge { background: rgba(44, 255, 179, 0.15); color: var(--accent); }
.insight-category { font-size: 0.78rem; color: var(--muted); }
.insight-card h3 { margin: 0; font-size: 1rem; line-height: 1.4; }
.insight-card p { margin: 0; font-size: 0.85rem; line-height: 1.5; }
.insight-impact {
  margin-top: auto;
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.impact-label { font-size: 0.78rem; color: var(--muted); }
.impact-value { font-size: 0.95rem; color: var(--accent); }
@media (max-width: 1100px) {
  .insights-grid { grid-template-columns: 1fr; }
}
@media (max-width: 480px) {
  .insight-card h3 { font-size: 0.92rem; }
  .insight-card p { font-size: 0.8rem; }
}
</style>
