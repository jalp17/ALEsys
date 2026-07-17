import axios from 'axios';

const API_BASE = import.meta.env.VITE_API_URL || '';

export interface ApiNode {
  id: string;
  label: string;
  docType: string;
  path: string;
  degree: number;
  pagerank?: number;
  betweenness?: number;
  community?: number;
  color?: string;
}

export interface ApiEdge {
  id: string;
  source: string;
  target: string;
  edgeType: string;
  context?: string;
  weight: number;
  color?: string;
}

export interface GraphStats {
  total_nodes: number;
  total_edges: number;
  density: number;
  avg_degree: number;
  num_communities: number;
}

export interface PaginationInfo {
  cursor: string | null;
  has_more: boolean;
  returned_nodes: number;
  total_available: number;
}

export interface GraphResponse {
  nodes: ApiNode[];
  edges: ApiEdge[];
  stats: GraphStats;
  pagination: PaginationInfo;
}

export interface CentralityValue {
  node_id: string;
  score: number;
}

export interface CentralityResponse {
  metric: string;
  values: CentralityValue[];
  top_nodes: string[];
  threshold: number | null;
}

export interface CommunityInfo {
  id: number;
  size: number;
  members: string[];
  avg_pagerank: number;
  label: string;
}

export interface CommunitiesResponse {
  communities: CommunityInfo[];
  algorithm: string;
  iterations: number;
}

export interface PathResponse {
  source: string;
  target: string;
  path: string[];
  distance: number;
  found: boolean;
  path_length: number;
}

export interface GraphQuery {
  docType?: string;
  edgeType?: string;
  depth?: number;
  limit?: number;
  cursor?: string;
  centerNodeId?: number;
  includeMetrics?: boolean;
}

export interface CentralityQuery {
  metric?: string;
  topK?: number;
  threshold?: number;
}

export interface CommunitiesQuery {
  maxIterations?: number;
}

export interface PathQuery {
  sourceId: number;
  targetId: number;
}

const api = axios.create({
  baseURL: `${API_BASE}/api/v1`,
  headers: { 'Content-Type': 'application/json' },
});

export async function fetchGraph(query: GraphQuery = {}): Promise<GraphResponse> {
  const params = new URLSearchParams();
  if (query.docType) params.set('docType', query.docType);
  if (query.edgeType) params.set('edgeType', query.edgeType);
  if (query.depth !== undefined) params.set('depth', String(query.depth));
  if (query.limit !== undefined) params.set('limit', String(query.limit));
  if (query.cursor) params.set('cursor', query.cursor);
  if (query.centerNodeId !== undefined) params.set('centerNodeId', String(query.centerNodeId));
  if (query.includeMetrics) params.set('includeMetrics', 'true');

  const { data } = await api.get<GraphResponse>('/graph', { params });
  return data;
}

export async function fetchCentrality(query: CentralityQuery = {}): Promise<CentralityResponse> {
  const params = new URLSearchParams();
  if (query.metric) params.set('metric', query.metric);
  if (query.topK !== undefined) params.set('topK', String(query.topK));
  if (query.threshold !== undefined) params.set('threshold', String(query.threshold));

  const { data } = await api.get<CentralityResponse>('/graph/centrality', { params });
  return data;
}

export async function fetchCommunities(query: CommunitiesQuery = {}): Promise<CommunitiesResponse> {
  const params = new URLSearchParams();
  if (query.maxIterations !== undefined) params.set('maxIterations', String(query.maxIterations));

  const { data } = await api.get<CommunitiesResponse>('/graph/communities', { params });
  return data;
}

export async function fetchShortestPath(query: PathQuery): Promise<PathResponse> {
  const params = new URLSearchParams({
    sourceId: String(query.sourceId),
    targetId: String(query.targetId),
  });

  const { data } = await api.get<PathResponse>('/graph/path', { params });
  return data;
}

export async function searchGraph(q: string, limit = 20): Promise<{ nodes: ApiNode[] }> {
  const { data } = await api.get('/graph/search', { params: { q, limit } });
  return data;
}

export async function exportGraphJson(): Promise<GraphResponse> {
  const { data } = await api.get<GraphResponse>('/graph/export');
  return data;
}
