import { useState, useEffect, useMemo } from 'react'
import CytoscapeComponent from 'react-cytoscapejs'
import { type Core } from 'cytoscape'

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

interface CitationGraphData {
  nodes: { data: { id: string; label: string; weight?: number } }[]
  edges: { data: { source: string; target: string; weight?: number } }[]
}

const citationService = {
  async listByChapter(chapterId: string): Promise<Citation[]> {
    const res = await fetch(`/api/v1/bibliography/citations/${chapterId}`)
    if (!res.ok) throw new Error('Failed to fetch citations')
    return res.json()
  },
}

const cyStylesheet: any[] = [
  {
    selector: 'node',
    style: {
      label: 'data(label)',
      'background-color': '#4f46e5',
      'font-size': '10px',
      'color': '#e5e7eb',
      'text-valign': 'bottom',
      'text-margin-y': '4px',
      width: '16px',
      height: '16px',
      'line-color': undefined,
      'target-arrow-color': undefined,
      'target-arrow-shape': undefined,
      'curve-style': undefined,
      'border-width': undefined,
      'border-color': undefined,
    } as any,
  },
  {
    selector: 'edge',
    style: {
      width: 1,
      'line-color': '#4b5563',
      'target-arrow-color': '#4b5563',
      'target-arrow-shape': 'triangle',
      'curve-style': 'bezier',
    } as any,
  },
  {
    selector: 'node:selected',
    style: {
      'background-color': '#22c55e',
      'border-width': 2,
      'border-color': '#86efac',
    } as any,
  },
]

type Props = {
  onToggleFullscreen?: () => void
  selectedChapterId?: string
}

export default function CitationPanel({ onToggleFullscreen: _onToggleFullscreen, selectedChapterId }: Props) {
  const [citations, setCitations] = useState<Citation[]>([])
  const [loading, setLoading] = useState(false)
  const [layout, setLayout] = useState<'cose' | 'grid' | 'circle'>('cose')

  useEffect(() => {
    if (!selectedChapterId) {
      setCitations([])
      return
    }

    setLoading(true)
    citationService
      .listByChapter(selectedChapterId)
      .then(setCitations)
      .catch((error) => console.error('Failed to load citations:', error))
      .finally(() => setLoading(false))
  }, [selectedChapterId])

  const graphData: CitationGraphData = useMemo(() => {
    const nodes: CitationGraphData['nodes'] = []
    const edges: CitationGraphData['edges'] = []

    citations.forEach((citation, index) => {
      const label = citation.title || citation.raw_text.slice(0, 40)
      nodes.push({
        data: {
          id: citation.id,
          label,
          weight: citation.confidence,
        },
      })

      if (index > 0) {
        edges.push({
          data: {
            source: citations[index - 1].id,
            target: citation.id,
            weight: 1,
          },
        })
      }
    })

    return { nodes, edges }
  }, [citations])

  const handleCyInit = (cy: Core) => {
    cy.on('tap', 'node', (event) => {
      const node = event.target
      const citation = citations.find((c) => c.id === node.id())
      if (citation) {
        console.log('Selected citation:', citation)
      }
    })
  }

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-gray-700 px-3 py-2">
        <h2 className="text-sm font-semibold text-gray-200">Red de Citas</h2>
        <p className="mt-1 text-xs text-gray-400">
          Grafo de referencias para el capítulo seleccionado.
        </p>
      </div>

      <div className="border-b border-gray-700 px-3 py-2">
        <div className="flex items-center gap-2">
          <label className="text-xs text-gray-400">Chapter ID:</label>
          <input
            type="text"
            defaultValue={selectedChapterId || ''}
            placeholder="UUID..."
            className="flex-1 rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
            readOnly
          />
        </div>
        <div className="mt-2 flex items-center gap-2">
          <label className="text-xs text-gray-400">Layout:</label>
          <select
            value={layout}
            onChange={(e) => setLayout(e.target.value as 'cose' | 'grid' | 'circle')}
            className="flex-1 rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 focus:border-primary-500 focus:outline-none"
          >
            <option value="cose">CoSE</option>
            <option value="grid">Grid</option>
            <option value="circle">Circle</option>
          </select>
        </div>
      </div>

      <div className="flex-1 overflow-hidden">
        {loading ? (
          <div className="flex items-center justify-center p-6 text-xs text-gray-400">
            Cargando red de citas...
          </div>
        ) : !selectedChapterId ? (
          <div className="flex items-center justify-center p-6 text-xs text-gray-400">
            Selecciona un capítulo para visualizar su red de citas.
          </div>
        ) : citations.length === 0 ? (
          <div className="flex items-center justify-center p-6 text-xs text-gray-400">
            Sin citas para este capítulo.
          </div>
        ) : (
          <div className="h-full w-full">
            <CytoscapeComponent
              elements={graphData as any}
              stylesheet={cyStylesheet as any}
              layout={{
                name: layout,
                animate: false,
                padding: 20,
              }}
              style={{ width: '100%', height: '100%' }}
              cy={handleCyInit as any}
            />
            <div className="border-t border-gray-700 px-3 py-2 text-xs text-gray-400">
              {citations.length} citas | Layout: {layout}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
