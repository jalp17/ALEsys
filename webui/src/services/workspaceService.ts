import axios from 'axios';

const API_BASE = import.meta.env.VITE_API_URL || '';

// =============================================================================
// Types
// =============================================================================

export interface Workspace {
  id: string;
  name: string;
  description: string;
  llm_model: string | null;
  temperature: number;
  top_p: number;
  system_prompt: string | null;
  embedder_mode: string;
  owner_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateWorkspaceRequest {
  name: string;
  description?: string;
  llm_model?: string;
  temperature?: number;
  system_prompt?: string;
  embedder_mode?: string;
}

export interface UpdateWorkspaceRequest {
  name?: string;
  description?: string;
  llm_model?: string | null;
  temperature?: number;
  top_p?: number;
  system_prompt?: string | null;
  embedder_mode?: string;
}

export interface WorkspaceDocument {
  id: string;
  workspace_id: string;
  document_id: number | null;
  title: string;
  mode: string;
  content: string | null;
  created_at: string;
}

export interface AddDocumentRequest {
  title: string;
  mode?: string;
  content?: string;
  document_id?: number;
}

export interface WorkspaceThread {
  id: string;
  workspace_id: string;
  name: string;
  mode: string;
  created_at: string;
  updated_at: string;
}

export interface CreateThreadRequest {
  name?: string;
  mode?: string;
}

// =============================================================================
// API Client
// =============================================================================

const api = axios.create({
  baseURL: `${API_BASE}/api/v1`,
  headers: { 'Content-Type': 'application/json' },
});

function getToken(): string | null {
  try {
    const stored = localStorage.getItem('alesys_token');
    if (stored) return stored;
  } catch { /* noop */ }
  return null;
}

api.interceptors.request.use((config) => {
  const token = getToken();
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});

// =============================================================================
// Workspaces
// =============================================================================

export async function fetchWorkspaces(): Promise<Workspace[]> {
  const { data } = await api.get<Workspace[]>('/workspaces');
  return data;
}

export async function createWorkspace(req: CreateWorkspaceRequest): Promise<Workspace> {
  const { data } = await api.post<Workspace>('/workspaces', req);
  return data;
}

export async function updateWorkspace(id: string, req: UpdateWorkspaceRequest): Promise<Workspace> {
  const { data } = await api.put<Workspace>(`/workspaces/${id}`, req);
  return data;
}

export async function deleteWorkspace(id: string): Promise<void> {
  await api.delete(`/workspaces/${id}`);
}

// =============================================================================
// Documents
// =============================================================================

export async function fetchWorkspaceDocuments(wsId: string): Promise<WorkspaceDocument[]> {
  const { data } = await api.get<WorkspaceDocument[]>(`/workspaces/${wsId}/documents`);
  return data;
}

export async function addWorkspaceDocument(wsId: string, req: AddDocumentRequest): Promise<WorkspaceDocument> {
  const { data } = await api.post<WorkspaceDocument>(`/workspaces/${wsId}/documents`, req);
  return data;
}

export async function deleteWorkspaceDocument(wsId: string, docId: string): Promise<void> {
  await api.delete(`/workspaces/${wsId}/documents/${docId}`);
}

// =============================================================================
// Threads
// =============================================================================

export async function fetchWorkspaceThreads(wsId: string): Promise<WorkspaceThread[]> {
  const { data } = await api.get<WorkspaceThread[]>(`/workspaces/${wsId}/threads`);
  return data;
}

export async function createWorkspaceThread(wsId: string, req: CreateThreadRequest): Promise<WorkspaceThread> {
  const { data } = await api.post<WorkspaceThread>(`/workspaces/${wsId}/threads`, req);
  return data;
}

export async function deleteWorkspaceThread(wsId: string, threadId: string): Promise<void> {
  await api.delete(`/workspaces/${wsId}/threads/${threadId}`);
}