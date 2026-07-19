import { useState } from 'react';

interface Parameter {
  name: string;
  type: string;
  optional: boolean;
  default?: string;
}

interface TestResult {
  suite_name: string;
  total_tests: number;
  test_code: string;
  summary: string;
}

const testGenService = {
  async generateTests(
    functionName: string,
    parameters: Parameter[],
    returnType: string,
    language: string
  ): Promise<TestResult> {
    const res = await fetch('/api/v1/test-generate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        function_name: functionName,
        parameters,
        return_type: returnType,
        language,
      }),
    });
    return res.json();
  },
};

export function TestGeneratorPanel() {
  const [functionName, setFunctionName] = useState('');
  const [language, setLanguage] = useState('rust');
  const [returnType, setReturnType] = useState('String');
  const [parameters, setParameters] = useState<Parameter[]>([]);
  const [result, setResult] = useState<TestResult | null>(null);
  const [loading, setLoading] = useState(false);

  const addParameter = () => {
    setParameters([...parameters, { name: '', type: 'String', optional: false }]);
  };

  const updateParameter = (index: number, field: keyof Parameter, value: string | boolean) => {
    const updated = [...parameters];
    (updated[index] as any)[field] = value;
    setParameters(updated);
  };

  const removeParameter = (index: number) => {
    setParameters(parameters.filter((_, i) => i !== index));
  };

  const handleGenerate = async () => {
    if (!functionName.trim()) return;
    setLoading(true);
    try {
      const res = await testGenService.generateTests(functionName, parameters, returnType, language);
      setResult(res);
    } catch (err) {
      console.error('Failed to generate tests:', err);
    }
    setLoading(false);
  };

  return (
    <div className="space-y-4">
      <div className="border rounded-lg bg-dark-800 p-4">
        <h3 className="font-semibold mb-3">Function Signature</h3>
        <div className="grid grid-cols-3 gap-3 mb-4">
          <input
            value={functionName}
            onChange={(e) => setFunctionName(e.target.value)}
            placeholder="function_name"
            className="bg-dark-900 border border-gray-700 rounded px-3 py-2 text-sm focus:outline-none focus:border-blue-500"
          />
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
          <input
            value={returnType}
            onChange={(e) => setReturnType(e.target.value)}
            placeholder="return type"
            className="bg-dark-900 border border-gray-700 rounded px-3 py-2 text-sm focus:outline-none focus:border-blue-500"
          />
        </div>

        <div className="mb-3">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-gray-400">Parameters</span>
            <button
              onClick={addParameter}
              className="text-xs px-2 py-1 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              + Add Parameter
            </button>
          </div>
          {parameters.map((param, i) => (
            <div key={i} className="flex items-center gap-2 mb-2">
              <input
                value={param.name}
                onChange={(e) => updateParameter(i, 'name', e.target.value)}
                placeholder="name"
                className="flex-1 bg-dark-900 border border-gray-700 rounded px-2 py-1 text-sm focus:outline-none focus:border-blue-500"
              />
              <input
                value={param.type}
                onChange={(e) => updateParameter(i, 'type', e.target.value)}
                placeholder="type"
                className="flex-1 bg-dark-900 border border-gray-700 rounded px-2 py-1 text-sm focus:outline-none focus:border-blue-500"
              />
              <label className="flex items-center gap-1 text-xs text-gray-400">
                <input
                  type="checkbox"
                  checked={param.optional}
                  onChange={(e) => updateParameter(i, 'optional', e.target.checked)}
                  className="rounded"
                />
                Optional
              </label>
              <button
                onClick={() => removeParameter(i)}
                className="text-red-500 hover:text-red-400 text-xs"
              >
                ×
              </button>
            </div>
          ))}
        </div>

        <button
          onClick={handleGenerate}
          disabled={loading || !functionName.trim()}
          className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
        >
          {loading ? 'Generating...' : 'Generate Tests'}
        </button>
      </div>

      {result && (
        <div className="border rounded-lg bg-dark-800 p-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="font-semibold">Generated Tests</h3>
            <span className="text-sm text-gray-400">{result.summary}</span>
          </div>
          <pre className="bg-dark-900 border border-gray-700 rounded p-3 text-sm overflow-x-auto font-mono">
            {result.test_code}
          </pre>
          <button
            onClick={() => navigator.clipboard.writeText(result.test_code)}
            className="mt-2 text-xs px-2 py-1 bg-gray-700 text-gray-300 rounded hover:bg-gray-600"
          >
            Copy to Clipboard
          </button>
        </div>
      )}
    </div>
  );
}
