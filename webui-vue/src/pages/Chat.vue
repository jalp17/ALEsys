<template>
  <div class="flex flex-col h-full">
    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      <div v-for="(msg, i) in messages" :key="i" :class="['flex', msg.role === 'user' ? 'justify-end' : 'justify-start']">
        <div :class="['max-w-3xl rounded-lg px-4 py-3', msg.role === 'user' ? 'bg-primary-500 text-white' : 'bg-dark-800 text-gray-100']">
          <div class="text-sm whitespace-pre-wrap">{{ msg.content }}</div>
          <div v-if="msg.sources && msg.sources.length" class="mt-2 text-xs text-gray-400">
            <span class="font-semibold">Fuentes:</span>
            <span v-for="(s, j) in msg.sources" :key="j" class="ml-1">{{ s.path }} ({{ (s.similarity * 100).toFixed(1) }}%){{ j < msg.sources.length - 1 ? ',' : '' }}</span>
          </div>
        </div>
      </div>
      <div v-if="streaming" class="flex justify-start">
        <div class="bg-dark-800 rounded-lg px-4 py-3">
          <div class="text-sm whitespace-pre-wrap">{{ streamingContent }}</div>
          <Loader2 class="h-4 w-4 animate-spin text-primary-400 mt-1" />
        </div>
      </div>
    </div>

    <div class="border-t border-gray-800 p-4">
      <div class="flex gap-2">
        <input
          v-model="query"
          @keyup.enter="sendMessage"
          type="text"
          placeholder="Escribe tu pregunta..."
          class="flex-1 bg-dark-900 border border-gray-700 rounded-lg px-4 py-2 text-sm focus:outline-none focus:border-primary-500"
          :disabled="loading"
        />
        <button
          @click="sendMessage"
          :disabled="loading || !query.trim()"
          class="px-4 py-2 bg-primary-500 hover:bg-primary-400 text-white rounded-lg transition disabled:opacity-50"
        >
          <Send class="h-4 w-4" />
        </button>
      </div>
      <div class="flex items-center gap-2 mt-2 text-xs text-gray-500">
        <span :class="['w-2 h-2 rounded-full', wsConnected ? 'bg-green-400' : 'bg-gray-500']" />
        {{ wsConnected ? 'WebSocket conectado' : 'HTTP mode' }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useWebSocket } from '../composables/useWebSocket'
import { Loader2, Send } from 'lucide-vue-next'

interface Source {
  fragment_id: number
  path: string
  similarity: number
}

interface Message {
  role: 'user' | 'assistant'
  content: string
  sources?: Source[]
}

const messages = ref<Message[]>([])
const query = ref('')
const loading = ref(false)
const streaming = ref(false)
const streamingContent = ref('')
const sessionId = ref<string | null>(null)
const wsConnected = ref(false)

const API_BASE = import.meta.env.VITE_API_URL || ''
const WS_URL = API_BASE ? `${API_BASE.replace('https:', 'wss:').replace('http:', 'ws:')}/ws/chat` : '/ws/chat'

const { connected, send } = useWebSocket(WS_URL, (data: any) => {
  wsConnected.value = true
  if (data.type === 'start') {
    streaming.value = true
    streamingContent.value = ''
  } else if (data.type === 'chunk') {
    streamingContent.value += data.content || ''
  } else if (data.type === 'done') {
    streaming.value = false
    messages.value.push({
      role: 'assistant',
      content: streamingContent.value,
      sources: data.sources || [],
    })
    streamingContent.value = ''
    loading.value = false
  } else if (data.type === 'error') {
    streaming.value = false
    messages.value.push({
      role: 'assistant',
      content: `Error: ${data.error || 'Error desconocido'}`,
    })
    loading.value = false
  }
})

// Watch connected status
import { watch } from 'vue'
watch(connected, (val) => { wsConnected.value = val })

const sendMessage = async () => {
  if (!query.value.trim() || loading.value) return

  const userMsg = query.value.trim()
  messages.value.push({ role: 'user', content: userMsg })
  query.value = ''
  loading.value = true

  if (connected.value) {
    // Use WebSocket
    send({ type: 'chat', query: userMsg, session_id: sessionId.value })
  } else {
    // Fallback to HTTP
    try {
      const token = localStorage.getItem('alesys_token')
      const res = await fetch(`${API_BASE}/api/v1/chat`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify({ query: userMsg, session_id: sessionId.value }),
      })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await res.json()
      messages.value.push({
        role: 'assistant',
        content: data.response || data.content || 'Sin respuesta',
        sources: data.sources || [],
      })
      if (data.session_id) sessionId.value = data.session_id
    } catch (err: any) {
      messages.value.push({
        role: 'assistant',
        content: `Error: ${err.message || 'Error desconocido'}`,
      })
    } finally {
      loading.value = false
    }
  }
}

onMounted(() => {
  // Auto-scroll to bottom
})
</script>

<style scoped>
.bg-dark-900 { background-color: #0f172a; }
.bg-dark-800 { background-color: #1e293b; }
.text-primary-400 { color: #60a5fa; }
.bg-primary-500 { background-color: #3b82f6; }
.bg-primary-500:hover { background-color: #60a5fa; }
</style>
