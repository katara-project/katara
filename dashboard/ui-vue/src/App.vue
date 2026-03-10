<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand-wrap">
        <img src="./assets/katara-mark.svg" class="brand-mark" alt="KATARA" />
        <div>
          <h1>KATARA</h1>
          <p>AI Context OS</p>
        </div>
      </div>
      <nav>
        <RouterLink :to="{ name: 'overview' }">Overview</RouterLink>
        <RouterLink :to="{ name: 'flow' }">AI Flow</RouterLink>
        <RouterLink :to="{ name: 'memory' }">Memory</RouterLink>
        <RouterLink :to="{ name: 'insights' }">Insights</RouterLink>
        <RouterLink :to="{ name: 'benchmarks' }">Benchmarks</RouterLink>
      </nav>
    </aside>
    <main class="content">
      <RouterView />
    </main>
  </div>
</template>
