<template>
  <section class="card metric-card">
    <span class="label">{{ label }}</span>
    <strong class="value">{{ value }}</strong>
    <small class="hint">{{ hint }}</small>
  </section>
</template>

<script setup lang="ts">
defineProps<{ label: string; value: string | number; hint?: string }>()
</script>
