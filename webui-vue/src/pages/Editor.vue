<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between p-4 border-b border-gray-800">
      <h1 class="text-xl font-bold">Editor IDE</h1>
      <div class="flex gap-2">
        <select v-model="language" class="bg-dark-900 border border-gray-700 rounded px-2 py-1 text-sm">
          <option value="javascript">JavaScript</option>
          <option value="typescript">TypeScript</option>
          <option value="python">Python</option>
          <option value="rust">Rust</option>
          <option value="html">HTML</option>
          <option value="css">CSS</option>
          <option value="json">JSON</option>
          <option value="markdown">Markdown</option>
        </select>
        <button @click="runCode" class="px-3 py-1.5 bg-green-600 hover:bg-green-500 text-white text-sm rounded transition">
          Ejecutar
        </button>
      </div>
    </div>
    <div class="flex-1 overflow-hidden">
      <div ref="editorContainer" class="w-full h-full"></div>
    </div>
    <div v-if="output" class="border-t border-gray-800 p-4 max-h-48 overflow-y-auto">
      <h3 class="text-sm font-semibold mb-2">Salida</h3>
      <pre class="text-xs text-gray-300 font-mono whitespace-pre-wrap">{{ output }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import * as monaco from 'monaco-editor'

const editorContainer = ref<HTMLDivElement | null>(null)
const language = ref('javascript')
const output = ref('')
let editor: any = null

onMounted(() => {
  if (!editorContainer.value) return
  editor = monaco.editor.create(editorContainer.value, {
    value: '// Escribe tu código aquí\nconsole.log("Hello, ALEsys!");',
    language: language.value,
    theme: 'vs-dark',
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 14,
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    wordWrap: 'on',
  })
})

onUnmounted(() => {
  editor?.dispose()
})

const runCode = () => {
  const code = editor?.getValue() || ''
  output.value = `// Ejecución de ${language.value}\n// (requiere backend de ejecución)\n${code}`
}
</script>

<style scoped>
.bg-dark-900 { background-color: #0f172a; }
</style>
