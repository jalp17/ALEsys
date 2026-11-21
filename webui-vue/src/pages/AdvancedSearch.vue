<template>
  <div class="max-w-4xl mx-auto p-6">
    <h1 class="text-2xl font-bold mb-6">Búsqueda Avanzada</h1>
    <div class="bg-dark-800 p-6 rounded-lg mb-6">
      <div class="flex gap-2 mb-4">
        <input v-model="query" type="text" placeholder="Buscar en el conocimiento..." class="flex-1 bg-dark-900 border border-gray-700 rounded px-3 py-2 text-sm focus:outline-none focus:border-primary-500" @keyup.enter="search" />
        <button @click="search" :disabled="loading" class="px-4 py-2 bg-primary-500 hover:bg-primary-400 text-white text-sm rounded transition disabled:opacity-50">
          {{ loading ? 'Buscando...' : 'Buscar' }}
        </button>
      </div>
      <div v-if="results.length" class="space-y-3">
        <div v-for="(r, i) in results" :key="i" class="bg-dark-900 rounded p-3 border border-gray-700">
          <p class="text-sm text-gray-200">{{ r.text || r.content }}</p>
          <div class="flex gap-2 mt-2 text-xs text-gray-500">
            <span v-if="r.topic">{{ r.topic }}</span>
            <span v-if="r.score">Score: {{ (r.score * 100).toFixed(1) }}%</span>
          </div>
        </div>
      </div>
      <p v-if="!loading && results.length === 0 && searched" class="text-gray-500 text-sm text-center py-4">Sin resultados</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import axios from 'axios'
const API_BASE = import.meta.env.VITE_API_URL || ''
const query = ref('')
const loading = ref(false)
const searched = ref(false)
const results = ref<any[]>([])

const search = async () => {
  if (!query.value.trim()) return
  loading.value = true
  searched.value = true
  try {
    const token = localStorage.getItem('alesys_token')
    const res = await axios.post(`${API_BASE}/api/v1/search/advanced`, { query: query.value, limit: 20 }, { headers: token ? { Authorization: `Bearer ${token}` } : {} })
    results.value = res.data.results || res.data || []
  } catch { results.value = [] }
  finally { loading.value = false }
}
</script>

<style scoped>
.bg-dark-900 { background-color: #0f172a; }
.bg-dark-800 { background-color: #1e293b; }
.text-primary-400 { color: #60a5fa; }
.bg-primary-500 { background-color: #3b82f6; }
.bg-primary-500:hover { background-color: #60a5fa; }
</style>
