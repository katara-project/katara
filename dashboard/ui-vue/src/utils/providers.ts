/**
 * Human-readable provider labels and tier classification.
 * Used across OverviewView, AuditView, FlowVisualizer.
 */

const PROVIDER_LABELS: Record<string, string> = {
  'ollama-llama3':                                   'Llama 3',
  'ollama-llama3.3':                                 'Llama 3.3',
  'ollama-qwen2.5-coder':                            'Qwen 2.5 Coder',
  'ollama-mistral-7b-instruct':                      'Mistral 7B',
  'ollama-deepseek-ocr':                             'DeepSeek OCR',
  'openrouter-step-3.5-flash-cloud':                 'Step 3.5 Flash',
  'openrouter-mistral-small-3.1-24b-instruct-cloud': 'Mistral Small 24B',
  'mistral-ocr-2512-cloud':                          'Mistral OCR',
  'openai-gpt4o':                                    'GPT-4o',
  'openai-gpt5':                                     'GPT-5',
  'anthropic-claude-sonnet-4-6':                     'Claude Sonnet 4.6',
  'google-gemini-2-flash':                           'Gemini 2 Flash',
  'lmstudio-default':                                'LM Studio',
  'openwebui-default':                               'OpenWebUI',
}

export interface TierInfo {
  routeLabel: string
  routeClass: 'local' | 'midtier' | 'cloud'
  tierDescription: string
}

/**
 * Maps a raw provider key (e.g. "ollama-mistral-7b-instruct") to a friendly
 * display name. Falls back to the raw key if unmapped.
 */
export function friendlyProvider(key: string): string {
  return PROVIDER_LABELS[key] ?? key
}

/** Quality tier for a provider key: "high" | "standard" | "low". */
const PROVIDER_QUALITY: Record<string, 'high' | 'standard' | 'low'> = {
  'ollama-llama3':                                   'standard',
  'ollama-llama3.3':                                 'high',
  'ollama-qwen2.5-coder':                            'high',
  'ollama-mistral-7b-instruct':                      'standard',
  'ollama-deepseek-ocr':                             'low',
  'openrouter-step-3.5-flash-cloud':                 'standard',
  'openrouter-mistral-small-3.1-24b-instruct-cloud': 'high',
  'mistral-ocr-2512-cloud':                          'high',
  'openai-gpt4o':                                    'high',
  'openai-gpt5':                                     'high',
  'anthropic-claude-sonnet-4-6':                     'high',
  'google-gemini-2-flash':                           'high',
}

export function qualityTier(key: string): 'high' | 'standard' | 'low' {
  return PROVIDER_QUALITY[key] ?? 'standard'
}

/**
 * Classifies a provider key into a routing tier.
 * Uses key structure (ollama- prefix, -cloud suffix) rather than model name
 * to avoid false positives like "mistral-small-cloud" being tagged as Mid-tier.
 */
export function classifyRoute(provider: string): TierInfo {
  const k = provider.toLowerCase()
  if (
    k.startsWith('ollama') ||
    k.includes('local') ||
    k.startsWith('lmstudio') ||
    k.startsWith('openwebui')
  ) {
    return {
      routeLabel: 'On-prem',
      routeClass: 'local',
      tierDescription: 'On-prem — Ollama, LM Studio, OpenWebUI. Data never leaves your machine.',
    }
  }
  if (k.endsWith('-cloud') || k.includes('openrouter') || k.includes('openai') || k.includes('anthropic') || k.includes('google') || k.includes('zhipu') || k.includes('dashscope')) {
    return {
      routeLabel: 'Cloud',
      routeClass: 'cloud',
      tierDescription: 'Cloud — routed to an external provider over the internet',
    }
  }
  return {
    routeLabel: 'Mid-tier',
    routeClass: 'midtier',
    tierDescription: 'Mid-tier — self-hosted or hybrid deployment.',
  }
}
