import { useAppStore } from '../stores/app'

const API_BASE = import.meta.env.VITE_API_URL || ''

export interface ChatMessage {
  role: 'user' | 'assistant'
  content: string
  sources?: string[]
}

export interface ChatResponse {
  response: string
  session_id: string
  sources?: string[]
}

export async function sendChat(query: string, sessionId: string | null): Promise<ChatResponse> {
  const store = useAppStore()
  const res = await fetch(`${API_BASE}/api/v1/chat`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(store.authHeaders() as Record<string, string>),
    },
    body: JSON.stringify({ query, session_id: sessionId }),
  })
  if (!res.ok) throw new Error(`Chat failed: ${res.status}`)
  return res.json()
}

export async function getSessions(): Promise<any[]> {
  const store = useAppStore()
  const res = await fetch(`${API_BASE}/api/v1/sessions`, {
    headers: store.authHeaders() as Record<string, string>,
  })
  if (!res.ok) throw new Error(`Sessions failed: ${res.status}`)
  const data = await res.json()
  return data.sessions || data || []
}

export async function getGraphNodes(limit = 500): Promise<{ nodes: any[]; edges: any[] }> {
  const store = useAppStore()
  const res = await fetch(`${API_BASE}/api/v1/graph/nodes?limit=${limit}`, {
    headers: store.authHeaders() as Record<string, string>,
  })
  if (!res.ok) throw new Error(`Graph failed: ${res.status}`)
  return res.json()
}
