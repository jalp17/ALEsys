import { useState, useEffect, useCallback } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import {
  NotebookPen, Trash2, Plus, Settings, MessageSquare, FolderOpen, Network, Loader2,
} from 'lucide-react'
import {
  fetchWorkspaces, createWorkspace, deleteWorkspace,
  Workspace, CreateWorkspaceRequest,
} from '../../services/workspaceService'
import { ChatPanel } from './panels/ChatPanel'
import { DocumentPanel } from './panels/DocumentPanel'
import { SettingsPanel } from './panels/SettingsPanel'
import { cn } from '../../utils/cn'

type Tab = 'chat' | 'docs' | 'graph'

export default function WorkspaceLayout() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([])
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [newName, setNewName] = useState('')
  const [showSettings, setShowSettings] = useState(false)
  const [tab, setTab] = useState<Tab>('chat')
  const [activeThreadId, setActiveThreadId] = useState<string | null>(null)
  const [activeThreadMode, setActiveThreadMode] = useState<string>('conversation')

  const navigate = useNavigate()
  const params = useParams<{ wsId?: string }>()

  const loadWorkspaces = useCallback(async () => {
    try {
      setLoading(true)
      const data = await fetchWorkspaces()
      setWorkspaces(data)
      setError(null)
    } catch (e: any) {
      setError(e?.message || 'Error cargando cuadernos')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { loadWorkspaces() }, [loadWorkspaces])

  useEffect(() => {
    if (params.wsId && params.wsId !== selectedId) setSelectedId(params.wsId)
  }, [params.wsId, selectedId])

  useEffect(() => {
    if (!selectedId && workspaces.length > 0) selectWorkspace(workspaces[0].id)
  }, [workspaces, selectedId])

  const selectWorkspace = (id: string) => {
    setSelectedId(id)
    setActiveThreadId(null)
    navigate(`/workspaces/${id}`, { replace: true })
  }

  const handleCreate = async () => {
    if (!newName.trim()) return
    const req: CreateWorkspaceRequest = { name: newName.trim(), description: '' }
    const ws = await createWorkspace(req)
    setNewName('')
    setWorkspaces((prev) => [ws, ...prev])
    selectWorkspace(ws.id)
  }

  const handleDelete = async (id: string) => {
    if (!confirm('¿Eliminar este cuaderno? Se borrarán sus documentos y threads.')) return
    await deleteWorkspace(id)
    setWorkspaces((prev) => prev.filter((w) => w.id !== id))
    if (selectedId === id) {
      setSelectedId(null)
      navigate('/workspaces', { replace: true })
    }
  }

  const selected = workspaces.find((w) => w.id === selectedId)

  return (
    <div className="flex h-full">
      {/* ============ NOTEBOOKS SIDEBAR ============ */}
      <div className="flex w-56 shrink-0 flex-col border-r border-gray-800 bg-dark-800/40">
        <div className="border-b border-gray-800 p-3">
          <div className="mb-2 flex items-center gap-2">
            <NotebookPen className="h-4 w-4 text-primary-400" />
            <span className="text-xs font-semibold uppercase tracking-wide text-gray-400">Cuadernos</span>
          </div>
          <div className="flex gap-1">
            <input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
              placeholder="Nombre del cuaderno..."
              className="min-w-0 flex-1 rounded-md border border-gray-700 bg-dark-900 px-2 py-1.5 text-xs text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
            />
            <button
              onClick={handleCreate}
              title="Crear cuaderno"
              className="rounded-md bg-primary-500 p-1.5 text-white hover:bg-primary-600"
            >
              <Plus className="h-4 w-4" />
            </button>
          </div>
        </div>

        <div className="flex-1 space-y-0.5 overflow-y-auto p-2">
          {loading && (
            <div className="flex items-center gap-2 p-3 text-xs text-gray-400">
              <Loader2 className="h-3 w-3 animate-spin" /> Cargando...
            </div>
          )}
          {error && <div className="p-2 text-xs text-red-400">{error}</div>}
          {workspaces.map((ws) => (
            <div
              key={ws.id}
              onClick={() => selectWorkspace(ws.id)}
              className={cn(
                'group flex cursor-pointer items-center justify-between rounded-md px-2 py-2',
                selectedId === ws.id
                  ? 'bg-primary-500/15 ring-1 ring-primary-500/30'
                  : 'hover:bg-gray-700/40',
              )}
            >
              <div className="min-w-0 flex-1">
                <div className={cn('truncate text-sm', selectedId === ws.id ? 'text-primary-300' : 'text-gray-200')}>
                  {ws.name}
                </div>
                {selectedId === ws.id && ws.description && (
                  <div className="truncate text-[11px] text-gray-500">{ws.description}</div>
                )}
              </div>
              <button
                onClick={(e) => { e.stopPropagation(); handleDelete(ws.id) }}
                title="Eliminar"
                className="ml-1 text-gray-600 opacity-0 transition hover:text-red-400 group-hover:opacity-100"
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </div>
          ))}
          {!loading && workspaces.length === 0 && (
            <div className="p-3 text-xs leading-relaxed text-gray-500">
              No hay cuadernos.
              <br />
              Crea uno para organizar documentos, conversaciones y análisis por tema o proyecto.
            </div>
          )}
        </div>

        {selected && (
          <div className="border-t border-gray-800 p-2">
            <button
              onClick={() => setShowSettings(true)}
              className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs text-gray-300 hover:bg-gray-700/50"
            >
              <Settings className="h-3.5 w-3.5" /> Configuración del cuaderno
            </button>
          </div>
        )}
      </div>

      {/* ============ WORKSPACE AREA ============ */}
      <div className="flex min-w-0 flex-1 flex-col">
        {selected ? (
          <>
            {/* Workspace header */}
            <div className="flex h-14 shrink-0 items-center gap-3 border-b border-gray-800 px-4">
              <div className="min-w-0 flex-1">
                <h1 className="truncate text-base font-semibold text-gray-100">{selected.name}</h1>
                <div className="flex items-center gap-2 text-[11px] text-gray-500">
                  <span className="rounded bg-gray-700/60 px-1.5 py-0.5">
                    {selected.embedder_mode === 'embed' ? 'RAG · embed' : 'contexto · attach'}
                  </span>
                  <span className="rounded bg-gray-700/60 px-1.5 py-0.5">T={selected.temperature}</span>
                  {selected.llm_model && <span className="rounded bg-gray-700/60 px-1.5 py-0.5">{selected.llm_model}</span>}
                </div>
              </div>

              {/* Tabs */}
              <div className="flex rounded-lg border border-gray-700 bg-dark-900 p-0.5">
                {([
                  ['chat', 'Chat', MessageSquare],
                  ['docs', 'Documentos', FolderOpen],
                  ['graph', 'Grafo', Network],
                ] as [Tab, string, any][]).map(([id, label, Icon]) => (
                  <button
                    key={id}
                    onClick={() => setTab(id)}
                    className={cn(
                      'flex items-center gap-1.5 rounded-md px-3 py-1 text-xs transition',
                      tab === id ? 'bg-primary-500 text-white' : 'text-gray-400 hover:text-gray-200',
                    )}
                  >
                    <Icon className="h-3.5 w-3.5" />
                    <span className="hidden sm:inline">{label}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Tab content */}
            <div className="flex min-h-0 flex-1">
              {tab === 'chat' && (
                <>
                  <div className="min-w-0 flex-1">
                    <ChatPanel
                      workspace={selected}
                      activeThreadId={activeThreadId}
                      activeThreadMode={activeThreadMode}
                      onThreadChange={(id, mode) => { setActiveThreadId(id); setActiveThreadMode(mode) }}
                    />
                  </div>
                  <div className="hidden w-72 shrink-0 border-l border-gray-800 xl:block">
                    <DocumentPanel workspace={selected} />
                  </div>
                </>
              )}
              {tab === 'docs' && (
                <div className="min-w-0 flex-1 overflow-y-auto p-4">
                  <div className="mx-auto max-w-3xl">
                    <DocumentPanel workspace={selected} />
                  </div>
                </div>
              )}
              {tab === 'graph' && (
                <div className="flex min-h-0 flex-1 items-center justify-center text-sm text-gray-500">
                  <div className="text-center">
                    <Network className="mx-auto mb-2 h-8 w-8 text-gray-600" />
                    Visualización del grafo de conocimiento del cuaderno.
                    <div className="mt-2 text-xs">
                      Ver grafo global en <span className="text-primary-400">Conocimiento → Grafo</span>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </>
        ) : (
          <div className="flex flex-1 flex-col items-center justify-center text-center text-gray-500">
            <NotebookPen className="mb-3 h-10 w-10 text-gray-600" />
            <p className="text-sm">Selecciona o crea un cuaderno para empezar.</p>
            <p className="mt-1 max-w-xs text-xs text-gray-600">
              Un cuaderno agrupa documentos, conversaciones (threads) y configuración de un tema o proyecto,
              con su propia base de conocimiento aislada.
            </p>
          </div>
        )}
      </div>

      {showSettings && selected && (
        <SettingsPanel
          workspace={selected}
          onClose={() => setShowSettings(false)}
          onSaved={(ws) => {
            setWorkspaces((prev) => prev.map((w) => (w.id === ws.id ? ws : w)))
            setShowSettings(false)
          }}
        />
      )}
    </div>
  )
}