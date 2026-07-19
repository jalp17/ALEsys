import { useState } from 'react';

interface DebugSuggestion {
  title: string;
  description: string;
  confidence: number;
  action: string;
  priority: string;
}

interface DebugResult {
  summary: string;
  severity: string;
  total_errors: number;
  total_warnings: number;
  patterns_found: number;
  root_cause: string | null;
  suggestions: DebugSuggestion[];
}

const debugService = {
  async analyzeLogs(logs: string): Promise<DebugResult> {
    const res = await fetch('/api/v1/debug/analyze', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ logs }),
    });
    return res.json();
  },
};

export function DebugAssistant() {
  const [logs, setLogs] = useState('');
  const [result, setResult] = useState<DebugResult | null>(null);
  const [loading, setLoading] = useState(false);

  const handleAnalyze = async () => {
    if (!logs.trim()) return;
    setLoading(true);
    try {
      const res = await debugService.analyzeLogs(logs);
      setResult(res);
    } catch (err) {
      console.error('Failed to analyze logs:', err);
    }
    setLoading(false);
  };

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'CRITICAL': return 'bg-red-900 text-red-300';
      case 'HIGH': return 'bg-orange-900 text-orange-300';
      case 'MEDIUM': return 'bg-yellow-900 text-yellow-300';
      case 'LOW': return 'bg-blue-900 text-blue-300';
      default: return 'bg-gray-800 text-gray-300';
    }
  };

  const priorityColor = (priority: string) => {
    switch (priority) {
      case 'High': return 'border-red-600';
      case 'Medium': return 'border-yellow-600';
      default: return 'border-gray-600';
    }
  };

  return (
    <div className="space-y-4">
      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Log Analysis</h3>
        <textarea
          value={logs}
          onChange={(e) => setLogs(e.target.value)}
          placeholder="Paste logs here...&#10;&#10;Example:&#10;[ERROR] db: connection refused&#10;[WARN] app: retrying...&#10;[INFO] server: started"
          className="w-full h-48 bg-dark-900 border border-gray-700 rounded p-3 text-sm font-mono resize-none focus:outline-none focus:border-blue-500"
        />
        <button
          onClick={handleAnalyze}
          disabled={loading || !logs.trim()}
          className="mt-2 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
        >
          {loading ? 'Analyzing...' : 'Analyze Logs'}
        </button>
      </div>

      {result && (
        <div className="space-y-4">
          <div className="border rounded-lg bg-dark-800 p-4">
            <div className="flex items-center gap-3 mb-2">
              <h3 className="font-semibold">Results</h3>
              <span className={`text-xs px-2 py-0.5 rounded ${severityColor(result.severity)}`}>
                {result.severity}
              </span>
            </div>
            <p className="text-sm text-gray-300">{result.summary}</p>
            <div className="grid grid-cols-3 gap-4 mt-3 text-sm">
              <div><span className="text-gray-500">Errors:</span> <span className="text-red-400">{result.total_errors}</span></div>
              <div><span className="text-gray-500">Warnings:</span> <span className="text-yellow-400">{result.total_warnings}</span></div>
              <div><span className="text-gray-500">Patterns:</span> <span className="text-blue-400">{result.patterns_found}</span></div>
            </div>
            {result.root_cause && (
              <div className="mt-3 p-2 bg-red-900 bg-opacity-30 border border-red-800 rounded text-sm">
                <span className="font-medium text-red-400">Root Cause:</span> {result.root_cause}
              </div>
            )}
          </div>

          {result.suggestions.length > 0 && (
            <div className="border rounded-lg bg-dark-800 p-4">
              <h3 className="font-semibold mb-3">Suggestions ({result.suggestions.length})</h3>
              <div className="space-y-3">
                {result.suggestions.map((s, i) => (
                  <div key={i} className={`p-3 rounded border bg-dark-900 ${priorityColor(s.priority)}`}>
                    <div className="flex items-center justify-between">
                      <span className="font-medium">{s.title}</span>
                      <span className="text-xs text-gray-500">{s.action} ({(s.confidence * 100).toFixed(0)}%)</span>
                    </div>
                    <p className="text-sm text-gray-300 mt-1">{s.description}</p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
