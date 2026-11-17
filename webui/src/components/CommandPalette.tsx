import { useEffect, useState, useCallback, useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import { Search, CornerDownLeft } from 'lucide-react'
import { flattenNav, SECTION_GROUPS } from '../config/sections'
import { cn } from '../utils/cn'

export function CommandPalette({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [query, setQuery] = useState('')
  const [cursor, setCursor] = useState(0)
  const navigate = useNavigate()

  const items = useMemo(() => {
    const flat = flattenNav()
    if (!query.trim()) return flat.slice(0, 8)
    const q = query.toLowerCase()
    return flat.filter(
      (i) => i.label.toLowerCase().includes(q) || i.description?.toLowerCase().includes(q),
    )
  }, [query])

  const go = useCallback(
    (to: string) => {
      navigate(to)
      onClose()
      setQuery('')
    },
    [navigate, onClose],
  )

  useEffect(() => { setCursor(0) }, [query])
  useEffect(() => { if (open) setQuery('') }, [open])

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
      if (e.key === 'ArrowDown') { e.preventDefault(); setCursor((c) => Math.min(c + 1, items.length - 1)) }
      if (e.key === 'ArrowUp') { e.preventDefault(); setCursor((c) => Math.max(c - 1, 0)) }
      if (e.key === 'Enter' && items[cursor]) { e.preventDefault(); go(items[cursor].to) }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [open, items, cursor, go, onClose])

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/60" onClick={onClose}>
      <div
        className="w-[560px] max-w-[92vw] overflow-hidden rounded-xl border border-gray-700 bg-dark-800 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-2 border-b border-gray-700 px-4 py-3">
          <Search className="h-4 w-4 text-gray-400" />
          <input
            autoFocus
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Buscar secciones, herramientas, entornos..."
            className="flex-1 bg-transparent text-sm text-gray-100 placeholder-gray-500 focus:outline-none"
          />
          <kbd className="rounded bg-gray-700 px-1.5 py-0.5 text-[10px] text-gray-400">ESC</kbd>
        </div>

        <div className="max-h-80 overflow-y-auto py-2">
          {items.length === 0 && <div className="px-4 py-6 text-center text-sm text-gray-500">Sin resultados</div>}
          {items.map((item, i) => {
            const group = SECTION_GROUPS.find((g) => g.items.some((x) => x.to === item.to))
            const Icon = item.icon
            return (
              <button
                key={item.to}
                onMouseEnter={() => setCursor(i)}
                onClick={() => go(item.to)}
                className={cn(
                  'flex w-full items-center gap-3 px-4 py-2 text-left',
                  i === cursor ? 'bg-primary-500/20' : 'hover:bg-gray-700/40',
                )}
              >
                <Icon className="h-4 w-4 shrink-0 text-primary-400" />
                <div className="min-w-0 flex-1">
                  <div className="text-sm text-gray-100">{item.label}</div>
                  {item.description && <div className="truncate text-xs text-gray-500">{item.description}</div>}
                </div>
                {group && <span className="text-[10px] text-gray-500">{group.label}</span>}
                {i === cursor && <CornerDownLeft className="h-3 w-3 text-gray-500" />}
              </button>
            )
          })}
        </div>
      </div>
    </div>
  )
}