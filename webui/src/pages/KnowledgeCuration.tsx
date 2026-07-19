import { KnowledgeCurationPanel } from '../components/KnowledgeCurationPanel';

export default function KnowledgeCurationPage() {
  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Knowledge Base Curation</h1>
        <p className="text-gray-400 text-sm mt-1">
          Manage, merge, split, archive, and quality-check your knowledge base documents.
        </p>
      </div>

      <KnowledgeCurationPanel />

      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Operations</h3>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-blue-400">Merge</div>
            <div className="text-gray-400 mt-1">Combine multiple documents into one with smart conflict resolution</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-green-400">Split</div>
            <div className="text-gray-400 mt-1">Break long documents into logical chunks by headers, paragraphs, or size</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-yellow-400">Duplicates</div>
            <div className="text-gray-400 mt-1">Detect duplicate or near-duplicate documents using fuzzy matching</div>
          </div>
          <div className="p-3 rounded bg-dark-900 border border-gray-700">
            <div className="font-medium text-purple-400">Quality</div>
            <div className="text-gray-400 mt-1">Score documents on completeness, freshness, readability, and more</div>
          </div>
        </div>
      </div>
    </div>
  );
}
