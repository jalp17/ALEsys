import { useState } from 'react'
import { Workspace, updateWorkspace } from '../../../services/workspaceService'

interface SettingsPanelProps {
  workspace: Workspace
  onClose: () => void
  onSaved: (ws: Workspace) => void
}

export function SettingsPanel({ workspace, onClose, onSaved }: SettingsPanelProps) {
  const [name, setName] = useState(workspace.name)
  const [description, setDescription] = useState(workspace.description)
  const [llmModel, setLlmModel] = useState(workspace.llm_model || '')
  const [temperature, setTemperature] = useState(workspace.temperature)
  const [topP, setTopP] = useState(workspace.top_p)
  const [systemPrompt, setSystemPrompt] = useState(workspace.system_prompt || '')
  const [embedderMode, setEmbedderMode] = useState(workspace.embedder_mode || 'embed')
  const [saving, setSaving] = useState(false)

  const handleSave = async () => {
    setSaving(true)
    try {
      const updated = await updateWorkspace(workspace.id, {
        name,
        description,
        llm_model: llmModel || null,
        temperature,
        top_p: topP,
        system_prompt: systemPrompt || null,
        embedder_mode: embedderMode,
      })
      onSaved(updated)
    } catch (e) {
      alert('Error guardando configuración')
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div className="w-[480px] max-w-[90vw] rounded-lg border border-gray-700 bg-dark-800 p-4" onClick={(e) => e.stopPropagation()}>
        <h3 className="text-sm font-semibold text-gray-200 mb-4">Configuración del cuaderno</h3>

        <div className="space-y-3">
          <div>
            <label className="text-xs text-gray-400">Nombre</label>
            <input value={name} onChange={(e) => setName(e.target.value)}
              className="w-full rounded bg-dark-900 border border-gray-700 px-2 py-1 text-sm text-gray-200" />
          </div>
          <div>
            <label className="text-xs text-gray-400">Descripción</label>
            <input value={description} onChange={(e) => setDescription(e.target.value)}
              className="w-full rounded bg-dark-900 border border-gray-700 px-2 py-1 text-sm text-gray-200" />
          </div>
          <div>
            <label className="text-xs text-gray-400">Modelo LLM (vacío = global)</label>
            <input value={llmModel} onChange={(e) => setLlmModel(e.target.value)} placeholder="openrouter/free"
              className="w-full rounded bg-dark-900 border border-gray-700 px-2 py-1 text-sm text-gray-200" />
          </div>
          <div className="flex gap-3">
            <div className="flex-1">
              <label className="text-xs text-gray-400">Temperatura ({temperature})</label>
              <input type="range" min="0" max="2" step="0.1" value={temperature} onChange={(e) => setTemperature(Number(e.target.value))}
                className="w-full" />
            </div>
            <div className="flex-1">
              <label className="text-xs text-gray-400">Top-P ({topP})</label>
              <input type="range" min="0" max="1" step="0.05" value={topP} onChange={(e) => setTopP(Number(e.target.value))}
                className="w-full" />
            </div>
          </div>
          <div>
            <label className="text-xs text-gray-400">Modo de ingesta</label>
            <div className="flex rounded overflow-hidden border border-gray-700">
              <button onClick={() => setEmbedderMode('embed')}
                className={`flex-1 px-2 py-1 text-xs ${embedderMode === 'embed' ? 'bg-primary-500 text-white' : 'bg-dark-900 text-gray-400'}`}>
                Embed (RAG)
              </button>
              <button onClick={() => setEmbedderMode('attach')}
                className={`flex-1 px-2 py-1 text-xs ${embedderMode === 'attach' ? 'bg-primary-500 text-white' : 'bg-dark-900 text-gray-400'}`}>
                Attach (contexto)
              </button>
            </div>
          </div>
          <div>
            <label className="text-xs text-gray-400">System prompt</label>
            <textarea value={systemPrompt} onChange={(e) => setSystemPrompt(e.target.value)} rows={4}
              placeholder="Instrucciones base para el asistente..."
              className="w-full rounded bg-dark-900 border border-gray-700 px-2 py-1 text-sm text-gray-200 resize-none" />
          </div>
        </div>

        <div className="mt-4 flex justify-end gap-2">
          <button onClick={onClose} className="rounded bg-gray-700 px-3 py-1.5 text-xs text-gray-200 hover:bg-gray-600">
            Cancelar
          </button>
          <button onClick={handleSave} disabled={saving}
            className="rounded bg-primary-500 px-3 py-1.5 text-xs text-white hover:bg-primary-600 disabled:opacity-50">
            {saving ? 'Guardando...' : 'Guardar'}
          </button>
        </div>
      </div>
    </div>
  )
}