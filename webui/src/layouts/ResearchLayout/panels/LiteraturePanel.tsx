import { useState, useEffect, useCallback } from 'react'

interface IngestionJob {
  job_id: string
  pdf_path: string
  topic: string
  status: string
  progress: number
  message?: string
  output_dir?: string
  markdown_path?: string
  created_at?: string
}

interface DocumentFragment {
  fragment_id: number
  contenido: string
  indice_orden?: number
  creado_en?: string
}

const documentService = {
  async listHistory(topic?: string, status?: string): Promise<IngestionJob[]> {
    const params = new URLSearchParams()
    if (topic) params.set('topic', topic)
    if (status) params.set('status', status)
    const res = await fetch(`/api/v1/ingestion/history?${params.toString()}`)
    if (!res.ok) throw new Error('Failed to fetch history')
    return res.json()
  },

  async getDocumentFragments(documentId: number): Promise<DocumentFragment[]> {
    const res = await fetch(`/api/v1/documents/${documentId}/fragments`)
    if (!res.ok) throw new Error('Failed to fetch fragments')
    return res.json()
  },
}

type StatusFilter = 'all' | 'completed' | 'failed' | 'processing' | 'pending'

export default function LiteraturePanel() {
  const [documents, setDocuments] = useState<IngestionJob[]>([])
  const [loading, setLoading] = useState(true)
  const [topicFilter, setTopicFilter] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [searchQuery, setSearchQuery] = useState('')
  const [expandedDoc, setExpandedDoc] = useState<string | null>(null)
  const [fragments, setFragments] = useState<DocumentFragment[]>([])
  const [loadingFragments, setLoadingFragments] = useState(false)

  const loadDocuments = useCallback(async () => {
    try {
      const data = await documentService.listHistory(
        topicFilter || undefined,
        statusFilter !== 'all' ? statusFilter : undefined,
      )
      setDocuments(data)
    } catch (error) {
      console.error('Failed to load documents:', error)
    } finally {
      setLoading(false)
    }
  }, [topicFilter, statusFilter])

  useEffect(() => {
    loadDocuments()
  }, [loadDocuments])

  const handleToggleExpand = async (jobId: string) => {
    if (expandedDoc === jobId) {
      setExpandedDoc(null)
      setFragments([])
      return
    }

    setExpandedDoc(jobId)
    setLoadingFragments(true)
    try {
      const documentId = parseInt(jobId, 10)
      const data = await documentService.getDocumentFragments(documentId)
      setFragments(data)
    } catch (error) {
      console.error('Failed to load fragments:', error)
      setFragments([])
    } finally {
      setLoadingFragments(false)
    }
  }

  const filteredDocuments = documents.filter((doc) => {
    if (searchQuery && !doc.pdf_path.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false
    }
    return true
  })

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'bg-green-900 text-green-300'
      case 'failed':
        return 'bg-red-900 text-red-300'
      case 'processing':
        return 'bg-yellow-900 text-yellow-300'
      default:
        return 'bg-gray-700 text-gray-300'
    }
  }

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-gray-700 px-3 py-2">
        <h2 className="text-sm font-semibold text-gray-200">Explorador de Literatura</h2>
        <p className="mt-1 text-xs text-gray-400">
          Documentos ingeridos, capítulos y fragmentos indexados.
        </p>
      </div>

      <div className="border-b border-gray-700 px-3 py-2">
        <input
          type="text"
          placeholder="Buscar por ruta..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="mb-2 w-full rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
        />
        <div className="flex gap-2">
          <input
            type="text"
            placeholder="Filtrar por topic..."
            value={topicFilter}
            onChange={(e) => setTopicFilter(e.target.value)}
            className="flex-1 rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
          />
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as StatusFilter)}
            className="rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 focus:border-primary-500 focus:outline-none"
          >
            <option value="all">Todos</option>
            <option value="completed">Completados</option>
            <option value="failed">Fallidos</option>
            <option value="processing">Procesando</option>
            <option value="pending">Pendientes</option>
          </select>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {loading ? (
          <div className="p-3 text-xs text-gray-400">Cargando documentos...</div>
        ) : filteredDocuments.length === 0 ? (
          <div className="p-3 text-xs text-gray-400">
            No hay documentos. Ingesta PDFs desde el panel de Ingesta.
          </div>
        ) : (
          <div className="divide-y divide-gray-800">
            {filteredDocuments.map((doc) => (
              <div key={doc.job_id} className="p-3">
                <div className="flex items-start justify-between">
                  <div className="flex-1 min-w-0">
                    <p className="truncate text-xs font-medium text-gray-200" title={doc.pdf_path}>
                      {doc.pdf_path}
                    </p>
                    <div className="mt-1 flex items-center gap-2">
                      <span className={`text-xs px-1.5 py-0.5 rounded ${getStatusColor(doc.status)}`}>
                        {doc.status}
                      </span>
                      <span className="text-xs text-gray-500">{doc.topic}</span>
                      {doc.created_at && (
                        <span className="text-xs text-gray-600">
                          {new Date(doc.created_at).toLocaleDateString()}
                        </span>
                      )}
                    </div>
                  </div>
                  <button
                    onClick={() => handleToggleExpand(doc.job_id)}
                    className="ml-2 rounded px-2 py-1 text-xs text-gray-400 hover:text-gray-200"
                    type="button"
                  >
                    {expandedDoc === doc.job_id ? 'Ocultar' : 'Ver'}
                  </button>
                </div>

                {expandedDoc === doc.job_id && (
                  <div className="mt-3">
                    {loadingFragments ? (
                      <div className="text-xs text-gray-400">Cargando fragmentos...</div>
                    ) : fragments.length === 0 ? (
                      <div className="text-xs text-gray-400">Sin fragmentos indexados.</div>
                    ) : (
                      <div className="space-y-2">
                        <p className="text-xs text-gray-400">
                          {fragments.length} fragmentos indexados
                        </p>
                        {fragments.slice(0, 20).map((frag) => (
                          <div
                            key={frag.fragment_id}
                            className="rounded border border-gray-800 bg-dark-800 p-2"
                            draggable
                            onDragStart={(e) => {
                              e.dataTransfer.setData('text/plain', frag.contenido)
                              e.dataTransfer.effectAllowed = 'copy'
                            }}
                          >
                            <p className="text-xs text-gray-300 line-clamp-3">
                              {frag.contenido}
                            </p>
                            {frag.indice_orden !== undefined && (
                              <span className="mt-1 text-xs text-gray-600">
                                Fragmento #{frag.indice_orden}
                              </span>
                            )}
                          </div>
                        ))}
                        {fragments.length > 20 && (
                          <p className="text-xs text-gray-500">
                            Mostrando 20 de {fragments.length} fragmentos
                          </p>
                        )}
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
