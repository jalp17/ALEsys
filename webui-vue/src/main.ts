import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { VueQueryPlugin } from '@tanstack/vue-query'
import App from './App.vue'
import './style.css'

const API_BASE = import.meta.env.VITE_API_URL || ''

// Auto-login
async function ensureAuthToken() {
  try {
    const existing = localStorage.getItem('alesys_token')
    if (existing) return
    const resp = await fetch(`${API_BASE}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        username: 'admin',
        password: import.meta.env.VITE_ADMIN_PASSWORD || 'alesys_admin_2026_secure',
      }),
    })
    if (resp.ok) {
      const data = await resp.json()
      if (data.token) localStorage.setItem('alesys_token', data.token)
    }
  } catch { /* noop */ }
}
ensureAuthToken()

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/workspaces' },
    { path: '/workspaces', component: () => import('./pages/Workspaces.vue') },
    { path: '/chat', component: () => import('./pages/Chat.vue') },
    { path: '/sessions', component: () => import('./pages/Sessions.vue') },
    { path: '/ingestion', component: () => import('./pages/Ingestion.vue') },
    { path: '/ingestion/markdown', component: () => import('./pages/MarkdownIngestion.vue') },
    { path: '/graph', component: () => import('./pages/Graph.vue') },
    { path: '/settings', component: () => import('./pages/Settings.vue') },
    { path: '/search', component: () => import('./pages/AdvancedSearch.vue') },
    { path: '/analytics', component: () => import('./pages/Analytics.vue') },
    { path: '/debug', component: () => import('./pages/Debug.vue') },
    { path: '/kb-curation', component: () => import('./pages/KnowledgeCuration.vue') },
    { path: '/agents', component: () => import('./pages/Agents.vue') },
    { path: '/orchestrator', component: () => import('./pages/Orchestrator.vue') },
    { path: '/plugins', component: () => import('./pages/Plugins.vue') },
    { path: '/test-generation', component: () => import('./pages/TestGeneration.vue') },
    { path: '/refactoring', component: () => import('./pages/Refactoring.vue') },
    { path: '/multi-agent', component: () => import('./pages/MultiAgent.vue') },
    { path: '/workflows', component: () => import('./pages/Workflows.vue') },
    { path: '/learning', component: () => import('./pages/Learning.vue') },
  ],
})

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(VueQueryPlugin)
app.mount('#app')
