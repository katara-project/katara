import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'overview',
      component: () => import('../views/OverviewView.vue'),
    },
    {
      path: '/savings',
      name: 'savings',
      component: () => import('../views/SavingsView.vue'),
    },
    {
      path: '/flow',
      name: 'flow',
      component: () => import('../views/FlowView.vue'),
    },
    {
      path: '/memory',
      name: 'memory',
      component: () => import('../views/MemoryView.vue'),
    },
    {
      path: '/insights',
      name: 'insights',
      component: () => import('../views/InsightsView.vue'),
    },
    {
      path: '/benchmarks',
      name: 'benchmarks',
      component: () => import('../views/BenchmarksView.vue'),
    },
    {
      path: '/audit',
      name: 'audit',
      component: () => import('../views/AuditView.vue'),
    },
    {
      path: '/providers',
      name: 'providers',
      component: () => import('../views/ProvidersView.vue'),
    },
    {
      path: '/guide',
      name: 'guide',
      component: () => import('../views/GuideView.vue'),
    },
  ],
})

export default router
