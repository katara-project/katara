import { defineStore } from 'pinia'

export interface MetricsState {
  rawTokens: number
  compiledTokens: number
  memoryReusedTokens: number
  efficiencyScore: number
  localRatio: number
  cloudRatio: number
}

export const useMetricsStore = defineStore('metrics', {
  state: (): MetricsState => ({
    rawTokens: 18_420,
    compiledTokens: 5_210,
    memoryReusedTokens: 6_030,
    efficiencyScore: 84,
    localRatio: 61,
    cloudRatio: 39,
  }),
})
