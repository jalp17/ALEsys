<template>
  <div
    class="border-2 border-dashed border-gray-700 rounded-lg p-8 text-center transition"
    :class="{ 'border-primary-500 bg-primary-500/10': isDragging }"
    @dragover.prevent="isDragging = true"
    @dragleave.prevent="isDragging = false"
    @drop.prevent="handleDrop"
    @click="fileInput?.click()"
  >
    <Upload class="h-8 w-8 mx-auto mb-2 text-gray-400" />
    <p class="text-sm text-gray-400">
      {{ isDragging ? 'Suelta los archivos aquí' : 'Arrastra archivos o haz clic para seleccionar' }}
    </p>
    <p class="text-xs text-gray-500 mt-1">PDF, Markdown, TXT</p>
    <input
      ref="fileInput"
      type="file"
      multiple
      accept=".pdf,.md,.txt"
      class="hidden"
      @change="handleFileSelect"
    />
  </div>

  <div v-if="files.length" class="mt-4 space-y-2">
    <div v-for="(file, i) in files" :key="i" class="flex items-center gap-2 bg-dark-800 rounded px-3 py-2">
      <FileText class="h-4 w-4 text-gray-400" />
      <span class="text-sm text-gray-300 flex-1 truncate">{{ file.name }}</span>
      <span class="text-xs text-gray-500">{{ formatSize(file.size) }}</span>
      <button @click="removeFile(i)" class="text-gray-500 hover:text-red-400">
        <X class="h-4 w-4" />
      </button>
    </div>
    <button
      @click="uploadFiles"
      :disabled="uploading"
      class="w-full px-4 py-2 bg-primary-500 hover:bg-primary-400 text-white rounded-lg transition disabled:opacity-50"
    >
      {{ uploading ? 'Subiendo...' : `Subir ${files.length} archivo(s)` }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Upload, FileText, X } from 'lucide-vue-next'

interface Props {
  onUpload?: (files: File[]) => void
}

const props = defineProps<Props>()

const fileInput = ref<HTMLInputElement | null>(null)
const files = ref<File[]>([])
const isDragging = ref(false)
const uploading = ref(false)

const handleDrop = (e: DragEvent) => {
  isDragging.value = false
  if (e.dataTransfer?.files) {
    files.value.push(...Array.from(e.dataTransfer.files))
  }
}

const handleFileSelect = (e: Event) => {
  const target = e.target as HTMLInputElement
  if (target.files) {
    files.value.push(...Array.from(target.files))
  }
}

const removeFile = (index: number) => {
  files.value.splice(index, 1)
}

const formatSize = (bytes: number) => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

const uploadFiles = async () => {
  if (!files.value.length) return
  uploading.value = true
  try {
    if (props.onUpload) {
      props.onUpload(files.value)
    }
    files.value = []
  } finally {
    uploading.value = false
  }
}
</script>

<style scoped>
.bg-dark-800 { background-color: #1e293b; }
.text-primary-400 { color: #60a5fa; }
.bg-primary-500 { background-color: #3b82f6; }
.bg-primary-500:hover { background-color: #60a5fa; }
.border-primary-500 { border-color: #3b82f6; }
.bg-primary-500\/10 { background-color: rgba(59, 130, 246, 0.1); }
</style>
