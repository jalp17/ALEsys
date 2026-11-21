import { createApp } from 'vue'
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
    { path: '/ingestion', component: () => import('./pages/Ingestion.vue') },
    { path: '/ingestion/markdown', component: () => import('./pages/MarkdownIngestion.vue') },
    { path: '/graph', component: () => import('./pages/Graph.vue') },
    { path: '/settings', component: () => import('./pages/Settings.vue') },
  ],
})

const app = createApp(App)
app.use(router)
app.use(VueQueryPlugin)
app.mount('#app')
