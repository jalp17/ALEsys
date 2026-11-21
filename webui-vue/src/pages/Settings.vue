<template>
  <div class="max-w-4xl mx-auto p-6">
    <h1 class="text-2xl font-bold mb-6">Configuración</h1>

    <!-- LLM Status -->
    <div class="bg-dark-800 p-6 rounded-lg mb-6">
      <h2 class="text-lg font-semibold mb-4">Modelo LLM</h2>
      <div class="flex items-center gap-3 mb-4">
        <div :class="['w-3 h-3 rounded-full', llmStatus.loaded ? 'bg-green-400' : 'bg-red-400']" />
        <span class="text-sm">{{ llmStatus.loaded ? 'Cargado' : 'No cargado' }}</span>
        <span class="text-xs text-gray-500">({{ llmStatus.backend }})</span>
      </div>
      <div class="flex gap-2">
        <button
          @click="loadLLM"
          :disabled="llmLoading"
          class="px-4 py-2 bg-primary-500 hover:bg-primary-400 text-white text-sm rounded transition disabled:opacity-50"
        >
          {{ llmLoading ? 'Cargando...' : 'Cargar LLM' }}
        </button>
        <button
          @click="unloadLLM"
          :disabled="llmLoading"
          class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white text-sm rounded transition disabled:opacity-50"
        >
          Descargar
        </button>
      </div>
      <p v-if="llmMessage" class="text-sm mt-2" :class="llmError ? 'text-red-400' : 'text-green-400'">
        {{ llmMessage }}
      </p>
    </div>

    <!-- Embedder Status -->
    <div class="bg-dark-800 p-6 rounded-lg mb-6">
      <h2 class="text-lg font-semibold mb-4">Embedder</h2>
      <div class="flex items-center gap-3">
        <div :class="['w-3 h-3 rounded-full', embedderStatus ? 'bg-green-400' : 'bg-red-400']" />
        <span class="text-sm">{{ embedderStatus ? 'Disponible' : 'No disponible' }}</span>
        <span class="text-xs text-gray-500">(Bge-M3 1024-dim, puerto 5001)</span>
      </div>
    </div>

    <!-- API Status -->
    <div class="bg-dark-800 p-6 rounded-lg">
      <h2 class="text-lg font-semibold mb-4">API</h2>
      <div class="flex items-center gap-3">
        <div :class="['w-3 h-3 rounded-full', apiStatus ? 'bg-green-400' : 'bg-red-400']" />
        <span class="text-sm">{{ apiStatus ? 'Conectado' : 'Desconectado' }}</span>
        <span class="text-xs text-gray-500">(puerto 3000)</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import axios from 'axios'

const API_BASE = import.meta.env.VITE_API_URL || ''

const llmStatus = ref({ loaded: false, backend: 'http', state: 'unloaded' })
const embedderStatus = ref(false)
const apiStatus = ref(false)
const llmLoading = ref(false)
const llmMessage = ref('')
const llmError = ref(false)

const fetchStatus = async () => {
  try {
    const token = localStorage.getItem('alesys_token')
    const headers = token ? { Authorization: `Bearer ${token}` } : {}

    const llmRes = await axios.get(`${API_BASE}/api/v1/llm/status`, { headers })
    llmStatus.value = llmRes.data

    const healthRes = await axios.get(`${API_BASE}/health`, { headers })
    apiStatus.value = healthRes.data?.status === 'ok'
    embedderStatus.value = healthRes.data?.embedder === true
  } catch {
    apiStatus.value = false
    embedderStatus.value = false
  }
}

const loadLLM = async () => {
  llmLoading.value = true
  llmMessage.value = ''
  try {
    const token = localStorage.getItem('alesys_token')
    const res = await axios.post(`${API_BASE}/api/v1/llm/load`, {}, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    })
    llmMessage.value = res.data.message || 'Modelo cargado'
    llmError.value = false
    await fetchStatus()
  } catch (err: any) {
    llmMessage.value = err.response?.data?.message || 'Error al cargar modelo'
    llmError.value = true
  } finally {
    llmLoading.value = false
  }
}

const unloadLLM = async () => {
  llmLoading.value = true
  llmMessage.value = ''
  try {
    const token = localStorage.getItem('alesys_token')
    await axios.post(`${API_BASE}/api/v1/llm/unload`, {}, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    })
    llmMessage.value = 'Modelo descargado'
    llmError.value = false
    await fetchStatus()
  } catch (err: any) {
    llmMessage.value = err.response?.data?.message || 'Error al descargar modelo'
    llmError.value = true
  } finally {
    llmLoading.value = false
  }
}

onMounted(() => {
  fetchStatus()
  setInterval(fetchStatus, 10000)
})
</script>

<style scoped>
.bg-dark-900 { background-color: #0f172a; }
.bg-dark-800 { background-color: #1e293b; }
.text-primary-400 { color: #60a5fa; }
.bg-primary-500 { background-color: #3b82f6; }
.bg-primary-500:hover { background-color: #60a5fa; }
</style>
