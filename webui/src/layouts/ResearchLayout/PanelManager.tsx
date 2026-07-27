import LiteraturePanel from './panels/LiteraturePanel'
import CitationPanel from './panels/CitationPanel'
import NotesPanel from './panels/NotesPanel'
import SynthesisPanel from './panels/SynthesisPanel'
import { useState, useCallback } from 'react'

const DEFAULT_PANELS = [
  { id: 'literature', label: 'Literatura', minWidth: 280, defaultWidth: 360, visible: true },
  { id: 'citation', label: 'Citas', minWidth: 280, defaultWidth: 360, visible: true },
  { id: 'notes', label: 'Notas', minWidth: 260, defaultWidth: 320, visible: true },
  { id: 'synthesis', label: 'Síntesis', minWidth: 320, defaultWidth: 420, visible: true },
]

export type PanelDefinition = {
  id: string
  label: string
  minWidth: number
  defaultWidth: number
  visible: boolean
}

const panelComponents: Record<string, React.ComponentType<{ onToggleFullscreen: () => void }>> = {
  literature: LiteraturePanel,
  citation: CitationPanel,
  notes: NotesPanel,
  synthesis: SynthesisPanel,
}

export default function PanelManager() {
  const [panels, setPanels] = useState<PanelDefinition[]>(DEFAULT_PANELS)
  const [fullscreen, setFullscreen] = useState<string | null>(null)

  const togglePanel = useCallback((id: string) => {
    setPanels((prev) =>
      prev.map((p) => (p.id === id ? { ...p, visible: !p.visible } : p)),
    )
  }, [])

  const setFullscreenPanel = useCallback((id: string | null) => {
    setFullscreen(id)
  }, [])

  const visiblePanels = panels.filter((p) => p.visible)
  const fullscreenPanel = fullscreen ? panels.find((p) => p.id === fullscreen) : null

  return (
    <div className="flex h-full w-full flex-col">
      <div className="flex items-center gap-2 border-b border-gray-700 px-3 py-2">
        {panels.map((p) => (
          <button
            key={p.id}
            onClick={() => togglePanel(p.id)}
            className={`rounded px-2 py-1 text-xs ${p.visible ? 'bg-primary-500/20 text-primary-300' : 'bg-gray-800 text-gray-400'}`}
            type="button"
          >
            {p.label}
          </button>
        ))}
        <div className="ml-auto flex items-center gap-2">
          {fullscreenPanel && (
            <button
              onClick={() => setFullscreenPanel(null)}
              className="rounded bg-gray-800 px-2 py-1 text-xs text-gray-200"
              type="button"
            >
              Salir fullscreen ({fullscreenPanel.label})
            </button>
          )}
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {fullscreen ? (
          <FullscreenPanel panel={fullscreenPanel!} />
        ) : (
          <div className="flex flex-1">
            {visiblePanels.map((panel, index) => (
              <ResizablePanel
                key={panel.id}
                panel={panel}
                isFirst={index === 0}
                onToggleFullscreen={() => setFullscreenPanel(panel.id)}
              />
            ))}
            {visiblePanels.length === 0 && (
              <div className="flex flex-1 items-center justify-center text-gray-500">
                Activa al menos un panel desde la barra superior.
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}

function ResizablePanel({
  panel,
  isFirst,
  onToggleFullscreen,
}: {
  panel: PanelDefinition
  isFirst: boolean
  onToggleFullscreen: () => void
}) {
  const [width, setWidth] = useState(panel.defaultWidth)
  const [isResizing, setIsResizing] = useState(false)

  const handleMouseDown = useCallback(() => {
    setIsResizing(true)
  }, [])

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isResizing) return
      const next = Math.max(panel.minWidth, Math.min(window.innerWidth - 200, e.clientX))
      setWidth(next)
    },
    [isResizing, panel.minWidth],
  )

  const handleMouseUp = useCallback(() => {
    if (isResizing) {
      setIsResizing(false)
    }
  }, [isResizing])

  if (typeof document !== 'undefined') {
    if (isResizing) {
      document.addEventListener('mousemove', handleMouseMove)
      document.addEventListener('mouseup', handleMouseUp)
    }
  }

  const PanelComponent = panelComponents[panel.id]

  return (
    <div className="flex h-full border-r border-gray-700 last:border-r-0" style={{ width }}>
      <div className="flex h-full flex-1 flex-col overflow-hidden">
        <div className="flex items-center justify-between border-b border-gray-700 px-3 py-2">
          <span className="text-xs font-medium text-gray-300">{panel.label}</span>
          <div className="flex items-center gap-1">
            <button
              onClick={onToggleFullscreen}
              className="rounded px-1.5 py-1 text-xs text-gray-400 hover:text-gray-200"
              type="button"
            >
              ⛶
            </button>
          </div>
        </div>
        <div className="flex-1 overflow-y-auto">
          {PanelComponent ? <PanelComponent onToggleFullscreen={onToggleFullscreen} /> : null}
        </div>
      </div>
      {!isFirst && (
        <div
          onMouseDown={handleMouseDown}
          className="w-1 cursor-col-resize bg-gray-700 hover:bg-primary-500"
        />
      )}
    </div>
  )
}

function FullscreenPanel({
  panel,
}: {
  panel: PanelDefinition
}) {
  const PanelComponent = panelComponents[panel.id]

  return (
    <div className="flex h-full w-full flex-col">
      <div className="flex items-center justify-between border-b border-gray-700 px-3 py-2">
        <span className="text-xs font-medium text-gray-300">{panel.label}</span>
      </div>
      <div className="flex-1 overflow-y-auto">
        {PanelComponent ? <PanelComponent onToggleFullscreen={() => {}} /> : null}
      </div>
    </div>
  )
}
