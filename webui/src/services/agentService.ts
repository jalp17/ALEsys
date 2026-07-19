import axios from 'axios';

const API_BASE = import.meta.env.VITE_API_URL || '';

// =============================================================================
// Types
// =============================================================================

export interface AgentInfo {
  id: string;
  name: string;
  os: string;
  arch: string;
  status: 'Connected' | 'Busy' | 'Idle' | 'Disconnected';
  connected_at: string;
}

export interface AgentStats {
  total: number;
  connected: number;
}

export interface AgentExecuteResult {
  exit_code: number;
  stdout: string;
  stderr: string;
}

export interface AgentFileEntry {
  name: string;
  is_dir: boolean;
  size: number;
}

// =============================================================================
// API Client
// =============================================================================

const api = axios.create({
  baseURL: `${API_BASE}/api/v1`,
  headers: { 'Content-Type': 'application/json' },
});

export async function fetchAgents(): Promise<AgentInfo[]> {
  const { data } = await api.get<{ agents: AgentInfo[] }>('/agents');
  return data.agents;
}

export async function fetchAgentStats(): Promise<AgentStats> {
  const { data } = await api.get<AgentStats>('/agents/stats');
  return data;
}

export async function executeOnAgent(
  agentId: string,
  command: string,
  args: string[] = [],
  workdir?: string,
  timeoutMs = 30000
): Promise<AgentExecuteResult> {
  const { data } = await api.post<AgentExecuteResult>(`/agents/${agentId}/execute`, {
    command,
    args,
    workdir,
    timeout_ms: timeoutMs,
  });
  return data;
}

export async function readAgentFile(agentId: string, path: string): Promise<string> {
  const { data } = await api.get<{ content: string }>(`/agents/${agentId}/files`, {
    params: { path },
  });
  return data.content;
}

export async function writeAgentFile(
  agentId: string,
  path: string,
  content: string
): Promise<void> {
  await api.post(`/agents/${agentId}/files`, { path, content });
}

export async function listAgentDir(
  agentId: string,
  path = '.'
): Promise<AgentFileEntry[]> {
  const { data } = await api.get<{ entries: AgentFileEntry[] }>(
    `/agents/${agentId}/files/list`,
    { params: { path } }
  );
  return data.entries;
}
