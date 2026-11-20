<template>
  <div class="flex h-screen overflow-hidden bg-dark-900 text-gray-100">
    <!-- Nav Rail -->
    <nav class="flex w-14 shrink-0 flex-col items-center border-r border-gray-800 bg-dark-800 py-3">
      <button
        @click="navigate('/workspaces')"
        class="mb-4 flex h-9 w-9 items-center justify-center rounded-lg bg-primary-500 text-white"
        title="ALEsys — Inicio"
      >
        <Sparkles class="h-5 w-5" />
      </button>

      <div class="flex flex-1 flex-col gap-1">
        <button
          v-for="g in groups"
          :key="g.id"
          @click="activeGroup = g.id"
          :title="g.label"
          :class="[
            'group relative flex h-10 w-10 items-center justify-center rounded-lg transition',
            activeGroup === g.id ? 'bg-primary-500/20 text-primary-400' : 'text-gray-400 hover:bg-gray-700/50 hover:text-gray-200',
          ]"
        >
          <component :is="g.icon" class="h-5 w-5" />
          <span v-if="activeGroup === g.id" class="absolute left-0 h-5 w-0.5 rounded-r bg-primary-400" />
          <span class="pointer-events-none absolute left-12 z-50 whitespace-nowrap rounded bg-gray-900 px-2 py-1 text-xs text-gray-200 opacity-0 shadow-lg group-hover:opacity-100">
            {{ g.label }}
          </span>
        </button>
      </div>

      <button
        @click="cmdOpen = true"
        title="Comandos (⌘K)"
        class="mb-2 flex h-10 w-10 items-center justify-center rounded-lg text-gray-400 hover:bg-gray-700/50 hover:text-gray-200"
      >
        <Command class="h-5 w-5" />
      </button>
    </nav>

    <!-- Section Panel -->
    <aside
      :class="[
        'flex shrink-0 flex-col border-r border-gray-800 bg-dark-800 transition-all',
        collapsed ? 'w-0 overflow-hidden' : 'w-60',
      ]"
    >
      <div class="flex h-12 items-center justify-between border-b border-gray-800 px-4">
        <div class="flex items-center gap-2">
          <component :is="activeGroupData.icon" class="h-4 w-4 text-primary-400" />
          <span class="text-sm font-semibold text-gray-200">{{ activeGroupData.label }}</span>
        </div>
        <button @click="collapsed = true" class="text-gray-500 hover:text-gray-300" title="Colapsar panel">
          <ChevronLeft class="h-4 w-4" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-2">
        <div class="mb-2 px-2 text-[10px] font-semibold uppercase tracking-wider text-gray-500">
          Entornos
        </div>
        <router-link
          v-for="item in activeGroupData.items"
          :key="item.to"
          :to="item.to"
          :class="[
            'mb-0.5 block rounded-lg px-2 py-2 transition',
            isActive(item.to) ? 'bg-primary-500/15 text-primary-300' : 'text-gray-300 hover:bg-gray-700/40',
          ]"
        >
          <div class="flex items-center gap-2">
            <component :is="item.icon" :class="['h-4 w-4', isActive(item.to) ? 'text-primary-400' : 'text-gray-400']" />
            <span class="text-sm">{{ item.label }}</span>
          </div>
          <div v-if="item.description && isActive(item.to)" class="mt-0.5 pl-6 text-[11px] text-gray-500">
            {{ item.description }}
          </div>
        </router-link>
      </div>

      <div class="border-t border-gray-800 p-3">
        <div class="mb-1 flex items-center justify-between text-[11px] text-gray-500">
          <span>Modelo activo</span>
          <Cpu class="h-3 w-3" />
        </div>
        <select
          v-model="model"
          @change="onModelChange"
          class="w-full rounded bg-dark-900 px-2 py-1 text-xs text-gray-300 border border-gray-700 focus:outline-none focus:border-primary-500"
        >
          <option v-for="m in modelOptions" :key="m.id" :value="m.id">{{ m.label }}</option>
        </select>
      </div>
    </aside>

    <button
      v-if="collapsed"
      @click="collapsed = false"
      class="absolute left-14 top-3 z-10 rounded bg-dark-800 p-1 text-gray-400 hover:text-gray-200"
      title="Expandir panel"
    >
      <ChevronRight class="h-4 w-4" />
    </button>

    <!-- Main -->
    <main class="flex min-w-0 flex-1 flex-col">
      <header class="flex h-12 shrink-0 items-center gap-3 border-b border-gray-800 bg-dark-800/60 px-4">
        <nav class="flex items-center gap-1.5 text-xs text-gray-500">
          <span>ALEsys</span>
          <ChevronRight class="h-3 w-3" />
          <span>{{ activeGroupData.label }}</span>
          <template v-if="currentItem">
            <ChevronRight class="h-3 w-3" />
            <span class="text-gray-300">{{ currentItem.label }}</span>
          </template>
        </nav>

        <div class="ml-auto flex items-center gap-3">
          <button
            @click="cmdOpen = true"
            class="flex items-center gap-1.5 rounded border border-gray-700 px-2 py-1 text-[11px] text-gray-400 hover:border-gray-500 hover:text-gray-300"
          >
            <Command class="h-3 w-3" />
            <span class="hidden sm:inline">Comandos</span>
            <kbd class="rounded bg-gray-700 px-1 text-[9px]">⌘K</kbd>
          </button>
          <TaskMonitor />
        </div>
      </header>

      <div class="flex min-h-0 flex-1 overflow-hidden">
        <div class="min-w-0 flex-1 overflow-auto">
          <router-view />
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  Sparkles, Cpu, Command, ChevronLeft, ChevronRight,
  NotebookPen, Network, Upload, FileText, GitGraph, Search, Settings,
} from 'lucide-vue-next'
import TaskMonitor from './components/TaskMonitor.vue'

const groups = [
  {
    id: 'research',
    label: 'Investigación',
    icon: NotebookPen,
    items: [
      { to: '/workspaces', label: 'Cuadernos', icon: NotebookPen, description: 'Espacios de trabajo con documentos y chat' },
      { to: '/ingestion', label: 'Ingesta', icon: Upload, description: 'Procesar PDFs y documentos' },
      { to: '/ingestion/history', label: 'Historial', icon: FileText, description: 'Trabajos de ingesta' },
      { to: '/ingestion/markdown', label: 'Markdown', icon: FileText, description: 'Indexar Markdown' },
    ],
  },
  {
    id: 'knowledge',
    label: 'Conocimiento',
    icon: Network,
    items: [
      { to: '/graph', label: 'Grafo', icon: GitGraph, description: 'Visualización GraphRAG' },
      { to: '/search', label: 'Búsqueda', icon: Search, description: 'Búsqueda semántica' },
    ],
  },
  {
    id: 'system',
    label: 'Sistema',
    icon: Settings,
    items: [
      { to: '/settings', label: 'Configuración', icon: Settings, description: 'Modelos y conexiones' },
    ],
  },
]

const collapsed = ref(false)
const activeGroup = ref('research')
const cmdOpen = ref(false)
const model = ref(localStorage.getItem('alesys_model') || 'openrouter/free')

const modelOptions = [
  { id: 'openrouter/free', label: 'OpenRouter · Auto' },
]

const activeGroupData = computed(() => groups.find(g => g.id === activeGroup.value) || groups[0])

const route = useRoute()
const router = useRouter()

const currentItem = computed(() => {
  const path = route.path
  for (const g of groups) {
    for (const item of g.items) {
      if (path === item.to || path.startsWith(item.to + '/')) {
        return item
      }
    }
  }
  return null
})

function isActive(path: string) {
  return route.path === path || route.path.startsWith(path + '/')
}

function navigate(path: string) {
  router.push(path)
}

function onModelChange() {
  localStorage.setItem('alesys_model', model.value)
}

// Sync active group with route
onMounted(() => {
  const g = groups.find(gr => gr.items.some(i => route.path === i.to || route.path.startsWith(i.to + '/')))
  if (g) activeGroup.value = g.id
})
</script>

<style scoped>
.bg-dark-900 { background-color: #0f172a; }
.bg-dark-800 { background-color: #1e293b; }
.bg-dark-700 { background-color: #334155; }
.text-primary-400 { color: #60a5fa; }
.text-primary-300 { color: #93c5fd; }
.bg-primary-500 { background-color: #3b82f6; }
.bg-primary-500\/15 { background-color: rgba(59, 130, 246, 0.15); }
.bg-primary-500\/20 { background-color: rgba(59, 130, 246, 0.2); }
</style>
