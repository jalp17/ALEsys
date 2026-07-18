/**
 * Search results list with loading states and pagination
 */

import type { AdvancedSearchResult } from './searchService';
import { SearchResultItem } from './SearchResultItem';

interface SearchResultsProps {
  results: AdvancedSearchResult[];
  total: number;
  tookMs: number;
  expandedTerms: string[];
  isLoading: boolean;
  error?: string | null;
  hasQuery: boolean;
}

export function SearchResults({
  results,
  total,
  tookMs,
  expandedTerms,
  isLoading,
  error,
  hasQuery,
}: SearchResultsProps) {
  // Empty state
  if (!hasQuery && !isLoading) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-center p-8">
        <div className="w-16 h-16 mb-4 text-gray-600">
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
        </div>
        <h3 className="text-lg font-medium text-gray-400 mb-2">
          Búsqueda Avanzada
        </h3>
        <p className="text-sm text-gray-500 max-w-sm">
          Usa la barra de búsqueda para encontrar documentos. Puedes filtrar por
          tipo, área, fecha y más.
        </p>
        <div className="mt-6 text-xs text-gray-600 space-y-1">
          <p>Combina múltiples filtros para refinar resultados</p>
          <p>Los resultados se fusionan con Reciprocal Rank Fusion (RRF)</p>
        </div>
      </div>
    );
  }

  // Loading state
  if (isLoading) {
    return (
      <div className="flex-1 p-6">
        <div className="space-y-4">
          {[1, 2, 3, 4, 5].map((i) => (
            <div key={i} className="bg-dark-700 rounded-lg border border-gray-600 p-4 animate-pulse">
              <div className="flex items-center gap-3 mb-3">
                <div className="h-4 w-8 bg-gray-600 rounded" />
                <div className="h-4 w-48 bg-gray-600 rounded" />
              </div>
              <div className="space-y-2">
                <div className="h-3 bg-gray-600 rounded w-full" />
                <div className="h-3 bg-gray-600 rounded w-4/5" />
                <div className="h-3 bg-gray-600 rounded w-3/5" />
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="flex-1 flex items-center justify-center p-8">
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-6 max-w-md">
          <h3 className="text-red-400 font-medium mb-2">Error en la búsqueda</h3>
          <p className="text-sm text-gray-300">{error}</p>
        </div>
      </div>
    );
  }

  // No results
  if (results.length === 0 && hasQuery) {
    return (
      <div className="flex-1 flex items-center justify-center p-8">
        <div className="text-center">
          <div className="w-12 h-12 mb-3 mx-auto text-gray-600">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
          <h3 className="text-gray-400 font-medium">Sin resultados</h3>
          <p className="text-sm text-gray-500 mt-1">
            Intenta con otros términos o ajusta los filtros
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 p-6">
      {/* Stats bar */}
      <div className="flex items-center justify-between mb-4 text-sm text-gray-400">
        <div className="flex items-center gap-4">
          <span>{total} resultado{total !== 1 ? 's' : ''}</span>
          <span className="text-gray-600">|</span>
          <span>{tookMs}ms</span>
        </div>
        {expandedTerms.length > 0 && (
          <div className="flex items-center gap-2">
            <span className="text-gray-500">Expandido:</span>
            {expandedTerms.map((term) => (
              <span
                key={term}
                className="text-xs px-2 py-0.5 bg-primary-500/20 text-primary-400 rounded"
              >
                {term}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Results list */}
      <div className="space-y-4">
        {results.map((result, index) => (
          <SearchResultItem
            key={result.fragment_id}
            result={result}
            index={index}
          />
        ))}
      </div>
    </div>
  );
}
