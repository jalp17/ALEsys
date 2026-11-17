import { ReactNode, useState, useEffect } from 'react'
import { NavLink, useLocation, useNavigate } from 'react-router-dom'
import {
  ChevronLeft, ChevronRight, Sparkles, Cpu, Command, PanelRightClose, PanelRight,
} from 'lucide-react'
import { SECTION_GROUPS, findSection, findItem } from '../config/sections'
import { CommandPalette } from '../components/CommandPalette'
import { LLMStatusIndicator } from '../components/LLMStatusIndicator'
import { AgentStatusIndicator } from '../components/AgentStatusIndicator'
import { cn } from '../utils/cn'

interface AppShellProps {
  children: ReactNode
  /** Optional right inspector panel content */
  inspector?: ReactNode
  inspectorOpen?: boolean
  onToggleInspector?: () => void
}

const MODEL_OPTIONS = [
  { id: 'openrouter/free', label: 'OpenRouter · Auto' },
  { id: 'meta/llama-3.3-70b-instruct', label: 'NIM · Llama 3.3 70B' },
  { id: 'gemini-2.5-flash', label: 'Gemini 2.5 Flash' },
]

export function AppShell({ children, inspector, inspectorOpen, onToggleInspector }: AppShellProps) {
  const [collapsed, setCollapsed] = useState(false)
  const [activeGroup, setActiveGroup] = useState<string>(SECTION_GROUPS[0].id)
  const [cmdOpen, setCmdOpen] = useState(false)
  const [model, setModel] = useState(() => localStorage.getItem('alesys_model') || MODEL_OPTIONS[0].id)
  const location = useLocation()
  const navigate = useNavigate()

  // Sync active section with route
  useEffect(() => {
    const g = findSection(location.pathname)
    if (g) setActiveGroup(g.id)
  }, [location.pathname])

  // Global shortcuts
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') { e.preventDefault(); setCmdOpen((o) => !o) }
      if ((e.metaKey || e.ctrlKey) && e.key === 'b') { e.preventDefault(); setCollapsed((c) => !c) }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  const group = SECTION_GROUPS.find((g) => g.id === activeGroup) || SECTION_GROUPS[0]
  const item = findItem(location.pathname)
  const GroupIcon = group.icon

  const onModelChange = (id: string) => {
    setModel(id)
    localStorage.setItem('alesys_model', id)
  }

  return (
    <div className="flex h-screen overflow-hidden bg-dark-900 text-gray-100">
      {/* ================= COLUMN 1: NAV RAIL ================= */}
      <nav className="flex w-14 shrink-0 flex-col items-center border-r border-gray-800 bg-dark-800 py-3">
        <button
          onClick={() => navigate('/workspaces')}
          className="mb-4 flex h-9 w-9 items-center justify-center rounded-lg bg-primary-500 text-white"
          title="ALEsys — Inicio"
        >
          <Sparkles className="h-5 w-5" />
        </button>

        <div className="flex flex-1 flex-col gap-1">
          {SECTION_GROUPS.map((g) => {
            const Icon = g.icon
            const isActive = g.id === activeGroup
            return (
              <button
                key={g.id}
                onClick={() => setActiveGroup(g.id)}
                title={g.label}
                className={cn(
                  'group relative flex h-10 w-10 items-center justify-center rounded-lg transition',
                  isActive ? 'bg-primary-500/20 text-primary-400' : 'text-gray-400 hover:bg-gray-700/50 hover:text-gray-200',
                )}
              >
                <Icon className="h-5 w-5" />
                {isActive && <span className="absolute left-0 h-5 w-0.5 rounded-r bg-primary-400" />}
                <span className="pointer-events-none absolute left-12 z-50 whitespace-nowrap rounded bg-gray-900 px-2 py-1 text-xs text-gray-200 opacity-0 shadow-lg group-hover:opacity-100">
                  {g.label}
                </span>
              </button>
            )
          })}
        </div>

        <button
          onClick={() => setCmdOpen(true)}
          title="Comandos (⌘K)"
          className="mb-2 flex h-10 w-10 items-center justify-center rounded-lg text-gray-400 hover:bg-gray-700/50 hover:text-gray-200"
        >
          <Command className="h-5 w-5" />
        </button>
      </nav>

      {/* ================= COLUMN 2: SECTION PANEL ================= */}
      <aside className={cn('flex shrink-0 flex-col border-r border-gray-800 bg-dark-800 transition-all', collapsed ? 'w-0 overflow-hidden' : 'w-60')}>
        <div className="flex h-12 items-center justify-between border-b border-gray-800 px-4">
          <div className="flex items-center gap-2">
            <GroupIcon className="h-4 w-4 text-primary-400" />
            <span className="text-sm font-semibold text-gray-200">{group.label}</span>
          </div>
          <button onClick={() => setCollapsed(true)} className="text-gray-500 hover:text-gray-300" title="Colapsar panel">
            <ChevronLeft className="h-4 w-4" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-2">
          <div className="mb-2 px-2 text-[10px] font-semibold uppercase tracking-wider text-gray-500">
            Entornos
          </div>
          {group.items.map((i) => {
            const Icon = i.icon
            const isActive = location.pathname === i.to || location.pathname.startsWith(i.to + '/')
            return (
              <NavLink
                key={i.to}
                to={i.to}
                className={cn(
                  'mb-0.5 block rounded-lg px-2 py-2 transition',
                  isActive ? 'bg-primary-500/15 text-primary-300' : 'text-gray-300 hover:bg-gray-700/40',
                )}
              >
                <div className="flex items-center gap-2">
                  <Icon className={cn('h-4 w-4', isActive ? 'text-primary-400' : 'text-gray-400')} />
                  <span className="text-sm">{i.label}</span>
                  {i.badge && <span className="ml-auto rounded bg-primary-500/20 px-1.5 text-[10px] text-primary-300">{i.badge}</span>}
                </div>
                {i.description && isActive && <div className="mt-0.5 pl-6 text-[11px] text-gray-500">{i.description}</div>}
              </NavLink>
            )
          })}
        </div>

        <div className="border-t border-gray-800 p-3">
          <div className="mb-1 flex items-center justify-between text-[11px] text-gray-500">
            <span>Modelo activo</span>
            <Cpu className="h-3 w-3" />
          </div>
          <select
            value={model}
            onChange={(e) => onModelChange(e.target.value)}
            className="w-full rounded bg-dark-900 px-2 py-1 text-xs text-gray-300 border border-gray-700 focus:outline-none focus:border-primary-500"
          >
            {MODEL_OPTIONS.map((m) => <option key={m.id} value={m.id}>{m.label}</option>)}
          </select>
        </div>
      </aside>

      {collapsed && (
        <button onClick={() => setCollapsed(false)} className="absolute left-14 top-3 z-10 rounded bg-dark-800 p-1 text-gray-400 hover:text-gray-200" title="Expandir panel">
          <ChevronRight className="h-4 w-4" />
        </button>
      )}

      {/* ================= COLUMN 3: MAIN ================= */}
      <main className="flex min-w-0 flex-1 flex-col">
        {/* Top header */}
        <header className="flex h-12 shrink-0 items-center gap-3 border-b border-gray-800 bg-dark-800/60 px-4">
          <nav className="flex items-center gap-1.5 text-xs text-gray-500">
            <span>ALEsys</span>
            <ChevronRight className="h-3 w-3" />
            <span>{group.label}</span>
            {item && (
              <>
                <ChevronRight className="h-3 w-3" />
                <span className="text-gray-300">{item.label}</span>
              </>
            )}
          </nav>

          <div className="ml-auto flex items-center gap-3">
            <button
              onClick={() => setCmdOpen(true)}
              className="flex items-center gap-1.5 rounded border border-gray-700 px-2 py-1 text-[11px] text-gray-400 hover:border-gray-500 hover:text-gray-300"
            >
              <Command className="h-3 w-3" />
              <span className="hidden sm:inline">Comandos</span>
              <kbd className="rounded bg-gray-700 px-1 text-[9px]">⌘K</kbd>
            </button>
            <LLMStatusIndicator />
            <AgentStatusIndicator />
            {onToggleInspector && (
              <button
                onClick={onToggleInspector}
                title="Panel inspector"
                className="rounded p-1 text-gray-400 hover:bg-gray-700/50 hover:text-gray-200"
              >
                {inspectorOpen ? <PanelRightClose className="h-4 w-4" /> : <PanelRight className="h-4 w-4" />}
              </button>
            )}
          </div>
        </header>

        {/* Content */}
        <div className="flex min-h-0 flex-1 overflow-hidden">
          <div className="min-w-0 flex-1 overflow-auto">{children}</div>
          {inspectorOpen && inspector && (
            <aside className="w-80 shrink-0 overflow-y-auto border-l border-gray-800 bg-dark-800/50">{inspector}</aside>
          )}
        </div>
      </main>

      <CommandPalette open={cmdOpen} onClose={() => setCmdOpen(false)} />
    </div>
  )
}