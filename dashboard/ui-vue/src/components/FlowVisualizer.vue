<template>
  <section class="card">
    <h2>AI Flow Visualizer</h2>
    <div class="flow-grid">
      <div class="flow-node">Request</div>
      <div class="flow-arrow">→</div>
      <div class="flow-node accent">Compiler</div>
      <div class="flow-arrow">→</div>
      <div class="flow-node accent-alt">Memory Lens</div>
      <div class="flow-arrow">→</div>
      <div class="flow-node good">Router</div>
      <div class="flow-arrow">→</div>
      <div class="flow-node">LLM</div>
    </div>
    <p class="muted">In the production runtime, this view will animate real request paths and show token avoidance in each stage.</p>
  </section>
</template>
