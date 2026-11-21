<template>
  <div class="max-w-4xl mx-auto p-6">
    <h1 class="text-2xl font-bold mb-6">Ingesta de Documentos</h1>

    <div class="bg-dark-800 p-6 rounded-lg mb-6">
      <h2 class="text-lg font-semibold mb-4">Ingesta de Directorio (Markdown)</h2>
      <div class="mb-4">
        <label class="block text-sm font-medium mb-1">Directorio raíz</label>
        <input
          v-model="dir"
          type="text"
          placeholder="/mnt/ws2/rescate_sdb/libros/datos_db_p_procesado/books"
          class="w-full bg-dark-900 border border-gray-700 rounded px-3 py-2 font-mono text-sm"
        />
      </div>
      <div class="flex gap-4">
        <button
          @click="startIngestion"
          :disabled="loading || !dir"
          class="px-6 py-2 bg-primary-500 hover:bg-primary-400 text-white font-medium rounded transition disabled:opacity-50"
        >
          {{ loading ? 'Iniciando...' : 'Iniciar Ingesta' }}
        </button>
      </div>
      <p v-if="error" class="text-red-400 text-sm mt-4">{{ error }}</p>
      <p v-if="result" class="text-green-400 text-sm mt-4">{{ result }}</p>
    </div>

    <div class="bg-dark-800 p-6 rounded-lg">
      <h2 class="text-lg font-semibold mb-4">Subir Archivos</h2>
      <FileUploader @upload="handleUpload" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import axios from 'axios'
import FileUploader from '../components/FileUploader.vue'

const API_BASE = import.meta.env.VITE_API_URL || ''
const dir = ref('')
const loading = ref(false)
const error = ref('')
const result = ref('')

const startIngestion = async () => {
  if (!dir.value) {
    error.value = 'Ingresa un directorio'
    return
  }
  loading.value = true
  error.value = ''
  result.value = ''
  try {
    const token = localStorage.getItem('alesys_token')
    const res = await axios.post(`${API_BASE}/api/v1/ingestion/markdown`, {
      dir: dir.value,
      dry_run: false,
      max_files: 5000,
    }, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    })
    result.value = `Job iniciado: ${res.data.job_id} (${res.data.files_found} archivos)`
  } catch (err: any) {
    error.value = err.response?.data?.error || 'Error al iniciar ingesta'
  } finally {
    loading.value = false
  }
}

const handleUpload = (files: File[]) => {
  console.log('Files to upload:', files)
  // TODO: Implement file upload to API
}
</script>

<style scoped>
.bg-dark-900 { background-color: #0f172a; }
.bg-dark-800 { background-color: #1e293b; }
.text-primary-400 { color: #60a5fa; }
.bg-primary-500 { background-color: #3b82f6; }
.bg-primary-500:hover { background-color: #60a5fa; }
</style>
