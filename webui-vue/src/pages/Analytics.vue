<template>
  <div class="max-w-4xl mx-auto p-6">
    <h1 class="text-2xl font-bold mb-6">Analytics</h1>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
      <div class="bg-dark-800 p-4 rounded-lg">
        <p class="text-gray-500 text-sm">Documentos</p>
        <p class="text-2xl font-bold">{{ stats.documents }}</p>
      </div>
      <div class="bg-dark-800 p-4 rounded-lg">
        <p class="text-gray-500 text-sm">Chunks</p>
        <p class="text-2xl font-bold">{{ stats.chunks }}</p>
      </div>
      <div class="bg-dark-800 p-4 rounded-lg">
        <p class="text-gray-500 text-sm">Topics</p>
        <p class="text-2xl font-bold">{{ stats.topics }}</p>
      </div>
    </div>
    <div class="bg-dark-800 p-6 rounded-lg">
      <p class="text-gray-400 text-sm">Métricas de uso y rendimiento (pendiente de implementación completa)</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import axios from 'axios'
const API_BASE = import.meta.env.VITE_API_URL || ''
const stats = ref({ documents: 0, chunks: 0, topics: 0 })

onMounted(async () => {
  try {
    const token = localStorage.getItem('alesys_token')
    const res = await axios.get(`${API_BASE}/api/v1/analytics/usage`, { headers: token ? { Authorization: `Bearer ${token}` } : {} })
    if (res.data) {
      stats.value.documents = res.data.documents || 0
      stats.value.chunks = res.data.chunks || 0
      stats.value.topics = res.data.topics || 0
    }
  } catch { /* noop */ }
})
</script>

<style scoped>
.bg-dark-800 { background-color: #1e293b; }
</style>
