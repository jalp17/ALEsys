<template>
  <div class="max-w-4xl mx-auto p-6">
    <h1 class="text-2xl font-bold mb-6">Sesiones</h1>

    <div v-if="loading" class="flex justify-center py-8">
      <Loader2 class="h-6 w-6 animate-spin text-primary-400" />
    </div>

    <div v-else-if="sessions.length === 0" class="text-center py-8 text-gray-500">
      No hay sesiones activas
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="session in sessions"
        :key="session.id"
        class="bg-dark-800 rounded-lg p-4 hover:bg-dark-700 transition cursor-pointer"
        @click="openSession(session.id)"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium text-gray-200">{{ session.title || 'Sin título' }}</span>
          <span class="text-xs text-gray-500">{{ formatDate(session.created_at) }}</span>
        </div>
        <p class="text-xs text-gray-400 truncate">{{ session.last_message || 'Sin mensajes' }}</p>
        <div class="flex items-center gap-2 mt-2">
          <span class="text-[10px] text-gray-500">{{ session.message_count || 0 }} mensajes</span>
          <span v-if="session.topic" class="text-[10px] bg-primary-500/20 text-primary-300 px-1.5 rounded">
            {{ session.topic }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import axios from 'axios'
import { Loader2 } from 'lucide-vue-next'
import { useRouter } from 'vue-router'

const API_BASE = import.meta.env.VITE_API_URL || ''
const router = useRouter()

interface Session {
  id: string
  title: string
  created_at: string
  last_message: string
  message_count: number
  topic: string
}

const sessions = ref<Session[]>([])
const loading = ref(true)

const fetchSessions = async () => {
  loading.value = true
  try {
    const token = localStorage.getItem('alesys_token')
    const res = await axios.get(`${API_BASE}/api/v1/sessions`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    })
    sessions.value = res.data.sessions || res.data || []
  } catch {
    sessions.value = []
  } finally {
    loading.value = false
  }
}

const openSession = (id: string) => {
  router.push(`/chat?session=${id}`)
}

const formatDate = (date: string) => {
  if (!date) return ''
  return new Date(date).toLocaleDateString('es-ES', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' })
}

onMounted(fetchSessions)
</script>

<style scoped>
.bg-dark-800 { background-color: #1e293b; }
.bg-dark-700 { background-color: #334155; }
.text-primary-400 { color: #60a5fa; }
.text-primary-300 { color: #93c5fd; }
.bg-primary-500\/20 { background-color: rgba(59, 130, 246, 0.2); }
</style>
