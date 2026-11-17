import type { LucideIcon } from 'lucide-react'
import {
  NotebookPen, Network, Terminal,
  Bot, FlaskConical, Bug, Hammer, Users, Workflow, Puzzle,
  GraduationCap, BarChart3, GitGraph, Search, Library,
  Upload, FileText, Layers, Settings, TestTube2, Scissors,
  ListChecks, Sparkles,
} from 'lucide-react'

// ============================================================================
// Section definition — drives the whole shell navigation
// ============================================================================

export interface NavItem {
  to: string
  label: string
  icon: LucideIcon
  description?: string
  badge?: string
}

export interface SectionGroup {
  id: string
  label: string
  icon: LucideIcon
  items: NavItem[]
}

export const SECTION_GROUPS: SectionGroup[] = [
  {
    id: 'research',
    label: 'Investigación',
    icon: NotebookPen,
    items: [
      { to: '/workspaces', label: 'Cuadernos', icon: NotebookPen, description: 'Espacios de trabajo con documentos y chat' },
      { to: '/ingestion', label: 'Ingesta', icon: Upload, description: 'Procesar PDFs y documentos' },
      { to: '/ingestion/history', label: 'Historial', icon: FileText, description: 'Trabajos de ingesta' },
      { to: '/research', label: 'Research', icon: Sparkles, description: 'Proyectos, notas y síntesis' },
      { to: '/sessions', label: 'Sesiones', icon: ListChecks, description: 'Historial de conversaciones' },
    ],
  },
  {
    id: 'development',
    label: 'Desarrollo',
    icon: Terminal,
    items: [
      { to: '/editor', label: 'Editor IDE', icon: Hammer, description: 'Editor con terminal y ejecución' },
      { to: '/generate', label: 'Generar', icon: Sparkles, description: 'Generación de código' },
      { to: '/test-generation', label: 'Tests', icon: TestTube2, description: 'Generación de pruebas' },
      { to: '/debug', label: 'Debug', icon: Bug, description: 'Asistente de depuración' },
      { to: '/refactoring', label: 'Refactor', icon: Scissors, description: 'Refactorización asistida' },
      { to: '/agents', label: 'Agentes', icon: Bot, description: 'Agentes de ejecución' },
      { to: '/orchestrator', label: 'Orchestrator', icon: Workflow, description: 'Orquestación multi-agente' },
      { to: '/collaboration', label: 'Colaboración', icon: Users, description: 'Trabajo colaborativo en vivo' },
      { to: '/multi-agent', label: 'Multi-Agente', icon: FlaskConical, description: 'Colaboración de agentes' },
    ],
  },
  {
    id: 'knowledge',
    label: 'Conocimiento',
    icon: Network,
    items: [
      { to: '/graph', label: 'Grafo', icon: GitGraph, description: 'Visualización GraphRAG' },
      { to: '/search', label: 'Búsqueda', icon: Search, description: 'Búsqueda semántica' },
      { to: '/advanced-search', label: 'Búsqueda Avanzada', icon: Layers, description: 'Filtros y facets' },
      { to: '/kb-curation', label: 'KB Curation', icon: Library, description: 'Deduplicar, unir, partir' },
      { to: '/analytics', label: 'Analytics', icon: BarChart3, description: 'Métricas de uso' },
      { to: '/workflows', label: 'Workflows', icon: Workflow, description: 'Automatizaciones' },
    ],
  },
  {
    id: 'system',
    label: 'Sistema',
    icon: Settings,
    items: [
      { to: '/plugins', label: 'Plugins', icon: Puzzle, description: 'Extensiones y sandbox' },
      { to: '/learning', label: 'Aprendizaje', icon: GraduationCap, description: 'Feedback y memoria' },
      { to: '/settings', label: 'Configuración', icon: Settings, description: 'Modelos y conexiones' },
    ],
  },
]

/** Flat list for command palette search */
export function flattenNav(): NavItem[] {
  return SECTION_GROUPS.flatMap((g) => g.items)
}

/** Find which section a route belongs to */
export function findSection(path: string): SectionGroup | undefined {
  let best: SectionGroup | undefined
  let bestLen = -1
  for (const g of SECTION_GROUPS) {
    for (const item of g.items) {
      if (path === item.to || path.startsWith(item.to + '/')) {
        if (item.to.length > bestLen) {
          best = g
          bestLen = item.to.length
        }
      }
    }
  }
  return best
}

export function findItem(path: string): NavItem | undefined {
  let best: NavItem | undefined
  let bestLen = -1
  for (const g of SECTION_GROUPS) {
    for (const item of g.items) {
      if (path === item.to || path.startsWith(item.to + '/')) {
        if (item.to.length > bestLen) {
          best = item
          bestLen = item.to.length
        }
      }
    }
  }
  return best
}