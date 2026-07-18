/**
 * Advanced Search API Client (Phase 6)
 *
 * Provides typed access to the /api/v1/search/advanced endpoint
 * with RRF fusion, multi-filter queries, and term highlighting.
 */

// =============================================================================
// Types
// =============================================================================

export interface VectorParams {
  limit: number;
  weight: number;
}

export interface GraphParams {
  degrees: number;
  weight: number;
  centrality_boost?: string;
}

export interface SearchFilters {
  doc_types: string[];
  areas: number[];
  subareas: number[];
  date_from?: string;
  date_to?: string;
  content_pattern?: string;
}

export interface ExpansionParams {
  enabled: boolean;
  max_terms: number;
}

export interface HighlightParams {
  enabled: boolean;
  frag_size: number;
}

export interface AdvancedSearchQuery {
  query: string;
  vector?: VectorParams;
  graph?: GraphParams;
  filters?: SearchFilters;
  expansion?: ExpansionParams;
  highlight?: HighlightParams;
  limit?: number;
  offset?: number;
}

export interface ScoreBreakdown {
  vector: number;
  graph: number;
  rrf: number;
}

export interface AdvancedSearchResult {
  fragment_id: number;
  document_id: number;
  path?: string;
  content: string;
  highlighted?: string;
  similarity: number;
  score_breakdown: ScoreBreakdown;
  source?: string;
}

export interface AdvancedSearchResponse {
  results: AdvancedSearchResult[];
  total: number;
  took_ms: number;
  expanded_terms: string[];
}

// =============================================================================
// API Client
// =============================================================================

const API_BASE = import.meta.env.VITE_API_URL || '';

export async function advancedSearch(
  query: AdvancedSearchQuery
): Promise<AdvancedSearchResponse> {
  const response = await fetch(`${API_BASE}/api/v1/search/advanced`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(query),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }));
    throw new Error(error.error || `Search failed: ${response.status}`);
  }

  return response.json();
}

// =============================================================================
// Helpers
// =============================================================================

/**
 * Default search query with sensible defaults
 */
export function defaultSearchQuery(partial?: Partial<AdvancedSearchQuery>): AdvancedSearchQuery {
  return {
    query: '',
    vector: { limit: 10, weight: 1.0 },
    graph: { degrees: 1, weight: 0.5 },
    filters: {
      doc_types: [],
      areas: [],
      subareas: [],
    },
    expansion: { enabled: true, max_terms: 5 },
    highlight: { enabled: true, frag_size: 150 },
    limit: 20,
    offset: 0,
    ...partial,
  };
}

/**
 * Get available document types from the API (or use defaults)
 */
export function getAvailableDocTypes(): string[] {
  return ['markdown', 'code', 'rust', 'python', 'javascript', 'typescript', 'pdf'];
}

/**
 * Get available areas (would come from API in production)
 */
export function getAvailableAreas(): { id: number; name: string }[] {
  return [
    { id: 1, name: 'Computación' },
    { id: 2, name: 'Física' },
    { id: 3, name: 'Matemáticas' },
    { id: 4, name: 'General' },
  ];
}

/**
 * Save a search to localStorage
 */
export interface SavedSearch {
  id: string;
  name: string;
  query: AdvancedSearchQuery;
  created_at: string;
}

/**
 * Validate a saved search object from localStorage
 */
function isValidSavedSearch(obj: unknown): obj is SavedSearch {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    typeof (obj as SavedSearch).id === 'string' &&
    typeof (obj as SavedSearch).name === 'string' &&
    typeof (obj as SavedSearch).created_at === 'string' &&
    typeof (obj as SavedSearch).query === 'object' &&
    (obj as SavedSearch).query !== null
  );
}

export function getSavedSearches(): SavedSearch[] {
  try {
    const stored = localStorage.getItem('alesys_saved_searches');
    if (!stored) return [];
    const parsed = JSON.parse(stored);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isValidSavedSearch);
  } catch {
    return [];
  }
}

export function saveSearch(name: string, query: AdvancedSearchQuery): SavedSearch | null {
  const trimmed = name.trim();
  if (!trimmed) return null;

  const saved: SavedSearch = {
    id: crypto.randomUUID(),
    name,
    query,
    created_at: new Date().toISOString(),
  };

  const searches = getSavedSearches();
  searches.unshift(saved);

  // Keep only last 20 saved searches
  if (searches.length > 20) {
    searches.length = 20;
  }

  localStorage.setItem('alesys_saved_searches', JSON.stringify(searches));
  return saved;
}

export function deleteSavedSearch(id: string): void {
  const searches = getSavedSearches().filter((s) => s.id !== id);
  localStorage.setItem('alesys_saved_searches', JSON.stringify(searches));
}

/**
 * Build URL search params from query for sharing/bookmarks
 */
export function queryToUrlParams(query: AdvancedSearchQuery): URLSearchParams {
  const params = new URLSearchParams();
  if (query.query) params.set('q', query.query);
  if (query.vector) {
    if (query.vector.limit !== 10) params.set('vl', String(query.vector.limit));
    if (query.vector.weight !== 1.0) params.set('vw', String(query.vector.weight));
  }
  if (query.graph) {
    if (query.graph.degrees !== 1) params.set('gd', String(query.graph.degrees));
    if (query.graph.weight !== 0.5) params.set('gw', String(query.graph.weight));
  }
  if (query.filters?.doc_types?.length) params.set('dt', query.filters.doc_types.join(','));
  if (query.filters?.areas?.length) params.set('ar', query.filters.areas.join(','));
  if (query.filters?.subareas?.length) params.set('sa', query.filters.subareas.join(','));
  if (query.filters?.date_from) params.set('df', query.filters.date_from);
  if (query.filters?.date_to) params.set('dt_to', query.filters.date_to);
  if (query.limit !== 20) params.set('lim', String(query.limit));
  return params;
}

/**
 * Parse URL search params back to query
 */
export function urlParamsToQuery(params: URLSearchParams): Partial<AdvancedSearchQuery> {
  const partial: Partial<AdvancedSearchQuery> = {};
  const q = params.get('q');
  if (q) partial.query = q;

  const filters: SearchFilters = {
    doc_types: [],
    areas: [],
    subareas: [],
  };

  const dt = params.get('dt');
  if (dt) filters.doc_types = dt.split(',');
  const ar = params.get('ar');
  if (ar) filters.areas = ar.split(',').map(Number).filter((n) => !isNaN(n) && n > 0);
  const sa = params.get('sa');
  if (sa) filters.subareas = sa.split(',').map(Number).filter((n) => !isNaN(n) && n > 0);
  const df = params.get('df');
  if (df) filters.date_from = df;
  const dtTo = params.get('dt_to');
  if (dtTo) filters.date_to = dtTo;

  if (filters.doc_types.length || filters.areas.length || filters.date_from || filters.date_to) {
    partial.filters = filters;
  }

  const lim = params.get('lim');
  if (lim) partial.limit = parseInt(lim, 10);

  return partial;
}
