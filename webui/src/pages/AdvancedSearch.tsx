import { AdvancedSearchPanel } from '../components/AdvancedSearchPanel';

export default function AdvancedSearchPage() {
  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Advanced Search</h1>
        <p className="text-gray-400 text-sm mt-1">
          Full-text search with faceted filters, autocomplete suggestions, and highlighting.
        </p>
      </div>

      <AdvancedSearchPanel />

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Search Features</h3>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-blue-400">Full-Text</div>
            <div className="text-gray-400 mt-1">Search across all documents with relevance scoring</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-green-400">Faceted Filters</div>
            <div className="text-gray-400 mt-1">Filter by type, tags, date, and custom fields</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-yellow-400">Synonyms</div>
            <div className="text-gray-400 mt-1">Automatic query expansion with synonyms</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-purple-400">Suggestions</div>
            <div className="text-gray-400 mt-1">Autocomplete as you type</div>
          </div>
        </div>
      </div>
    </div>
  );
}
