import { useState } from 'react';

interface FacetValue {
  value: string;
  count: number;
}

interface Facet {
  field: string;
  values: FacetValue[];
}

interface SearchResult {
  total: number;
  query_time_ms: number;
  facets: Facet[];
  suggestions: string[];
}

const searchService = {
  async facetedSearch(text: string, facets: string[]): Promise<SearchResult> {
    const res = await fetch('/api/v1/search/faceted', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text, facets, page: 0, page_size: 20 }),
    });
    return res.json();
  },

  async suggest(text: string): Promise<{ suggestions: string[] }> {
    const res = await fetch('/api/v1/search/suggest', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text }),
    });
    return res.json();
  },
};

export function AdvancedSearchPanel() {
  const [query, setQuery] = useState('');
  const [result, setResult] = useState<SearchResult | null>(null);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [selectedFilters, setSelectedFilters] = useState<Record<string, string[]>>({});
  const [loading, setLoading] = useState(false);

  const handleSearch = async () => {
    if (!query.trim()) return;
    setLoading(true);
    try {
      const r = await searchService.facetedSearch(query, ['type', 'tags']);
      setResult(r);
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  const handleSuggest = async (text: string) => {
    if (text.length < 2) { setSuggestions([]); return; }
    try {
      const r = await searchService.suggest(text);
      setSuggestions(r.suggestions);
    } catch (err) { console.error(err); }
  };

  const toggleFilter = (field: string, value: string) => {
    setSelectedFilters((prev) => {
      const current = prev[field] || [];
      const next = current.includes(value)
        ? current.filter((v) => v !== value)
        : [...current, value];
      return { ...prev, [field]: next };
    });
  };

  return (
    <div className="space-y-4">
      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Advanced Search</h3>
        <div className="flex gap-2 mb-2">
          <input
            value={query}
            onChange={(e) => { setQuery(e.target.value); handleSuggest(e.target.value); }}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Search documents, code, notes..."
            className="flex-1 bg-dark-900 border border-gray-700 rounded p-2 text-sm"
          />
          <button onClick={handleSearch} disabled={loading}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 text-sm">
            {loading ? 'Searching...' : 'Search'}
          </button>
        </div>

        {suggestions.length > 0 && (
          <div className="flex gap-2 flex-wrap">
            {suggestions.map((s, i) => (
              <button key={i} onClick={() => { setQuery(s); setSuggestions([]); handleSearch(); }}
                className="text-xs px-2 py-1 bg-dark-900 border border-gray-700 rounded text-gray-300 hover:bg-dark-700">
                {s}
              </button>
            ))}
          </div>
        )}
      </div>

      {result && (
        <div className="flex gap-4">
          <div className="w-48 space-y-3">
            <h4 className="font-medium text-sm">Filters</h4>
            {result.facets.map((facet) => (
              <div key={facet.field}>
                <div className="text-xs text-gray-400 uppercase mb-1">{facet.field}</div>
                {facet.values.map((v) => (
                  <label key={v.value} className="flex items-center gap-1 text-sm text-gray-300 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={(selectedFilters[facet.field] || []).includes(v.value)}
                      onChange={() => toggleFilter(facet.field, v.value)}
                    />
                    {v.value} <span className="text-gray-500">({v.count})</span>
                  </label>
                ))}
              </div>
            ))}
          </div>

          <div className="flex-1">
            <div className="text-sm text-gray-400 mb-2">
              {result.total} results in {result.query_time_ms}ms
            </div>
            <div className="p-4 bg-dark-900 border border-gray-700 rounded text-sm text-gray-400">
              No documents indexed yet. Results will appear here once the knowledge base is populated.
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
