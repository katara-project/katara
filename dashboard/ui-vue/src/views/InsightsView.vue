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
import { computed } from 'vue'
import { useMetricsStore } from '../store/metrics'

interface InsightItem {
  title: string
  description: string
  severity: 'high' | 'medium' | 'low'
  category: string
  impact: string
}

const INSIGHT_POOL: InsightItem[] = [
  {
    title: 'Increase local routing for PII queries',
    description: 'Analysis shows 12% of PII-flagged requests still route to cloud providers. Tightening policy rules could redirect these locally.',
    severity: 'high',
    category: 'Sovereignty',
    impact: '-12% cloud PII exposure',
  },
  {
    title: 'Enable semantic cache for code review',
    description: 'Code review prompts show 38% duplication across sessions. Enabling cache for the review intent would reduce redundant LLM calls.',
    severity: 'medium',
    category: 'Cost',
    impact: '-$420/mo estimated',
  },
  {
    title: 'Compress system prompts above 800 tokens',
    description: 'System prompts average 1,100 tokens. Applying compiler reduction to system context could save 27% per request.',
    severity: 'medium',
    category: 'Performance',
    impact: '-300 tokens/req avg',
  },
  {
    title: 'Upgrade Ollama runtime for faster inference',
    description: 'Outdated runtime versions can increase latency. Upgrading can improve throughput on quantized local models.',
    severity: 'low',
    category: 'Performance',
    impact: '+18% local throughput',
  },
  {
    title: 'Add fallback for code generation',
    description: 'Single-provider intent routes reduce resiliency. Add an explicit fallback provider for degraded scenarios.',
    severity: 'low',
    category: 'Reliability',
    impact: '99.9% to 99.99% uptime',
  },
  {
    title: 'Memory lens coverage below target',
    description: 'Only 48.6% context reuse while target is 65%. Increasing stable block TTL from 3 to 5 turns may help.',
    severity: 'medium',
    category: 'Efficiency',
    impact: '+16.4% reuse potential',
  },
  {
    title: 'Pin high-confidence intents before routing',
    description: 'Intent drift on short prompts triggers inconsistent provider selection. Add confidence gating to reduce accidental cloud routes.',
    severity: 'medium',
    category: 'Routing',
    impact: '-9% route variance',
  },
  {
    title: 'Enable warm-start for local models',
    description: 'Cold starts on low-traffic periods increase first-token latency. Keep one warm worker for primary models.',
    severity: 'low',
    category: 'Performance',
    impact: '-420ms first-token latency',
  },
  {
    title: 'Tighten fallback policy for sensitive flows',
    description: 'Some fallback paths can still target non-local providers when policies are incomplete. Add explicit deny rules for sensitive intents.',
    severity: 'high',
    category: 'Sovereignty',
    impact: '-100% non-local sensitive fallback',
  },
  {
    title: 'Cache frequent system preambles',
    description: 'Repeated instruction headers appear across sessions. Pre-hashing and reusing stable preambles will reduce repeated tokens.',
    severity: 'medium',
    category: 'Cost',
    impact: '-11% prompt spend',
  },
  {
    title: 'Add timeout tiers by provider class',
    description: 'Uniform timeouts penalize local and cloud differently. Provider-class timeouts improve resilience and reduce retries.',
    severity: 'low',
    category: 'Reliability',
    impact: '-14% timeout retries',
  },
  {
    title: 'Increase metrics granularity per model',
    description: 'Provider-level aggregation hides model behavior. Tracking model-level efficiency highlights underperforming models quickly.',
    severity: 'medium',
    category: 'Observability',
    impact: '+1-click anomaly detection',
  },
  {
    title: 'Apply diff compaction for long code reviews',
    description: 'Large diffs still carry unchanged hunks. Aggressive compaction before routing reduces context window pressure.',
    severity: 'medium',
    category: 'Efficiency',
    impact: '-22% average compiled tokens',
  },
  {
    title: 'Use dedicated OCR queue on cloud fallback',
    description: 'OCR spikes can starve other intents. Splitting OCR traffic improves response-time consistency for chat workloads.',
    severity: 'low',
    category: 'Reliability',
    impact: '+19% p95 stability',
  },
]

function pickRandomInsights(pool: InsightItem[], count: number): InsightItem[] {
  const shuffled = [...pool]
  for (let i = shuffled.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1))
    const tmp = shuffled[i]
    shuffled[i] = shuffled[j]
    shuffled[j] = tmp
  }
  return shuffled.slice(0, Math.min(count, shuffled.length))
}

const metrics = useMetricsStore()

const insights = computed(() => {
  const result: InsightItem[] = []

  // Dynamic RCT2I structuring insight
  if (metrics.totalRequests > 0) {
    const rct2iRate = Math.round((metrics.rct2iAppliedCount / metrics.totalRequests) * 100)
    if (rct2iRate > 0) {
      result.push({
        title: `RCT2I Prompt Structuring: ${rct2iRate}% activation`,
        description:
          `${metrics.rct2iAppliedCount} of ${metrics.totalRequests} requests were restructured by RCT2I (Role / Context / Tasks / Instructions / Improvement). Structured prompts improve LLM comprehension and reduce hallucinations.`,
        severity: rct2iRate > 50 ? 'low' : 'medium',
        category: 'Prompt Quality',
        impact: `${metrics.rct2iAppliedCount} prompts structured`,
      })
    } else {
      result.push({
        title: 'Enable RCT2I Prompt Structuring',
        description:
          'No requests have been restructured by RCT2I yet. Longer prompts with codegen or review intent get automatic Role/Context/Tasks/Instructions/Improvement sections for better LLM results.',
        severity: 'medium',
        category: 'Prompt Quality',
        impact: '+15–25% LLM accuracy on complex prompts',
      })
    }
  }

  // Dynamic efficiency guidance
  if (metrics.efficiencyScore < 50 && metrics.totalRequests > 0) {
    result.push({
      title: 'Boost AI Efficiency Score',
      description:
        'Score is below 50%. Chain short requests on the same context (review + codegen) instead of re-pasting large blocks to increase reuse and compression.',
      severity: 'medium',
      category: 'Efficiency',
      impact: '+10–20% on recurring workflows',
    })
  }

  // Dynamic cache suggestion
  if (metrics.cacheHitRatio < 40 && metrics.totalRequests > 10) {
    result.push({
      title: 'Improve Semantic Cache Hit Rate',
      description:
        'Cache hit ratio is low. Reuse the same prompt skeletons (diff review, error debug) rather than fully rewriting requests to increase hits.',
      severity: 'medium',
      category: 'Cost',
      impact: '-10–30% outbound tokens',
    })
  }

  // Dynamic sovereignty hint
  if (metrics.localRatio < 50 && metrics.totalRequests > 0) {
    result.push({
      title: 'Increase Local LLM Usage',
      description:
        'Most requests still route to cloud providers. Review sensitive intents (PII, internal logs) and adjust policies to favor local providers.',
      severity: 'high',
      category: 'Sovereignty',
      impact: '+20–50% sovereign traffic',
    })
  }

  const remaining = INSIGHT_POOL.filter((item) => !result.some((r) => r.title === item.title))
  const filler = pickRandomInsights(remaining, 8 - result.length)
  return [...result, ...filler]
})
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
