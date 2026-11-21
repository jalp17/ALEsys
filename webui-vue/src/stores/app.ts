import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

const API_BASE = import.meta.env.VITE_API_URL || ''

export const useAppStore = defineStore('app', () => {
  const token = ref(localStorage.getItem('alesys_token') || '')
  const user = ref<{ username: string; role: string } | null>(null)
  const llmLoaded = ref(false)
  const embedderAvailable = ref(false)
  const apiConnected = ref(false)

  const isAuthenticated = computed(() => !!token.value)

  const setToken = (t: string) => {
    token.value = t
    localStorage.setItem('alesys_token', t)
  }

  const clearToken = () => {
    token.value = ''
    localStorage.removeItem('alesys_token')
    user.value = null
  }

  const authHeaders = () => token.value ? { Authorization: `Bearer ${token.value}` } : {}

  const login = async (username: string, password: string) => {
    const res = await fetch(`${API_BASE}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    })
    if (!res.ok) throw new Error('Login failed')
    const data = await res.json()
    setToken(data.token)
    user.value = { username: data.username || username, role: data.role || 'user' }
    return data
  }

  const logout = () => {
    clearToken()
  }

  const checkHealth = async () => {
    try {
      const res = await fetch(`${API_BASE}/health`)
      if (res.ok) {
        const data = await res.json()
        apiConnected.value = data.status === 'ok'
        embedderAvailable.value = data.embedder === true
        llmLoaded.value = data.llm_loaded === true
      }
    } catch {
      apiConnected.value = false
      embedderAvailable.value = false
      llmLoaded.value = false
    }
  }

  const autoLogin = async () => {
    if (token.value) return
    try {
      await login('admin', import.meta.env.VITE_ADMIN_PASSWORD || 'alesys_admin_2026_secure')
    } catch {
      // noop
    }
  }

  return {
    token,
    user,
    llmLoaded,
    embedderAvailable,
    apiConnected,
    isAuthenticated,
    setToken,
    clearToken,
    authHeaders,
    login,
    logout,
    checkHealth,
    autoLogin,
  }
})
