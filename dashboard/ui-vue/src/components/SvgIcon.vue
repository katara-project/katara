<template>
  <svg
    :viewBox="meta.viewBox"
    :width="size"
    :height="size"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="svg-icon"
    :aria-label="name"
    role="img"
  >
    <!-- eslint-disable vue/no-v-html -->
    <g v-html="meta.paths" />
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  name: string
  size?: number | string
}>(), { size: 20 })

/**
 * Lightweight inline SVG icon set — zero external deps.
 * Each icon: viewBox + raw <path> content.
 * Follows numérique-responsable: no icon-font, no sprite sheet,
 * only the icons actually used are shipped.
 */
const icons: Record<string, { viewBox: string; paths: string }> = {
  /* ── Navigation ── */
  'chart-bar': {
    viewBox: '0 0 24 24',
    paths: '<path d="M3 20h18M5 20V10h3v10M10 20V4h3v16M17 20v-8h3v8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  'git-branch': {
    viewBox: '0 0 24 24',
    paths: '<circle cx="7" cy="7" r="3" stroke="currentColor" stroke-width="1.8"/><circle cx="17" cy="17" r="3" stroke="currentColor" stroke-width="1.8"/><path d="M7 10v2a4 4 0 004 4h6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>',
  },
  brain: {
    viewBox: '0 0 24 24',
    paths: '<path d="M12 2a5 5 0 00-4.78 3.53A4 4 0 004 9.5a4.5 4.5 0 00.68 7.77A4 4 0 008 21h1m3-19a5 5 0 014.78 3.53A4 4 0 0120 9.5a4.5 4.5 0 01-.68 7.77A4 4 0 0016 21h-1M12 2v19" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  lightbulb: {
    viewBox: '0 0 24 24',
    paths: '<path d="M9 18h6M10 22h4M12 2a7 7 0 00-4 12.7V18h8v-3.3A7 7 0 0012 2z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  'trending-up': {
    viewBox: '0 0 24 24',
    paths: '<polyline points="23 6 13.5 15.5 8.5 10.5 1 18" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><polyline points="17 6 23 6 23 12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },

  /* ── Pipeline stages ── */
  inbox: {
    viewBox: '0 0 24 24',
    paths: '<polyline points="22 12 16 12 14 15 10 15 8 12 2 12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><path d="M5.45 5.11L2 12v6a2 2 0 002 2h16a2 2 0 002-2v-6l-3.45-6.89A2 2 0 0016.76 4H7.24a2 2 0 00-1.79 1.11z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  fingerprint: {
    viewBox: '0 0 24 24',
    paths: '<path d="M2 12C2 6.48 6.48 2 12 2s10 4.48 10 10M12 14a2 2 0 100-4 2 2 0 000 4zM12 2v4M22 12h-4M12 22v-4M2 12h4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/><path d="M7.05 7.05l2.83 2.83M14.12 14.12l2.83 2.83" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>',
  },
  zap: {
    viewBox: '0 0 24 24',
    paths: '<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  wrench: {
    viewBox: '0 0 24 24',
    paths: '<path d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  shield: {
    viewBox: '0 0 24 24',
    paths: '<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  radio: {
    viewBox: '0 0 24 24',
    paths: '<circle cx="12" cy="12" r="2" stroke="currentColor" stroke-width="1.8"/><path d="M16.24 7.76a6 6 0 010 8.49M7.76 16.24a6 6 0 010-8.49M19.07 4.93a10 10 0 010 14.14M4.93 19.07a10 10 0 010-14.14" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>',
  },

  /* ── Providers ── */
  home: {
    viewBox: '0 0 24 24',
    paths: '<path d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-4 0v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  cloud: {
    viewBox: '0 0 24 24',
    paths: '<path d="M18 10h-1.26A8 8 0 109 20h9a5 5 0 000-10z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  globe: {
    viewBox: '0 0 24 24',
    paths: '<circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.8"/><path d="M2 12h20M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>',
  },

  /* ── Guide ── */
  book: {
    viewBox: '0 0 24 24',
    paths: '<path d="M4 19.5A2.5 2.5 0 016.5 17H20" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },

  /* ── Misc ── */
  server: {
    viewBox: '0 0 24 24',
    paths: '<rect x="2" y="2" width="20" height="8" rx="2" ry="2" stroke="currentColor" stroke-width="1.8" fill="none"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2" stroke="currentColor" stroke-width="1.8" fill="none"/><line x1="6" y1="6" x2="6.01" y2="6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><line x1="6" y1="18" x2="6.01" y2="18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>',
  },
  download: {
    viewBox: '0 0 24 24',
    paths: '<path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  database: {
    viewBox: '0 0 24 24',
    paths: '<ellipse cx="12" cy="5" rx="9" ry="3" stroke="currentColor" stroke-width="1.8"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3M21 5v14c0 1.66-4 3-9 3s-9-1.34-9-3V5" stroke="currentColor" stroke-width="1.8"/>',
  },
  layers: {
    viewBox: '0 0 24 24',
    paths: '<polygon points="12 2 2 7 12 12 22 7 12 2" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><polyline points="2 17 12 22 22 17" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><polyline points="2 12 12 17 22 12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  },
  crosshair: {
    viewBox: '0 0 24 24',
    paths: '<circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.8"/><line x1="22" y1="12" x2="18" y2="12" stroke="currentColor" stroke-width="1.8"/><line x1="6" y1="12" x2="2" y2="12" stroke="currentColor" stroke-width="1.8"/><line x1="12" y1="6" x2="12" y2="2" stroke="currentColor" stroke-width="1.8"/><line x1="12" y1="22" x2="12" y2="18" stroke="currentColor" stroke-width="1.8"/>',
  },
  leaf: {
    viewBox: '0 0 24 24',
    paths: '<path d="M17 8C8 10 5.9 16.17 3.82 21.34l1.89.66.95-2.3c.48.17.98.3 1.34.3C19 20 22 3 22 3c-1 2-8 2.25-13 3.25S2 11.5 2 13.5s1.75 3.75 1.75 3.75" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"/>',
  },
}

const meta = computed(() => icons[props.name] ?? { viewBox: '0 0 24 24', paths: '' })
</script>

<style scoped>
.svg-icon {
  display: inline-block;
  vertical-align: middle;
  flex-shrink: 0;
}
</style>
