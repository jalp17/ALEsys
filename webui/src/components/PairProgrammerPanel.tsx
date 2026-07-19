import { useState, useEffect } from 'react';

interface Suggestion {
  id: string;
  suggestion_type: string;
  file_path: string;
  line: number;
  description: string;
  severity: 'Low' | 'Medium' | 'High' | 'Critical';
  auto_fixable: boolean;
}

interface ProjectContext {
  total_files: number;
  total_lines: number;
  file_types: Record<string, number>;
}

const pairProgrammerService = {
  async analyzeCode(code: string, filePath: string): Promise<Suggestion[]> {
    const res = await fetch('/api/v1/pair-programmer/analyze', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, file_path: filePath }),
    });
    const data = await res.json();
    return data.suggestions || [];
  },

  async analyzeProject(): Promise<ProjectContext> {
    const res = await fetch('/api/v1/pair-programmer/project');
    return res.json();
  },

  async applyRefactor(code: string, type: string): Promise<string> {
    const res = await fetch('/api/v1/pair-programmer/refactor', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, refactor_type: type }),
    });
    const data = await res.json();
    return data.code || code;
  },
};

export function SuggestionsPanel({
  code,
  filePath,
  onApplyFix,
}: {
  code: string;
  filePath: string;
  onApplyFix: (suggestion: Suggestion) => void;
}) {
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [loading, setLoading] = useState(false);
  const [expanded, setExpanded] = useState(true);

  useEffect(() => {
    if (code) {
      analyzeCode();
    }
  }, [code]);

  const analyzeCode = async () => {
    setLoading(true);
    try {
      const result = await pairProgrammerService.analyzeCode(code, filePath);
      setSuggestions(result);
    } catch (err) {
      console.error('Failed to analyze code:', err);
    }
    setLoading(false);
  };

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'Critical':
        return 'bg-red-100 text-red-800 border-red-200';
      case 'High':
        return 'bg-orange-100 text-orange-800 border-orange-200';
      case 'Medium':
        return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      default:
        return 'bg-blue-100 text-blue-800 border-blue-200';
    }
  };

  return (
    <div className="border rounded-lg bg-dark-800">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full px-4 py-2 flex items-center justify-between text-left"
      >
        <span className="font-semibold">
          Suggestions ({suggestions.length})
        </span>
        <span>{expanded ? '▲' : '▼'}</span>
      </button>
      {expanded && (
        <div className="px-4 pb-4 space-y-2 max-h-64 overflow-y-auto">
          {loading ? (
            <div className="text-gray-500 text-sm">Analyzing...</div>
          ) : suggestions.length === 0 ? (
            <div className="text-green-500 text-sm">No issues found</div>
          ) : (
            suggestions.map((s) => (
              <div
                key={s.id}
                className={`p-2 rounded border text-sm ${severityColor(s.severity)}`}
              >
                <div className="flex items-center justify-between">
                  <span className="font-medium">{s.description}</span>
                  <span className="text-xs">{s.file_path}:{s.line}</span>
                </div>
                <div className="flex items-center gap-2 mt-1">
                  <span className="text-xs px-1.5 py-0.5 rounded bg-white bg-opacity-50">
                    {s.suggestion_type}
                  </span>
                  {s.auto_fixable && (
                    <button
                      onClick={() => onApplyFix(s)}
                      className="text-xs px-2 py-0.5 bg-blue-600 text-white rounded hover:bg-blue-700"
                    >
                      Auto-fix
                    </button>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}

export function ProjectAnalyzer() {
  const [context, setContext] = useState<ProjectContext | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadContext();
  }, []);

  const loadContext = async () => {
    try {
      const result = await pairProgrammerService.analyzeProject();
      setContext(result);
    } catch (err) {
      console.error('Failed to load project context:', err);
    }
    setLoading(false);
  };

  if (loading) {
    return <div className="text-gray-500">Loading project analysis...</div>;
  }

  if (!context) {
    return <div className="text-gray-500">Failed to load project</div>;
  }

  return (
    <div className="border rounded-lg p-4 bg-dark-800">
      <h3 className="font-semibold mb-3">Project Overview</h3>
      <div className="grid grid-cols-2 gap-4 text-sm">
        <div>
          <span className="text-gray-500">Total Files:</span>
          <span className="ml-2 font-medium">{context.total_files}</span>
        </div>
        <div>
          <span className="text-gray-500">Total Lines:</span>
          <span className="ml-2 font-medium">{context.total_lines}</span>
        </div>
      </div>
      <div className="mt-3">
        <span className="text-gray-500 text-sm">Languages:</span>
        <div className="flex flex-wrap gap-1 mt-1">
          {Object.entries(context.file_types).map(([ext, count]) => (
            <span
              key={ext}
              className="px-2 py-0.5 bg-gray-700 text-gray-300 text-xs rounded"
            >
              .{ext} ({count})
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}
