import { useState } from 'react'

interface Cell {
  id: string
  type: 'code' | 'markdown'
  content: string
  output?: string
}

interface NotebookProps {
  initialCells?: Cell[]
  onRunCell?: (cell: Cell) => Promise<string>
}

export default function Notebook({ initialCells = [], onRunCell }: NotebookProps) {
  const [cells, setCells] = useState<Cell[]>(
    initialCells.length
      ? initialCells
      : [{ id: '1', type: 'code', content: '# Write Python here\nprint("hello")' }]
  )

  const addCell = (type: 'code' | 'markdown') => {
    setCells((prev) => [...prev, { id: String(Date.now()), type, content: '' }])
  }

  const updateCell = (id: string, content: string) => {
    setCells((prev) => prev.map((c) => (c.id === id ? { ...c, content } : c)))
  }

  const runCell = async (id: string) => {
    const cell = cells.find((c) => c.id === id)
    if (!cell || cell.type !== 'code' || !onRunCell) return
    const output = await onRunCell(cell)
    setCells((prev) => prev.map((c) => (c.id === id ? { ...c, output } : c)))
  }

  return (
    <div className="flex h-full flex-col gap-2 overflow-auto p-2">
      {cells.map((cell) => (
        <div key={cell.id} className="rounded border border-gray-700 bg-gray-900 p-2">
          <div className="mb-1 flex items-center justify-between">
            <span className="text-xs text-gray-400">{cell.type}</span>
            {cell.type === 'code' && (
              <button
                onClick={() => runCell(cell.id)}
                className="rounded bg-green-700 px-2 py-0.5 text-xs text-white hover:bg-green-600"
              >
                Run
              </button>
            )}
          </div>
          <textarea
            value={cell.content}
            onChange={(e) => updateCell(cell.id, e.target.value)}
            className="w-full rounded bg-gray-800 p-2 font-mono text-sm text-gray-100"
            rows={cell.type === 'code' ? 4 : 2}
          />
          {cell.output && (
            <pre className="mt-2 rounded bg-black p-2 text-xs text-green-300">{cell.output}</pre>
          )}
        </div>
      ))}
      <div className="flex gap-2">
        <button onClick={() => addCell('code')} className="rounded bg-blue-700 px-3 py-1 text-xs text-white hover:bg-blue-600">
          + Code
        </button>
        <button onClick={() => addCell('markdown')} className="rounded bg-gray-700 px-3 py-1 text-xs text-white hover:bg-gray-600">
          + Markdown
        </button>
      </div>
    </div>
  )
}
