<template>
  <div class="max-w-4xl mx-auto p-6">
    <h1 class="text-2xl font-bold mb-6">KB Curation</h1>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
      <div class="bg-dark-800 p-4 rounded-lg">
        <h3 class="text-sm font-semibold mb-2">Fusionar Documentos</h3>
        <p class="text-xs text-gray-400 mb-3">Combina múltiples documentos en uno solo</p>
        <button class="px-3 py-1.5 bg-primary-500 hover:bg-primary-400 text-white text-sm rounded transition">
          Fusionar
        </button>
      </div>
      <div class="bg-dark-800 p-4 rounded-lg">
        <h3 class="text-sm font-semibold mb-2">Dividir Documento</h3>
        <p class="text-xs text-gray-400 mb-3">Parte un documento en secciones</p>
        <button class="px-3 py-1.5 bg-primary-500 hover:bg-primary-400 text-white text-sm rounded transition">
          Dividir
        </button>
      </div>
      <div class="bg-dark-800 p-4 rounded-lg">
        <h3 class="text-sm font-semibold mb-2">Detectar Duplicados</h3>
        <p class="text-xs text-gray-400 mb-3">Encuentra documentos similares</p>
        <button class="px-3 py-1.5 bg-primary-500 hover:bg-primary-400 text-white text-sm rounded transition">
          Detectar
        </button>
      </div>
      <div class="bg-dark-800 p-4 rounded-lg">
        <h3 class="text-sm font-semibold mb-2">Archivar</h3>
        <p class="text-xs text-gray-400 mb-3">Mueve documentos a archivo</p>
        <button class="px-3 py-1.5 bg-primary-500 hover:bg-primary-400 text-white text-sm rounded transition">
          Archivar
        </button>
      </div>
    </div>

    <div class="bg-dark-800 p-6 rounded-lg">
      <h3 class="text-sm font-semibold mb-3">Calidad de Documentos</h3>
      <p class="text-xs text-gray-400">Evalúa y mejora la calidad de los documentos indexados</p>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
.bg-dark-800 { background-color: #1e293b; }
.text-primary-400 { color: #60a5fa; }
.bg-primary-500 { background-color: #3b82f6; }
.bg-primary-500:hover { background-color: #60a5fa; }
</style>
