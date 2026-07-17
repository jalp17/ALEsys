/**
 * GeneratePage — MVP de generacion de codigo (Fase 2)
 *
 * Prototipo funcional para probar el servicio POST /api/generate.
 * En Fase 7 sera reemplazado por Monaco editor + tree view + terminal.
 *
 * Responsabilidades de Fase 2 (este componente):
 *   - Formulario de prompt + selector de lenguaje
 *   - Context injection: archivos existentes como contexto
 *   - Vista previa del codigo generado
 *   - Copiar al portapapeles y descargar archivo
 *   - Historial de generaciones en localStorage
 *
 * NO es responsabilidad de Fase 2 (Fase 7):
 *   - Editor inline (Monaco)
 *   - Ejecucion de codigo (sandbox Docker)
 *   - Modificacion de archivos generados
 *   - Tree view de archivos del proyecto
 */

import { useState, useEffect } from 'react';
import { API_BASE_URL } from '../utils/platform';

interface ContextFile {
  name: string;
  content: string;
}

interface GenerateResult {
  file_name: string;
  content: string;
  language: string;
  explanation: string;
  suggestions: string[];
}

interface HistoryEntry {
  id: string;
  timestamp: number;
  prompt: string;
  language: string;
  result: GenerateResult;
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

const HISTORY_KEY = 'alesys_generate_history';
const MAX_HISTORY = 20;

function loadHistory(): HistoryEntry[] {
  try {
    return JSON.parse(localStorage.getItem(HISTORY_KEY) || '[]');
  } catch {
    return [];
  }
}

function saveHistory(entries: HistoryEntry[]) {
  localStorage.setItem(HISTORY_KEY, JSON.stringify(entries.slice(0, MAX_HISTORY)));
}

export function Generate() {
  const [prompt, setPrompt] = useState('');
  const [language, setLanguage] = useState('python');
  const [maxTokens, setMaxTokens] = useState(2048);
  const [contextFiles, setContextFiles] = useState<ContextFile[]>([]);
  const [newFileName, setNewFileName] = useState('');
  const [newFileContent, setNewFileContent] = useState('');
  const [showContext, setShowContext] = useState(false);

  const [isLoading, setIsLoading] = useState(false);
  const [result, setResult] = useState<GenerateResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [showHistory, setShowHistory] = useState(false);

  useEffect(() => {
    setHistory(loadHistory());
  }, []);

  const addContextFile = () => {
    if (!newFileName.trim() || !newFileContent.trim()) return;
    setContextFiles([...contextFiles, { name: newFileName, content: newFileContent }]);
    setNewFileName('');
    setNewFileContent('');
  };

  const removeContextFile = (index: number) => {
    setContextFiles(contextFiles.filter((_, i) => i !== index));
  };

  const handleGenerate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!prompt.trim() || isLoading) return;

    setIsLoading(true);
    setError(null);
    setResult(null);

    const body: Record<string, unknown> = {
      prompt,
      language,
      max_tokens: maxTokens,
    };

    if (contextFiles.length > 0) {
      body.context = {
        existing_files: contextFiles,
        dependencies: [],
      };
    }

    try {
      const response = await fetch(`${API_BASE_URL}/api/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });

      if (!response.ok) {
        const errData = await response.json().catch(() => ({}));
        throw new Error(errData.error || `Error ${response.status}`);
      }

      const data: GenerateResult = await response.json();
      setResult(data);

      // Guardar en historial
      const entry: HistoryEntry = {
        id: crypto.randomUUID(),
        timestamp: Date.now(),
        prompt,
        language,
        result: data,
      };
      const updated = [entry, ...history];
      setHistory(updated);
      saveHistory(updated);
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

  const loadFromHistory = (entry: HistoryEntry) => {
    setPrompt(entry.prompt);
    setLanguage(entry.language);
    setResult(entry.result);
    setShowHistory(false);
  };

  const clearHistory = () => {
    setHistory([]);
    localStorage.removeItem(HISTORY_KEY);
  };

  return (
    <div className="max-w-5xl mx-auto h-full flex flex-col">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white mb-2">Generador de Codigo</h1>
          <p className="text-gray-400">Describe que necesitas y genera codigo en el lenguaje que prefieras</p>
        </div>
        <button
          onClick={() => setShowHistory(!showHistory)}
          className="px-3 py-1 text-sm bg-dark-700 hover:bg-dark-600 rounded transition text-gray-300"
        >
          Historial ({history.length})
        </button>
      </div>

      {/* Historial */}
      {showHistory && (
        <div className="bg-dark-800 border border-gray-700 rounded-lg p-4 mb-6">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-300">Historial de generaciones</h3>
            {history.length > 0 && (
              <button onClick={clearHistory} className="text-xs text-red-400 hover:text-red-300">
                Limpiar
              </button>
            )}
          </div>
          {history.length === 0 ? (
            <p className="text-sm text-gray-500">Sin generaciones previas</p>
          ) : (
            <div className="space-y-2 max-h-48 overflow-y-auto">
              {history.map((entry) => (
                <button
                  key={entry.id}
                  onClick={() => loadFromHistory(entry)}
                  className="w-full text-left px-3 py-2 bg-dark-700 hover:bg-dark-600 rounded transition flex items-center justify-between"
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-sm text-gray-200 truncate">{entry.prompt}</p>
                    <p className="text-xs text-gray-500">
                      {entry.language} - {new Date(entry.timestamp).toLocaleString()}
                    </p>
                  </div>
                  <span className="text-xs text-gray-500 ml-2 shrink-0">
                    {entry.result.file_name}
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Formulario */}
      <form onSubmit={handleGenerate} className="space-y-4 mb-6">
        <div>
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder={"Describe el codigo que necesitas...\n\nEjemplo: Crea una funcion que calcule el factorial de un numero usando recursion"}
            rows={4}
            className="w-full px-4 py-3 bg-dark-800 border border-gray-700 rounded-lg focus:outline-none focus:border-primary-500 text-white placeholder-gray-500 resize-none"
            disabled={isLoading}
          />
        </div>

        {/* Contexto de archivos */}
        <div>
          <button
            type="button"
            onClick={() => setShowContext(!showContext)}
            className="text-sm text-gray-400 hover:text-gray-300 transition"
          >
            {showContext ? '- Ocultar contexto' : '+ Agregar archivos de contexto'}
          </button>

          {showContext && (
            <div className="mt-3 bg-dark-800 border border-gray-700 rounded-lg p-4 space-y-3">
              <p className="text-xs text-gray-500">
                Archivos existentes que el LLM debe conocer para generar codigo compatible.
              </p>

              {contextFiles.map((file, i) => (
                <div key={i} className="flex items-center gap-2 bg-dark-700 rounded px-3 py-2">
                  <span className="text-sm text-gray-300 font-mono truncate flex-1">{file.name}</span>
                  <span className="text-xs text-gray-500">{file.content.length} chars</span>
                  <button
                    type="button"
                    onClick={() => removeContextFile(i)}
                    className="text-red-400 hover:text-red-300 text-sm"
                  >
                    x
                  </button>
                </div>
              ))}

              <div className="flex gap-2">
                <input
                  type="text"
                  value={newFileName}
                  onChange={(e) => setNewFileName(e.target.value)}
                  placeholder="nombre.py"
                  className="flex-1 px-3 py-1.5 bg-dark-700 border border-gray-700 rounded text-sm text-white placeholder-gray-500 focus:outline-none focus:border-primary-500"
                />
                <button
                  type="button"
                  onClick={addContextFile}
                  disabled={!newFileName.trim() || !newFileContent.trim()}
                  className="px-3 py-1.5 text-sm bg-dark-600 hover:bg-dark-500 rounded transition text-gray-300 disabled:opacity-50"
                >
                  Agregar
                </button>
              </div>

              <textarea
                value={newFileContent}
                onChange={(e) => setNewFileContent(e.target.value)}
                placeholder=" contenido del archivo..."
                rows={4}
                className="w-full px-3 py-2 bg-dark-700 border border-gray-700 rounded text-sm text-white placeholder-gray-500 font-mono resize-none focus:outline-none focus:border-primary-500"
              />
            </div>
          )}
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

          <div className="flex-1 overflow-auto bg-dark-900 border border-gray-700 rounded-lg">
            <pre className="p-4 text-sm text-gray-100 font-mono whitespace-pre-wrap overflow-x-auto">
              {result.content}
            </pre>
          </div>

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
                      <span className="text-yellow-500 mt-0.5">*</span>
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
