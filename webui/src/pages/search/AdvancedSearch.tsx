/**
 * Advanced Search Page (Phase 6)
 *
 * Full-featured search with:
 * - Debounced text search (300ms)
 * - Sidebar filters (type, area, date)
 * - URL state for sharing/bookmarks
 * - Saved searches
 * - Term highlighting
 * - RRF fusion scores
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { useSearchParams } from 'react-router-dom';
import {
  advancedSearch,
  defaultSearchQuery,
  queryToUrlParams,
  urlParamsToQuery,
  saveSearch,
  getSavedSearches,
  deleteSavedSearch,
  type AdvancedSearchQuery,
  type AdvancedSearchResult,
  type SavedSearch,
} from './searchService';
import { SearchSidebar } from './SearchSidebar';
import { SearchResults } from './SearchResults';

export function AdvancedSearch() {
  const [searchParams, setSearchParams] = useSearchParams();

  // Initialize query from URL params
  const [query, setQuery] = useState<AdvancedSearchQuery>(() => {
    const urlPartial = urlParamsToQuery(searchParams);
    return defaultSearchQuery({
      ...urlPartial,
      query: searchParams.get('q') || '',
    });
  });

  const [results, setResults] = useState<AdvancedSearchResult[]>([]);
  const [total, setTotal] = useState(0);
  const [tookMs, setTookMs] = useState(0);
  const [expandedTerms, setExpandedTerms] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [showSavedSearches, setShowSavedSearches] = useState(false);
  const [savedSearches, setSavedSearches] = useState<SavedSearch[]>([]);

  const debounceRef = useRef<ReturnType<typeof setTimeout>>();
  const inputRef = useRef<HTMLInputElement>(null);

  // Load saved searches
  useEffect(() => {
    setSavedSearches(getSavedSearches());
  }, []);

  // Update URL when query changes
  useEffect(() => {
    const params = queryToUrlParams(query);
    setSearchParams(params, { replace: true });
  }, [query, setSearchParams]);

  // Perform search
  const performSearch = useCallback(async (searchQuery: AdvancedSearchQuery) => {
    if (!searchQuery.query && !searchQuery.filters?.doc_types?.length && !searchQuery.filters?.areas?.length) {
      setResults([]);
      setTotal(0);
      setTookMs(0);
      setExpandedTerms([]);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const response = await advancedSearch(searchQuery);
      setResults(response.results);
      setTotal(response.total);
      setTookMs(response.took_ms);
      setExpandedTerms(response.expanded_terms);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error desconocido');
      setResults([]);
      setTotal(0);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Debounced search on query change
  useEffect(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = setTimeout(() => {
      performSearch(query);
    }, 300);

    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, [query, performSearch]);

  // Handle text input
  const handleQueryChange = (value: string) => {
    setQuery((prev) => ({ ...prev, query: value }));
  };

  // Handle filter changes
  const handleFiltersChange = (filters: typeof query.filters) => {
    setQuery((prev) => ({ ...prev, filters, offset: 0 }));
  };

  // Handle saved search
  const handleSaveSearch = () => {
    const name = prompt('Nombre para esta búsqueda:');
    if (name) {
      saveSearch(name, query);
      setSavedSearches(getSavedSearches());
    }
  };

  // Load saved search
  const handleLoadSaved = (saved: SavedSearch) => {
    setQuery(saved.query);
    setShowSavedSearches(false);
  };

  // Delete saved search
  const handleDeleteSaved = (id: string) => {
    deleteSavedSearch(id);
    setSavedSearches(getSavedSearches());
  };

  // Keyboard shortcut: / to focus search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === '/' && document.activeElement !== inputRef.current) {
        e.preventDefault();
        inputRef.current?.focus();
      }
      if (e.key === 'Escape') {
        inputRef.current?.blur();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      <SearchSidebar
        filters={query.filters!}
        onFiltersChange={handleFiltersChange}
        isOpen={sidebarOpen}
        onToggle={() => setSidebarOpen(!sidebarOpen)}
      />

      {/* Main content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Search bar */}
        <div className="p-4 bg-dark-800 border-b border-gray-700">
          <div className="flex items-center gap-3">
            {/* Search icon */}
            <div className="text-gray-400">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
            </div>

            {/* Input */}
            <input
              ref={inputRef}
              type="text"
              value={query.query}
              onChange={(e) => handleQueryChange(e.target.value)}
              placeholder="Buscar documentos... (presiona / para enfocar)"
              className="flex-1 bg-dark-700 border border-gray-600 rounded-lg px-4 py-2 text-white placeholder-gray-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            />

            {/* Actions */}
            <div className="flex items-center gap-2">
              {/* Saved searches toggle */}
              <button
                onClick={() => setShowSavedSearches(!showSavedSearches)}
                className="px-3 py-2 text-sm text-gray-400 hover:text-white bg-dark-700 border border-gray-600 rounded-lg hover:border-gray-500 transition"
                title="Búsquedas guardadas"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
                  />
                </svg>
              </button>

              {/* Save current search */}
              <button
                onClick={handleSaveSearch}
                className="px-3 py-2 text-sm text-gray-400 hover:text-white bg-dark-700 border border-gray-600 rounded-lg hover:border-gray-500 transition"
                title="Guardar búsqueda"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"
                  />
                </svg>
              </button>

              {/* Sidebar toggle (mobile) */}
              <button
                onClick={() => setSidebarOpen(!sidebarOpen)}
                className="lg:hidden px-3 py-2 text-sm text-gray-400 hover:text-white bg-dark-700 border border-gray-600 rounded-lg hover:border-gray-500 transition"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                  />
                </svg>
              </button>
            </div>
          </div>

          {/* Saved searches dropdown */}
          {showSavedSearches && savedSearches.length > 0 && (
            <div className="mt-2 bg-dark-700 border border-gray-600 rounded-lg p-2 max-h-60 overflow-y-auto">
              <h4 className="text-xs text-gray-400 uppercase px-2 py-1 mb-1">
                Búsquedas guardadas
              </h4>
              {savedSearches.map((saved) => (
                <div
                  key={saved.id}
                  className="flex items-center justify-between px-2 py-1.5 hover:bg-dark-600 rounded cursor-pointer group"
                >
                  <span
                    className="text-sm text-gray-300 truncate"
                    onClick={() => handleLoadSaved(saved)}
                  >
                    {saved.name}
                  </span>
                  <button
                    onClick={() => handleDeleteSaved(saved.id)}
                    className="text-gray-600 hover:text-red-400 opacity-0 group-hover:opacity-100 transition"
                  >
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M6 18L18 6M6 6l12 12"
                      />
                    </svg>
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Results */}
        <SearchResults
          results={results}
          total={total}
          tookMs={tookMs}
          expandedTerms={expandedTerms}
          isLoading={isLoading}
          error={error}
          hasQuery={!!query.query || !!query.filters?.doc_types?.length || !!query.filters?.areas?.length}
        />
      </div>
    </div>
  );
}
