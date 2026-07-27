const STORAGE_KEY = 'alesys.research.layout.v1'

export type SavedLayout = {
  panels: Array<{
    id: string
    width: number
    visible: boolean
  }>
  lastProjectId?: string
}

export function loadLayout(): SavedLayout | null {
  if (typeof window === 'undefined') return null
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return null
    return JSON.parse(raw) as SavedLayout
  } catch {
    return null
  }
}

export function saveLayout(layout: SavedLayout): void {
  if (typeof window === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(layout))
  } catch {
    // ignore storage errors
  }
}
