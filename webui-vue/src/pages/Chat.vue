<template>
  <div class="flex flex-col h-full">
    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      <div v-for="(msg, i) in messages" :key="i" :class="['flex', msg.role === 'user' ? 'justify-end' : 'justify-start']">
        <div :class="['max-w-3xl rounded-lg px-4 py-3', msg.role === 'user' ? 'bg-primary-500 text-white' : 'bg-dark-800 text-gray-100']">
          <div class="text-sm whitespace-pre-wrap">{{ msg.content }}</div>
          <div v-if="msg.sources && msg.sources.length" class="mt-2 text-xs text-gray-400">
            <span class="font-semibold">Fuentes:</span>
            <span v-for="(s, j) in msg.sources" :key="j" class="ml-1">{{ s }}{{ j < msg.sources.length - 1 ? ',' : '' }}</span>
          </div>
        </div>
      </div>
      <div v-if="loading" class="flex justify-start">
        <div class="bg-dark-800 rounded-lg px-4 py-3">
          <Loader2 class="h-5 w-5 animate-spin text-primary-400" />
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import axios from 'axios'
import { Loader2, Send } from 'lucide-vue-next'

const API_BASE = import.meta.env.VITE_API_URL || ''

interface Message {
  role: 'user' | 'assistant'
  content: string
  sources?: string[]
}

const messages = ref<Message[]>([])
const query = ref('')
const loading = ref(false)
const sessionId = ref<string | null>(null)

const sendMessage = async () => {
  if (!query.value.trim() || loading.value) return

  const userMsg = query.value.trim()
  messages.value.push({ role: 'user', content: userMsg })
  query.value = ''
  loading.value = true

  try {
    const token = localStorage.getItem('alesys_token')
    const res = await axios.post(`${API_BASE}/api/v1/chat`, {
      query: userMsg,
      session_id: sessionId.value,
    }, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      timeout: 120000,
    })

    messages.value.push({
      role: 'assistant',
      content: res.data.response || res.data.content || 'Sin respuesta',
      sources: res.data.sources || [],
    })

    if (res.data.session_id) {
      sessionId.value = res.data.session_id
    }
  } catch (err: any) {
    messages.value.push({
      role: 'assistant',
      content: `Error: ${err.response?.data?.error || err.message || 'Error desconocido'}`,
    })
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.bg-dark-900 { background-color: #0f172a; }
.bg-dark-800 { background-color: #1e293b; }
.text-primary-400 { color: #60a5fa; }
.bg-primary-500 { background-color: #3b82f6; }
.bg-primary-500:hover { background-color: #60a5fa; }
</style>
