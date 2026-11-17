import { useState, useEffect, useRef, useCallback } from 'react'
import {
  Workspace, WorkspaceThread,
  fetchWorkspaceThreads, createWorkspaceThread, deleteWorkspaceThread,
  fetchWorkspaceDocuments,
} from '../../../services/workspaceService'

const API_BASE = import.meta.env.VITE_API_URL || '';

interface Message {
  role: 'user' | 'assistant'
  content: string
  sources?: any[]
  verification?: any
}

interface ChatPanelProps {
  workspace: Workspace
  activeThreadId: string | null
  activeThreadMode: string
  onThreadChange: (id: string | null, mode: string) => void
}

export function ChatPanel({ workspace, activeThreadId, activeThreadMode, onThreadChange }: ChatPanelProps) {
  const [threads, setThreads] = useState<WorkspaceThread[]>([])
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [sending, setSending] = useState(false)
  const [attachedDocs, setAttachedDocs] = useState<{ id: string; title: string; content: string }[]>([])
  const [wsDocs, setWsDocs] = useState<{ id: string; title: string; mode: string }[]>([])
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const loadThreads = useCallback(async () => {
    try {
      const data = await fetchWorkspaceThreads(workspace.id)
      setThreads(data)
    } catch { /* noop */ }
  }, [workspace.id])

  const loadDocs = useCallback(async () => {
    try {
      const data = await fetchWorkspaceDocuments(workspace.id)
      setWsDocs(data.map((d) => ({ id: d.id, title: d.title, mode: d.mode })))
    } catch { /* noop */ }
  }, [workspace.id])

  useEffect(() => { loadThreads() }, [loadThreads])
  useEffect(() => { loadDocs() }, [loadDocs])

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const createThread = async (mode: string) => {
    const thread = await createWorkspaceThread(workspace.id, { name: `Thread ${threads.length + 1}`, mode })
    setThreads((prev) => [thread, ...prev])
    onThreadChange(thread.id, thread.mode)
    setMessages([])
  }

  const attachDoc = (doc: { id: string; title: string; content: string }) => {
    setAttachedDocs((prev) => (prev.find((d) => d.id === doc.id) ? prev : [...prev, doc]))
  }

  const detachDoc = (id: string) => {
    setAttachedDocs((prev) => prev.filter((d) => d.id !== id))
  }

  const sendMessage = async () => {
    if (!input.trim() || sending) return
    const userMsg: Message = { role: 'user', content: input }
    setMessages((prev) => [...prev, userMsg])
    const query = input
    setInput('')
    setSending(true)
    try {
      // Build attached content into query (AnythingLLM attach mode)
      let fullQuery = query
      if (attachedDocs.length > 0) {
        const ctx = attachedDocs.map((d) => `--- Documento: ${d.title} ---\n${d.content}`).join('\n\n')
        fullQuery = `Contexto de documentos adjuntos:\n${ctx}\n\nPregunta: ${query}`
      }
      // Mode: conversation includes history, query is fresh
      const history = activeThreadMode === 'conversation' ? messages.slice(0, -1) : []
      const chatMessages = [
        ...(workspace.system_prompt ? [{ role: 'system', content: workspace.system_prompt }] : []),
        ...history.map((m: Message) => ({ role: m.role, content: m.content })),
        { role: 'user', content: fullQuery },
      ]
      const resp = await fetch(`${API_BASE}/api/v1/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: fullQuery,
          session_id: undefined,
          messages: chatMessages,
        }),
      }).then((r) => r.json())
      setMessages((prev) => [...prev, {
        role: 'assistant',
        content: resp.response || resp.error || 'Sin respuesta',
        sources: resp.sources,
        verification: resp.verification,
      }])
    } catch (e: any) {
      setMessages((prev) => [...prev, { role: 'assistant', content: `Error: ${e?.message || e}` }])
    } finally {
      setSending(false)
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Thread bar */}
      <div className="flex items-center gap-2 border-b border-gray-700 px-3 py-2">
        <span className="text-xs text-gray-400">Thread:</span>
        <select
          value={activeThreadId || ''}
          onChange={(e) => {
            const t = threads.find((x) => x.id === e.target.value)
            onThreadChange(e.target.value || null, t?.mode || 'conversation')
            setMessages([])
          }}
          className="rounded bg-dark-900 border border-gray-700 px-2 py-1 text-xs text-gray-200"
        >
          <option value="">Nuevo thread</option>
          {threads.map((t) => (
            <option key={t.id} value={t.id}>{t.name} ({t.mode})</option>
          ))}
        </select>
        <div className="flex rounded overflow-hidden border border-gray-700">
          <button
            onClick={() => createThread('conversation')}
            className={`px-2 py-1 text-xs ${activeThreadMode === 'conversation' ? 'bg-primary-500 text-white' : 'bg-dark-900 text-gray-400'}`}
          >
            Conversación
          </button>
          <button
            onClick={() => createThread('query')}
            className={`px-2 py-1 text-xs ${activeThreadMode === 'query' ? 'bg-primary-500 text-white' : 'bg-dark-900 text-gray-400'}`}
          >
            Query
          </button>
        </div>
        {activeThreadId && (
          <button
            onClick={async () => { await deleteWorkspaceThread(workspace.id, activeThreadId); onThreadChange(null, 'conversation'); loadThreads() }}
            className="ml-auto text-xs text-red-400 hover:text-red-300"
          >
            Eliminar thread
          </button>
        )}
      </div>

      {/* Attached documents chips */}
      {attachedDocs.length > 0 && (
        <div className="flex flex-wrap gap-1 border-b border-gray-700 px-3 py-2">
          {attachedDocs.map((d) => (
            <span key={d.id} className="flex items-center gap-1 rounded-full bg-primary-500/20 px-2 py-0.5 text-xs text-primary-300">
              {d.title}
              <button onClick={() => detachDoc(d.id)} className="text-primary-400 hover:text-white">×</button>
            </span>
          ))}
        </div>
      )}

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="text-center text-gray-500 text-sm mt-10">
            Escribe una pregunta sobre los documentos de este cuaderno.
          </div>
        )}
        {messages.map((m, i) => (
          <div key={i} className={`flex ${m.role === 'user' ? 'justify-end' : 'justify-start'}`}>
            <div className={`max-w-[80%] rounded-lg px-3 py-2 text-sm ${
              m.role === 'user' ? 'bg-primary-500 text-white' : 'bg-dark-800 text-gray-200 border border-gray-700'
            }`}>
              <div className="whitespace-pre-wrap">{m.content}</div>
              {m.sources && m.sources.length > 0 && (
                <div className="mt-2 border-t border-gray-700 pt-1">
                  <div className="text-xs text-gray-400 mb-1">Fuentes:</div>
                  {m.sources.map((s, j) => (
                    <div key={j} className="text-xs text-gray-400">• {s.path || s.ruta_relativa || `doc ${s.document_id}`} ({((s.similarity || 0) * 100).toFixed(0)}%)</div>
                  ))}
                </div>
              )}
              {m.verification && (
                <div className="mt-1 text-xs">
                  <span className={m.verification.overall_pass ? 'text-green-400' : 'text-yellow-400'}>
                    {m.verification.overall_pass ? '✓ Verificado' : '⚠️ Revisar'} (faithfulness: {(m.verification.faithfulness * 100).toFixed(0)}%)
                  </span>
                </div>
              )}
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="border-t border-gray-700 p-3">
        {/* Attach from workspace docs */}
        {wsDocs.filter((d) => d.mode === 'attach' || d.mode === 'embed').length > 0 && (
          <div className="mb-2 flex flex-wrap gap-1">
            <span className="text-xs text-gray-500">Adjuntar:</span>
            {wsDocs.map((d) => (
              <button
                key={d.id}
                onClick={() => attachDoc({ id: d.id, title: d.title, content: '' })}
                className="rounded bg-gray-700 px-2 py-0.5 text-xs text-gray-300 hover:bg-gray-600"
              >
                + {d.title}
              </button>
            ))}
          </div>
        )}
        <div className="flex gap-2">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendMessage() } }}
            placeholder="Pregunta algo..."
            rows={2}
            className="flex-1 rounded bg-dark-900 border border-gray-700 px-3 py-2 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-primary-500 resize-none"
          />
          <button
            onClick={sendMessage}
            disabled={sending || !input.trim()}
            className="rounded bg-primary-500 px-4 py-2 text-sm text-white hover:bg-primary-600 disabled:opacity-50"
          >
            {sending ? '...' : 'Enviar'}
          </button>
        </div>
      </div>
    </div>
  )
}