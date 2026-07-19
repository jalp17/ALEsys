import { useState } from 'react';

interface Chunk {
  id: string;
  content: string;
  index: number;
  start_offset: number;
  end_offset: number;
}

interface MergeResult {
  success: boolean;
  merged_content: string;
  sources_count: number;
  warnings: string[];
}

interface SplitResult {
  original_id: string;
  chunks_count: number;
  chunks: Chunk[];
  strategy_used: string;
}

interface DuplicatePair {
  doc_a_id: string;
  doc_b_id: string;
  similarity_score: number;
}

interface QualityResult {
  document_id: string;
  overall_score: number;
  metrics: { metric: string; score: number; details: string }[];
  recommendations: string[];
}

const kbService = {
  async merge(documents: { id: string; title: string; content: string }[], strategy: string): Promise<MergeResult> {
    const res = await fetch('/api/v1/kb/merge', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ documents, strategy }),
    });
    return res.json();
  },

  async split(content: string, strategy: string, documentId: string): Promise<SplitResult> {
    const res = await fetch('/api/v1/kb/split', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content, strategy, document_id: documentId }),
    });
    return res.json();
  },

  async findDuplicates(documents: { id: string; title: string; content: string }[]): Promise<{ pairs: DuplicatePair[] }> {
    const res = await fetch('/api/v1/kb/duplicates', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ documents, threshold: 0.5, method: 'fuzzy' }),
    });
    return res.json();
  },

  async qualityCheck(documentId: string, content: string): Promise<QualityResult> {
    const res = await fetch('/api/v1/kb/quality', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ document_id: documentId, content, metadata: {} }),
    });
    return res.json();
  },
};

type Tab = 'merge' | 'split' | 'duplicates' | 'quality';

export function KnowledgeCurationPanel() {
  const [tab, setTab] = useState<Tab>('merge');
  const [content, setContent] = useState('');
  const [docId, setDocId] = useState('doc-1');
  const [strategy, setStrategy] = useState('smart');
  const [mergeResult, setMergeResult] = useState<MergeResult | null>(null);
  const [splitResult, setSplitResult] = useState<SplitResult | null>(null);
  const [qualityResult, setQualityResult] = useState<QualityResult | null>(null);
  const [dupCount, setDupCount] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);

  const handleMerge = async () => {
    setLoading(true);
    try {
      const result = await kbService.merge(
        [{ id: docId, title: 'Doc 1', content }],
        strategy
      );
      setMergeResult(result);
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  const handleSplit = async () => {
    setLoading(true);
    try {
      const result = await kbService.split(content, strategy, docId);
      setSplitResult(result);
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  const handleDuplicates = async () => {
    setLoading(true);
    try {
      const result = await kbService.findDuplicates([
        { id: docId, title: 'Doc', content },
      ]);
      setDupCount(result.pairs.length);
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  const handleQuality = async () => {
    setLoading(true);
    try {
      const result = await kbService.qualityCheck(docId, content);
      setQualityResult(result);
    } catch (err) { console.error(err); }
    setLoading(false);
  };

  const tabs: { key: Tab; label: string }[] = [
    { key: 'merge', label: 'Merge' },
    { key: 'split', label: 'Split' },
    { key: 'duplicates', label: 'Duplicates' },
    { key: 'quality', label: 'Quality' },
  ];

  return (
    <div className="space-y-4">
      <div className="border rounded-lg bg-dark-800 p-4">
        <div className="flex gap-2 mb-4">
          {tabs.map((t) => (
            <button
              key={t.key}
              onClick={() => setTab(t.key)}
              className={`px-3 py-1 rounded text-sm ${tab === t.key ? 'bg-blue-600 text-white' : 'bg-dark-900 text-gray-300'}`}
            >
              {t.label}
            </button>
          ))}
        </div>

        <input
          value={docId}
          onChange={(e) => setDocId(e.target.value)}
          placeholder="Document ID"
          className="w-full bg-dark-900 border border-gray-700 rounded p-2 text-sm mb-2"
        />
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder="Document content..."
          className="w-full h-40 bg-dark-900 border border-gray-700 rounded p-3 text-sm font-mono resize-none"
        />

        <div className="flex items-center gap-3 mt-2">
          <select
            value={strategy}
            onChange={(e) => setStrategy(e.target.value)}
            className="bg-dark-900 border border-gray-700 rounded px-3 py-2 text-sm"
          >
            <option value="smart">Smart</option>
            <option value="concatenate">Concatenate</option>
            <option value="by-headers">By Headers</option>
            <option value="by-paragraphs">By Paragraphs</option>
            <option value="by-size">By Size</option>
          </select>
          <button
            onClick={() => {
              if (tab === 'merge') handleMerge();
              else if (tab === 'split') handleSplit();
              else if (tab === 'duplicates') handleDuplicates();
              else handleQuality();
            }}
            disabled={loading}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Processing...' : `Run ${tab}`}
          </button>
        </div>
      </div>

      {mergeResult && (
        <div className="border rounded-lg bg-dark-800 p-4">
          <h3 className="font-semibold mb-2">Merge Result</h3>
          <div className="text-sm text-gray-400 mb-2">
            {mergeResult.sources_count} source(s) merged
          </div>
          <pre className="bg-dark-900 border border-gray-700 rounded p-3 text-sm overflow-auto max-h-60 font-mono">
            {mergeResult.merged_content}
          </pre>
        </div>
      )}

      {splitResult && (
        <div className="border rounded-lg bg-dark-800 p-4">
          <h3 className="font-semibold mb-2">Split Result — {splitResult.chunks_count} chunk(s)</h3>
          <div className="space-y-2">
            {splitResult.chunks.map((chunk) => (
              <div key={chunk.id} className="p-2 bg-dark-900 border border-gray-700 rounded text-sm">
                <span className="text-blue-400">{chunk.id}</span>
                <span className="text-gray-500 ml-2">[{chunk.start_offset}..{chunk.end_offset}]</span>
                <pre className="mt-1 text-xs text-gray-300 overflow-auto max-h-20 font-mono">
                  {chunk.content.slice(0, 200)}{chunk.content.length > 200 ? '...' : ''}
                </pre>
              </div>
            ))}
          </div>
        </div>
      )}

      {dupCount !== null && (
        <div className="border rounded-lg bg-dark-800 p-4">
          <h3 className="font-semibold mb-2">Duplicates Found</h3>
          <div className="text-2xl font-bold text-yellow-400">{dupCount}</div>
        </div>
      )}

      {qualityResult && (
        <div className="border rounded-lg bg-dark-800 p-4">
          <h3 className="font-semibold mb-2">Quality Report</h3>
          <div className="flex items-center gap-3 mb-3">
            <span className="text-3xl font-bold text-blue-400">
              {(qualityResult.overall_score * 100).toFixed(0)}%
            </span>
            <span className="text-gray-400">Overall Score</span>
          </div>
          <div className="space-y-2 mb-3">
            {qualityResult.metrics.map((m) => (
              <div key={m.metric} className="flex items-center gap-2 text-sm">
                <span className="w-24 text-gray-400">{m.metric}</span>
                <div className="flex-1 h-2 bg-dark-900 rounded">
                  <div
                    className="h-2 bg-blue-500 rounded"
                    style={{ width: `${m.score * 100}%` }}
                  />
                </div>
                <span className="w-10 text-right text-gray-500">{(m.score * 100).toFixed(0)}%</span>
              </div>
            ))}
          </div>
          {qualityResult.recommendations.length > 0 && (
            <div>
              <h4 className="text-sm font-medium mb-1">Recommendations</h4>
              {qualityResult.recommendations.map((r, i) => (
                <div key={i} className="text-xs text-gray-400">• {r}</div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
