import { useState } from 'react';
import { API_BASE_URL } from '../utils/platform';

interface GenerateResult {
  file_name: string;
  content: string;
  language: string;
  explanation: string;
  suggestions: string[];
}

const LANGUAGES = [
  { id: 'python', label: 'Python', ext: '.py' },
  { id: 'javascript', label: 'JavaScript', ext: '.js' },
  { id: 'typescript', label: 'TypeScript', ext: '.ts' },
  { id: 'rust', label: 'Rust', ext: '.rs' },
  { id: 'java', label: 'Java', ext: '.java' },
  { id: 'c', label: 'C', ext: '.c' },
  { id: 'cpp', label: 'C++', ext: '.cpp' },
];

export function Generate() {
  const [prompt, setPrompt] = useState('');
  const [language, setLanguage] = useState('python');
  const [maxTokens, setMaxTokens] = useState(2048);
  const [isLoading, setIsLoading] = useState(false);
  const [result, setResult] = useState<GenerateResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const handleGenerate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!prompt.trim() || isLoading) return;

    setIsLoading(true);
    setError(null);
    setResult(null);

    try {
      const response = await fetch(`${API_BASE_URL}/api/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          prompt,
          language,
          max_tokens: maxTokens,
        }),
      });

      if (!response.ok) {
        const errData = await response.json().catch(() => ({}));
        throw new Error(errData.error || `Error ${response.status}`);
      }

      const data: GenerateResult = await response.json();
      setResult(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error desconocido');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCopy = async () => {
    if (!result) return;
    await navigator.clipboard.writeText(result.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleDownload = () => {
    if (!result) return;
    const blob = new Blob([result.content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = result.file_name;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="max-w-5xl mx-auto h-full flex flex-col">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-white mb-2">Generador de Código</h1>
        <p className="text-gray-400">Describe qué necesitas y genera código en el lenguaje que prefieras</p>
      </div>

      {/* Formulario */}
      <form onSubmit={handleGenerate} className="space-y-4 mb-6">
        <div>
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Describe el código que necesitas...&#10;&#10;Ejemplo: Crea una función que calcule el factorial de un número usando recursión"
            rows={4}
            className="w-full px-4 py-3 bg-dark-800 border border-gray-700 rounded-lg focus:outline-none focus:border-primary-500 text-white placeholder-gray-500 resize-none"
            disabled={isLoading}
          />
        </div>

        <div className="flex gap-4 items-end">
          <div className="flex-1">
            <label className="block text-sm text-gray-400 mb-1">Lenguaje</label>
            <select
              value={language}
              onChange={(e) => setLanguage(e.target.value)}
              className="w-full px-3 py-2 bg-dark-800 border border-gray-700 rounded-lg focus:outline-none focus:border-primary-500 text-white"
              disabled={isLoading}
            >
              {LANGUAGES.map((lang) => (
                <option key={lang.id} value={lang.id}>
                  {lang.label}
                </option>
              ))}
            </select>
          </div>

          <div className="w-32">
            <label className="block text-sm text-gray-400 mb-1">Max tokens</label>
            <input
              type="number"
              value={maxTokens}
              onChange={(e) => setMaxTokens(Number(e.target.value))}
              min={256}
              max={8192}
              step={256}
              className="w-full px-3 py-2 bg-dark-800 border border-gray-700 rounded-lg focus:outline-none focus:border-primary-500 text-white"
              disabled={isLoading}
            />
          </div>

          <button
            type="submit"
            disabled={isLoading || !prompt.trim()}
            className="px-6 py-2 bg-primary-600 rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed transition font-semibold"
          >
            {isLoading ? 'Generando...' : 'Generar'}
          </button>
        </div>
      </form>

      {/* Error */}
      {error && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg p-4 mb-6">
          <p className="text-red-400">{error}</p>
        </div>
      )}

      {/* Resultado */}
      {result && (
        <div className="flex-1 flex flex-col min-h-0">
          {/* Header del resultado */}
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-3">
              <span className="text-sm text-gray-400">
                Archivo: <span className="text-white font-mono">{result.file_name}</span>
              </span>
              <span className="text-xs bg-dark-700 px-2 py-1 rounded text-gray-300">
                {result.language}
              </span>
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleCopy}
                className="px-3 py-1 text-sm bg-dark-700 hover:bg-dark-600 rounded transition text-gray-300"
              >
                {copied ? 'Copiado!' : 'Copiar'}
              </button>
              <button
                onClick={handleDownload}
                className="px-3 py-1 text-sm bg-primary-600 hover:bg-primary-700 rounded transition text-white"
              >
                Descargar
              </button>
            </div>
          </div>

          {/* Código */}
          <div className="flex-1 overflow-auto bg-dark-900 border border-gray-700 rounded-lg">
            <pre className="p-4 text-sm text-gray-100 font-mono whitespace-pre-wrap overflow-x-auto">
              {result.content}
            </pre>
          </div>

          {/* Explicación y sugerencias */}
          <div className="mt-4 space-y-3">
            <div className="bg-dark-800 rounded-lg p-3">
              <p className="text-sm text-gray-300">{result.explanation}</p>
            </div>

            {result.suggestions.length > 0 && (
              <div className="bg-dark-800 rounded-lg p-3">
                <p className="text-xs text-gray-400 mb-2">Sugerencias:</p>
                <ul className="space-y-1">
                  {result.suggestions.map((suggestion, i) => (
                    <li key={i} className="text-sm text-yellow-300/80 flex items-start gap-2">
                      <span className="text-yellow-500 mt-0.5">•</span>
                      {suggestion}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
