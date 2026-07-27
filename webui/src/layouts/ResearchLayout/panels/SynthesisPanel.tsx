import { useState, useEffect, useCallback } from 'react'

interface Citation {
  id: string
  title?: string
  authors: string[]
  journal?: string
  year?: number
  doi?: string
  isbn?: string
  url?: string
  raw_text: string
  cited_in_chapter?: string
  cited_page: number
  confidence: number
}

interface SynthesisState {
  title: string
  content: string
  citations: Citation[]
}

const citationService = {
  async listByChapter(chapterId: string): Promise<Citation[]> {
    const res = await fetch(`/api/v1/bibliography/citations/${chapterId}`)
    if (!res.ok) throw new Error('Failed to fetch citations')
    return res.json()
  },

  formatCitation(citation: Citation, style: 'apa' | 'mla' | 'chicago' | 'ieee'): string {
    const authors = citation.authors.join(', ')
    const year = citation.year ? `(${citation.year})` : 's.f.'
    const title = citation.title || citation.raw_text
    const journal = citation.journal || ''
    const doi = citation.doi ? `https://doi.org/${citation.doi}` : citation.url || ''

    switch (style) {
      case 'apa':
        return `${authors} ${year}. ${title}. ${journal}. ${doi}`
      case 'mla':
        return `${authors}. "${title}." ${journal}, ${citation.year || 's.f.'}, ${doi}`
      case 'chicago':
        return `${authors}. "${title}." ${journal} (${citation.year || 's.f.'}). ${doi}`
      case 'ieee':
        return `${authors}, "${title}," ${journal}, ${doi}`
      default:
        return `${authors} ${year}. ${title}. ${journal}. ${doi}`
    }
  },
}

type CitationStyle = 'apa' | 'mla' | 'chicago' | 'ieee'

type Props = {
  onToggleFullscreen?: () => void
  selectedChapterIds?: string[]
}

export default function SynthesisPanel({ onToggleFullscreen: _onToggleFullscreen, selectedChapterIds }: Props) {
  const [state, setState] = useState<SynthesisState>({
    title: '',
    content: '',
    citations: [],
  })
  const [availableCitations, setAvailableCitations] = useState<Citation[]>([])
  const [selectedChapterId, setSelectedChapterId] = useState<string>(selectedChapterIds?.[0] || '')
  const [citationStyle, setCitationStyle] = useState<CitationStyle>('apa')
  const [showCitationPicker, setShowCitationPicker] = useState(false)
  const [isPreviewMode, setIsPreviewMode] = useState(false)

  useEffect(() => {
    if (!selectedChapterId) {
      setAvailableCitations([])
      return
    }

    citationService
      .listByChapter(selectedChapterId)
      .then(setAvailableCitations)
      .catch((error) => console.error('Failed to load citations:', error))
  }, [selectedChapterId])

  const insertCitation = useCallback(
    (citation: Citation) => {
      const formatted = citationService.formatCitation(citation, citationStyle)
      const insertion = `[@${citation.authors[0]?.split(' ')[0] || 'ref'}${citation.year || 's.f.'}]`
      const citationRef = `\n\n> ${formatted}\n`

      setState((prev) => ({
        ...prev,
        content: prev.content + insertion + citationRef,
      }))
      setShowCitationPicker(false)
    },
    [citationStyle],
  )

  const exportMarkdown = useCallback(() => {
    const markdown = `# ${state.title || 'Síntesis sin título'}\n\n${state.content}\n\n## Referencias\n\n${state.citations.map((c, i) => `${i + 1}. ${citationService.formatCitation(c, citationStyle)}`).join('\n')}\n`

    const blob = new Blob([markdown], { type: 'text/markdown;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `${state.title || 'sintesis'}.md`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  }, [state, citationStyle])

  const wordCount = state.content.split(/\s+/).filter(Boolean).length

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-gray-700 px-3 py-2">
        <h2 className="text-sm font-semibold text-gray-200">Síntesis</h2>
        <p className="mt-1 text-xs text-gray-400">
          Editor de revisión sistemática con citas integradas.
        </p>
      </div>

      <div className="border-b border-gray-700 px-3 py-2">
        <input
          type="text"
          placeholder="Título de la síntesis..."
          value={state.title}
          onChange={(e) => setState((prev) => ({ ...prev, title: e.target.value }))}
          className="mb-2 w-full rounded border border-gray-700 bg-dark-800 px-2 py-1 text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
        />
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <select
              value={citationStyle}
              onChange={(e) => setCitationStyle(e.target.value as CitationStyle)}
              className="rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 focus:border-primary-500 focus:outline-none"
            >
              <option value="apa">APA</option>
              <option value="mla">MLA</option>
              <option value="chicago">Chicago</option>
              <option value="ieee">IEEE</option>
            </select>
            <input
              type="text"
              placeholder="Chapter ID..."
              value={selectedChapterId}
              onChange={(e) => setSelectedChapterId(e.target.value)}
              className="w-40 rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
            />
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowCitationPicker(!showCitationPicker)}
              className="rounded bg-primary-500 px-2 py-1 text-xs text-white hover:bg-primary-600"
              type="button"
            >
              Insertar cita
            </button>
            <button
              onClick={() => setIsPreviewMode(!isPreviewMode)}
              className="rounded border border-gray-700 px-2 py-1 text-xs text-gray-300 hover:bg-gray-800"
              type="button"
            >
              {isPreviewMode ? 'Editar' : 'Preview'}
            </button>
            <button
              onClick={exportMarkdown}
              className="rounded border border-gray-700 px-2 py-1 text-xs text-gray-300 hover:bg-gray-800"
              type="button"
            >
              Exportar
            </button>
          </div>
        </div>
      </div>

      {showCitationPicker && (
        <div className="border-b border-gray-700 px-3 py-2">
          <div className="max-h-32 space-y-1 overflow-y-auto">
            {availableCitations.length === 0 ? (
              <p className="text-xs text-gray-400">Sin citas disponibles para este capítulo.</p>
            ) : (
              availableCitations.map((citation) => (
                <button
                  key={citation.id}
                  onClick={() => insertCitation(citation)}
                  className="flex w-full items-start gap-2 rounded border border-gray-800 bg-dark-800 p-2 text-left hover:border-primary-500"
                  type="button"
                >
                  <div className="flex-1">
                    <p className="text-xs font-medium text-gray-200">
                      {citation.title || citation.raw_text.slice(0, 60)}
                    </p>
                    <p className="text-xs text-gray-400">
                      {citation.authors.slice(0, 2).join(', ')}
                      {citation.authors.length > 2 && ' et al.'}
                      {citation.year && ` (${citation.year})`}
                    </p>
                  </div>
                </button>
              ))
            )}
          </div>
        </div>
      )}

      <div className="flex-1 overflow-hidden">
        {isPreviewMode ? (
          <div className="h-full overflow-y-auto p-4">
            <div className="prose prose-invert max-w-none">
              <h1 className="text-2xl font-bold text-gray-100">{state.title || 'Sin título'}</h1>
              <div className="mt-4 whitespace-pre-wrap text-sm text-gray-300">
                {state.content || 'Comienza a escribir tu síntesis...'}
              </div>
            </div>
          </div>
        ) : (
          <textarea
            value={state.content}
            onChange={(e) => setState((prev) => ({ ...prev, content: e.target.value }))}
            placeholder="Escribe tu síntesis aquí... Usa el botón 'Insertar cita' para agregar referencias."
            className="h-full w-full resize-none border-none bg-dark-900 p-4 text-sm text-gray-200 placeholder-gray-500 focus:outline-none"
          />
        )}
      </div>

      <div className="border-t border-gray-700 px-3 py-2 text-xs text-gray-400">
        {wordCount} palabras | {state.citations.length} citas
      </div>
    </div>
  )
}
