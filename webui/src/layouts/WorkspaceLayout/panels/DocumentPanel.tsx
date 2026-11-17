import { useState, useEffect, useCallback } from 'react'
import {
  Workspace, WorkspaceDocument,
  fetchWorkspaceDocuments, addWorkspaceDocument, deleteWorkspaceDocument,
} from '../../../services/workspaceService'

interface DocumentPanelProps {
  workspace: Workspace
}

export function DocumentPanel({ workspace }: DocumentPanelProps) {
  const [docs, setDocs] = useState<WorkspaceDocument[]>([])
  const [title, setTitle] = useState('')
  const [content, setContent] = useState('')
  const [mode, setMode] = useState<'embed' | 'attach'>('embed')
  const [showForm, setShowForm] = useState(false)

  const load = useCallback(async () => {
    try {
      setDocs(await fetchWorkspaceDocuments(workspace.id))
    } catch { /* noop */ }
  }, [workspace.id])

  useEffect(() => { load() }, [load])

  const handleAdd = async () => {
    if (!title.trim()) return
    await addWorkspaceDocument(workspace.id, { title: title.trim(), content, mode })
    setTitle(''); setContent(''); setShowForm(false)
    load()
  }

  const handleDelete = async (id: string) => {
    await deleteWorkspaceDocument(workspace.id, id)
    load()
  }

  return (
    <div className="flex flex-col h-full">
      <div className="border-b border-gray-700 p-3">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-sm font-semibold text-gray-200">Documentos</h3>
          <button
            onClick={() => setShowForm(!showForm)}
            className="rounded bg-primary-500 px-2 py-1 text-xs text-white hover:bg-primary-600"
          >
            {showForm ? 'Cerrar' : '+ Añadir'}
          </button>
        </div>

        {showForm && (
          <div className="space-y-2">
            <div className="flex rounded overflow-hidden border border-gray-700">
              <button
                onClick={() => setMode('embed')}
                className={`flex-1 px-2 py-1 text-xs ${mode === 'embed' ? 'bg-primary-500 text-white' : 'bg-dark-900 text-gray-400'}`}
              >
                Embed (RAG)
              </button>
              <button
                onClick={() => setMode('attach')}
                className={`flex-1 px-2 py-1 text-xs ${mode === 'attach' ? 'bg-primary-500 text-white' : 'bg-dark-900 text-gray-400'}`}
              >
                Attach (contexto)
              </button>
            </div>
            <input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Título del documento"
              className="w-full rounded bg-dark-900 border border-gray-700 px-2 py-1 text-xs text-gray-200"
            />
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="Pega el contenido aquí..."
              rows={6}
              className="w-full rounded bg-dark-900 border border-gray-700 px-2 py-1 text-xs text-gray-200 resize-none"
            />
            <button
              onClick={handleAdd}
              disabled={!title.trim()}
              className="w-full rounded bg-green-600 px-2 py-1 text-xs text-white hover:bg-green-500 disabled:opacity-50"
            >
              Guardar documento
            </button>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {docs.length === 0 && <div className="text-xs text-gray-500 p-2">Sin documentos en este cuaderno.</div>}
        {docs.map((d) => (
          <div key={d.id} className="group flex items-start justify-between rounded bg-dark-900 px-2 py-1.5">
            <div className="min-w-0 flex-1">
              <div className="text-xs text-gray-200 truncate">{d.title}</div>
              <div className="text-[10px] text-gray-500">
                {d.mode === 'embed' ? 'RAG · embed' : 'contexto · attach'}
                {d.content ? ` · ${d.content.length} chars` : ''}
              </div>
            </div>
            <button
              onClick={() => handleDelete(d.id)}
              className="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-300 text-xs ml-1"
            >
              ×
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}