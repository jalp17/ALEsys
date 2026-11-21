<template>
  <div class="relative">
    <button
      @click="expanded = !expanded"
      :class="[
        'flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium transition border',
        activeJobs.length > 0
          ? 'bg-blue-900/40 text-blue-400 border-blue-700/50 hover:bg-blue-900/60'
          : 'bg-gray-800/40 text-gray-400 border-gray-700/50 hover:bg-gray-800/60',
      ]"
      :title="activeJobs.length > 0 ? activeJobs.length + ' tarea(s) activa(s)' : 'Tareas completadas'"
      aria-label="Monitor de tareas de indexación"
    >
      <Loader2 v-if="activeJobs.length > 0" class="h-3 w-3 animate-spin" />
      <CheckCircle v-else class="h-3 w-3" />
      <span>Tareas {{ activeJobs.length > 0 ? '(' + activeJobs.length + ')' : '' }}</span>
      <ChevronUp v-if="expanded" class="h-3 w-3" />
      <ChevronDown v-else class="h-3 w-3" />
    </button>

    <div
      v-if="expanded"
      class="absolute right-0 top-full mt-2 w-96 max-h-96 overflow-y-auto rounded-lg border border-gray-700 bg-dark-800 shadow-xl z-50"
      role="dialog"
      aria-label="Panel de tareas"
    >
      <div class="p-3 border-b border-gray-700">
        <h3 class="text-sm font-semibold text-gray-200">Tareas de Indexación</h3>
        <p class="text-xs text-gray-500">{{ activeJobs.length }} activa(s), {{ completedJobs.length }} completada(s), {{ failedJobs.length }} fallida(s)</p>
      </div>

      <div v-if="jobs.length === 0" class="p-4 text-center text-gray-500 text-sm">
        No hay tareas
      </div>

      <div v-else class="divide-y divide-gray-700/50">
        <div
          v-for="job in jobs"
          :key="job.job_id"
          class="p-3 hover:bg-dark-700/50"
        >
          <div class="flex items-center gap-2 mb-1">
            <component :is="getStatusIcon(job.status)" :class="['h-3 w-3', getStatusColor(job.status)]" />
            <span :class="['text-xs font-medium', getStatusColor(job.status)]">
              {{ job.status }}
            </span>
            <span class="text-xs text-gray-500 ml-auto">
              {{ new Date(job.created_at).toLocaleTimeString() }}
            </span>
          </div>

          <div class="text-xs text-gray-400 truncate" :title="job.dir">
            {{ job.dir.split('/').pop() || job.dir }}
          </div>

          <div v-if="job.status === 'processing' || job.status === 'queued'" class="mt-2">
            <div class="flex justify-between text-[10px] text-gray-500 mb-1">
              <span>{{ job.files_done }} / {{ job.files_total }} archivos</span>
              <span>{{ job.chunks_indexed }} chunks</span>
            </div>
            <div class="w-full bg-dark-700 rounded-full h-1.5">
              <div
                class="bg-blue-400 h-1.5 rounded-full transition-all duration-300"
                :style="{ width: Math.min(job.progress, 100) + '%' }"
              />
            </div>
            <p v-if="job.current_topic" class="text-[10px] text-gray-500 mt-1 truncate">
              Topic: {{ job.current_topic }}
            </p>
          </div>

          <div v-if="job.status === 'completed'" class="text-[10px] text-green-400 mt-1">
            {{ job.files_done }} archivos, {{ job.chunks_indexed }} chunks indexados
          </div>

          <div v-if="job.error" class="text-[10px] text-red-400 mt-1 truncate">
            {{ job.error }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import axios from 'axios'
import { Loader2, CheckCircle, XCircle, Clock, ChevronDown, ChevronUp } from 'lucide-vue-next'

const API_BASE = import.meta.env.VITE_API_URL || ''

interface Job {
  job_id: string
  dir: string
  status: string
  progress: number
  files_done: number
  files_total: number
  chunks_indexed: number
  current_topic?: string
  message?: string
  error?: string
  created_at: string
}

const jobs = ref<Job[]>([])
const expanded = ref(false)

const activeJobs = computed(() => jobs.value.filter(j => j.status === 'processing' || j.status === 'queued' || j.status === 'cancelling'))
const completedJobs = computed(() => jobs.value.filter(j => j.status === 'completed'))
const failedJobs = computed(() => jobs.value.filter(j => j.status === 'failed' || j.status === 'cancelled'))

const fetchJobs = async () => {
  try {
    const token = localStorage.getItem('alesys_token')
    const res = await axios.get<{ jobs: Job[] }>(`${API_BASE}/api/v1/ingestion/markdown/jobs`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    })
    jobs.value = res.data.jobs || []
  } catch {
    // Silently ignore
  }
}

const getStatusIcon = (status: string) => {
  switch (status) {
    case 'processing':
    case 'queued':
      return Loader2
    case 'completed':
      return CheckCircle
    case 'failed':
    case 'cancelled':
      return XCircle
    default:
      return Clock
  }
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'processing':
    case 'queued':
      return 'text-blue-400'
    case 'completed':
      return 'text-green-400'
    case 'failed':
    case 'cancelled':
      return 'text-red-400'
    default:
      return 'text-gray-400'
  }
}

onMounted(() => {
  fetchJobs()
  const interval = setInterval(fetchJobs, 5000)
  onUnmounted(() => clearInterval(interval))
})
</script>

<style scoped>
.bg-dark-800 { background-color: #1e293b; }
.bg-dark-700 { background-color: #334155; }
.text-primary-400 { color: #60a5fa; }
</style>
