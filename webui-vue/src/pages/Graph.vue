<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between p-4 border-b border-gray-800">
      <h1 class="text-xl font-bold">Grafo de Conocimiento</h1>
      <div class="flex gap-2">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Buscar nodos..."
          class="bg-dark-900 border border-gray-700 rounded px-3 py-1.5 text-sm focus:outline-none focus:border-primary-500"
        />
        <button
          @click="loadGraph"
          class="px-3 py-1.5 bg-primary-500 hover:bg-primary-400 text-white text-sm rounded transition"
        >
          Cargar
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-hidden relative">
      <GraphCanvas :nodes="nodes" :edges="edges" :loading="loading" />

      <!-- Node detail panel -->
      <div v-if="selectedNode" class="absolute right-4 top-4 w-72 bg-dark-800 rounded-lg p-4 shadow-xl border border-gray-700">
        <h3 class="text-sm font-semibold text-gray-200 mb-2">{{ selectedNode.label }}</h3>
        <div class="text-xs text-gray-400 space-y-1">
          <p><span class="text-gray-500">Tipo:</span> {{ selectedNode.type }}</p>
          <p><span class="text-gray-500">ID:</span> {{ selectedNode.id }}</p>
          <p v-if="selectedNode.connections"><span class="text-gray-500">Conexiones:</span> {{ selectedNode.connections }}</p>
        </div>
        <button
          @click="selectedNode = null"
          class="mt-3 text-xs text-gray-500 hover:text-gray-300"
        >
          Cerrar
        </button>
      </div>
    </div>

    <!-- Stats bar -->
    <div class="border-t border-gray-800 px-4 py-2 flex gap-4 text-xs text-gray-500">
      <span>Nodos: {{ nodes.length }}</span>
      <span>Aristas: {{ edges.length }}</span>
      <span v-if="searchQuery">Filtrado: {{ filteredNodes.length }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import axios from 'axios'
import GraphCanvas from '../components/GraphCanvas.vue'

const API_BASE = import.meta.env.VITE_API_URL || ''

interface GraphNode {
  id: string
  label: string
  type: string
  connections?: number
}

interface GraphEdge {
  source: string
  target: string
  type: string
}

const loading = ref(false)
const nodes = ref<GraphNode[]>([])
const edges = ref<GraphEdge[]>([])
const selectedNode = ref<GraphNode | null>(null)
const searchQuery = ref('')

const filteredNodes = computed(() => {
  if (!searchQuery.value) return nodes.value
  const q = searchQuery.value.toLowerCase()
  return nodes.value.filter(n => n.label.toLowerCase().includes(q) || n.type.toLowerCase().includes(q))
})

const loadGraph = async () => {
  loading.value = true
  try {
    const token = localStorage.getItem('alesys_token')
    // Try to fetch graph data from API
    const res = await axios.get(`${API_BASE}/api/v1/graph/nodes`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      params: { limit: 500 },
    })
    nodes.value = res.data.nodes || []
    edges.value = res.data.edges || []
  } catch (err: any) {
    // Fallback: show placeholder
    console.error('Graph load error:', err)
    nodes.value = []
    edges.value = []
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  // Auto-load graph on mount
  loadGraph()
})
</script>

<style scoped>
.bg-dark-900 { background-color: #0f172a; }
.bg-dark-800 { background-color: #1e293b; }
.text-primary-400 { color: #60a5fa; }
</style>
