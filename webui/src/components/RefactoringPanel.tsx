import { useState } from 'react';

interface RefactoringOpportunity {
  type: string;
  description: string;
  confidence: number;
  impact: string;
}

interface AnalyzeResult {
  blocks: number;
  opportunities: RefactoringOpportunity[];
  dependency_graph: {
    nodes: number;
    edges: number;
    circular_deps: number;
  };
}

interface PreviewResult {
  success: boolean;
  changes: number;
  preview: string;
  can_apply: boolean;
  warnings: string[];
}

const refactoringService = {
  async analyzeCode(code: string, language: string): Promise<AnalyzeResult> {
    const res = await fetch('/api/v1/refactoring/analyze', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, language }),
    });
    return res.json();
  },

  async previewRefactoring(code: string, language: string, refactoringType: string): Promise<PreviewResult> {
    const res = await fetch('/api/v1/refactoring/preview', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, language, refactoring_type: refactoringType }),
    });
    return res.json();
  },
};

export function RefactoringPanel() {
  const [code, setCode] = useState('');
  const [language, setLanguage] = useState('rust');
  const [analysis, setAnalysis] = useState<AnalyzeResult | null>(null);
  const [preview, setPreview] = useState<PreviewResult | null>(null);
  const [loading, setLoading] = useState(false);

  const handleAnalyze = async () => {
    if (!code.trim()) return;
    setLoading(true);
    try {
      const result = await refactoringService.analyzeCode(code, language);
      setAnalysis(result);
      setPreview(null);
    } catch (err) {
      console.error('Failed to analyze:', err);
    }
    setLoading(false);
  };

  const handlePreview = async (type: string) => {
    setLoading(true);
    try {
      const result = await refactoringService.previewRefactoring(code, language, type);
      setPreview(result);
    } catch (err) {
      console.error('Failed to preview:', err);
    }
    setLoading(false);
  };

  const impactColor = (impact: string) => {
    switch (impact) {
      case 'High': return 'text-red-400';
      case 'Medium': return 'text-yellow-400';
      default: return 'text-green-400';
    }
  };

  return (
    <div className="space-y-4">
      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Code Analysis</h3>
        <textarea
          value={code}
          onChange={(e) => setCode(e.target.value)}
          placeholder="Paste your code here..."
          className="w-full h-48 bg-dark-900 border border-gray-700 rounded p-3 text-sm font-mono resize-none focus:outline-none focus:border-blue-500"
        />
        <div className="flex items-center gap-3 mt-2">
          <select
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
            className="bg-dark-900 border border-gray-700 rounded px-3 py-2 text-sm"
          >
            <option value="rust">Rust</option>
            <option value="python">Python</option>
            <option value="typescript">TypeScript</option>
            <option value="javascript">JavaScript</option>
          </select>
          <button
            onClick={handleAnalyze}
            disabled={loading || !code.trim()}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Analyzing...' : 'Analyze Code'}
          </button>
        </div>
      </div>

      {analysis && (
        <div className="border rounded-lg bg-dark-800 p-4">
          <h3 className="font-semibold mb-3">Analysis Results</h3>
          <div className="grid grid-cols-4 gap-4 mb-4 text-sm">
            <div className="p-3 rounded bg-dark-900 border border-gray-700 text-center">
              <div className="text-2xl font-bold text-blue-400">{analysis.blocks}</div>
              <div className="text-gray-400">Code Blocks</div>
            </div>
            <div className="p-3 rounded bg-dark-900 border border-gray-700 text-center">
              <div className="text-2xl font-bold text-yellow-400">{analysis.opportunities.length}</div>
              <div className="text-gray-400">Opportunities</div>
            </div>
            <div className="p-3 rounded bg-dark-900 border border-gray-700 text-center">
              <div className="text-2xl font-bold text-green-400">{analysis.dependency_graph.nodes}</div>
              <div className="text-gray-400">Dependencies</div>
            </div>
            <div className="p-3 rounded bg-dark-900 border border-gray-700 text-center">
              <div className={`text-2xl font-bold ${analysis.dependency_graph.circular_deps > 0 ? 'text-red-400' : 'text-green-400'}`}>
                {analysis.dependency_graph.circular_deps}
              </div>
              <div className="text-gray-400">Circular Deps</div>
            </div>
          </div>

          {analysis.opportunities.length > 0 && (
            <div>
              <h4 className="text-sm font-medium mb-2">Refactoring Opportunities</h4>
              <div className="space-y-2">
                {analysis.opportunities.map((opp, i) => (
                  <div key={i} className="flex items-center justify-between p-2 rounded bg-dark-900 border border-gray-700">
                    <div className="flex-1">
                      <span className="text-sm font-medium">{opp.type}</span>
                      <span className="text-xs text-gray-400 ml-2">{opp.description}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`text-xs ${impactColor(opp.impact)}`}>{opp.impact}</span>
                      <span className="text-xs text-gray-500">{(opp.confidence * 100).toFixed(0)}%</span>
                      <button
                        onClick={() => handlePreview(opp.type)}
                        className="text-xs px-2 py-1 bg-green-600 text-white rounded hover:bg-green-700"
                      >
                        Preview
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {preview && (
        <div className="border rounded-lg bg-dark-800 p-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="font-semibold">Preview</h3>
            <div className="flex items-center gap-2">
              <span className={`text-xs px-2 py-0.5 rounded ${preview.can_apply ? 'bg-green-900 text-green-300' : 'bg-red-900 text-red-300'}`}>
                {preview.can_apply ? 'Safe to Apply' : 'Review Required'}
              </span>
              <span className="text-xs text-gray-400">{preview.changes} change(s)</span>
            </div>
          </div>
          {preview.warnings.length > 0 && (
            <div className="mb-3 p-2 bg-yellow-900 bg-opacity-30 border border-yellow-800 rounded text-sm">
              {preview.warnings.map((w, i) => (
                <div key={i} className="text-yellow-400">⚠ {w}</div>
              ))}
            </div>
          )}
          <pre className="bg-dark-900 border border-gray-700 rounded p-3 text-sm overflow-x-auto font-mono whitespace-pre-wrap">
            {preview.preview}
          </pre>
        </div>
      )}
    </div>
  );
}
