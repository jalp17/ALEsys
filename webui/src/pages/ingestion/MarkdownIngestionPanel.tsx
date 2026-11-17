import { useState, useCallback, useEffect } from 'react';

interface MarkdownJob {
  job_id: string;
  dir: string;
  status: 'queued' | 'processing' | 'completed' | 'failed' | 'cancelled' | 'cancelling';
  progress: number;
  files_done: number;
  files_total: number;
  files_indexed: number;
  chunks_indexed: number;
  skipped_empty: number;
  topics: string[];
  current_file?: string;
  current_topic?: string;
  message?: string;
  error?: string;
  created_at: string;
}

interface MarkdownPreviewFile {
  path: string;
  topic: string;
  title: string;
  bytes: number;
}

interface DryRunResult {
  dry_run: true;
  dir: string;
  files_found: number;
  total_bytes: number;
  topics: string[];
  sample: MarkdownPreviewFile[];
}

interface StartResult {
  job_id: string;
  status: 'queued';
  dir: string;
  files_found: number;
  files_total: number;
  total_bytes: number;
  topics: string[];
  poll_url: string;
}

const apiBase = import.meta.env.VITE_API_URL || '';

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const token = localStorage.getItem('alesys_token');
  const res = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...options?.headers,
    },
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: 'Request failed' }));
    throw new Error(err.error || `HTTP ${res.status}`);
  }
  return res.json();
}

const markdownService = {
  async dryRun(dir: string, maxFiles = 5000): Promise<DryRunResult> {
    return fetchJson(`${apiBase}/api/v1/ingestion/markdown`, {
      method: 'POST',
      body: JSON.stringify({ dir, max_files: maxFiles, dry_run: true }),
    });
  },

  async start(dir: string, maxFiles = 5000, topic?: string): Promise<StartResult> {
    return fetchJson(`${apiBase}/api/v1/ingestion/markdown`, {
      method: 'POST',
      body: JSON.stringify({ dir, max_files: maxFiles, topic, dry_run: false }),
    });
  },

  async getStatus(jobId: string): Promise<MarkdownJob> {
    return fetchJson(`${apiBase}/api/v1/ingestion/markdown/status/${jobId}`);
  },

  async listJobs(): Promise<{ jobs: MarkdownJob[] }> {
    return fetchJson(`${apiBase}/api/v1/ingestion/markdown/jobs`);
  },

  async cancel(jobId: string): Promise<{ job_id: string; status: string; message: string }> {
    return fetchJson(`${apiBase}/api/v1/ingestion/markdown/${jobId}/cancel`, {
      method: 'POST',
    });
  },
};

export function MarkdownIngestionPanel() {
  const [dir, setDir] = useState('');
  const [topic, setTopic] = useState('');
  const [maxFiles, setMaxFiles] = useState(5000);
  const [dryRunResult, setDryRunResult] = useState<DryRunResult | null>(null);
  const [job, setJob] = useState<MarkdownJob | null>(null);
  const [jobs, setJobs] = useState<MarkdownJob[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [activeTab, setActiveTab] = useState<'config' | 'monitor' | 'history'>('config');
  const [pollInterval, setPollInterval] = useState<number | null>(null);

  useEffect(() => {
    loadJobs();
  }, []);

  const loadJobs = async () => {
    try {
      const { jobs: list } = await markdownService.listJobs();
      setJobs(list);
    } catch {
      // ignore
    }
  };

  const handleDryRun = async () => {
    if (!dir.trim()) {
      setError('Ingresa un directorio');
      return;
    }
    setLoading(true);
    setError('');
    try {
      const result = await markdownService.dryRun(dir, maxFiles);
      setDryRunResult(result);
      setActiveTab('monitor');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error en dry-run');
    } finally {
      setLoading(false);
    }
  };

  const handleStart = async () => {
    if (!dir.trim()) {
      setError('Ingresa un directorio');
      return;
    }
    setLoading(true);
    setError('');
    try {
      const result = await markdownService.start(dir, maxFiles, topic || undefined);
      setJob(result as unknown as MarkdownJob);
      setActiveTab('monitor');
      startPolling(result.job_id);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error al iniciar ingesta');
    } finally {
      setLoading(false);
    }
  };

  const startPolling = (jobId: string) => {
    if (pollInterval) window.clearInterval(pollInterval);
    const id = window.setInterval(async () => {
      try {
        const status = await markdownService.getStatus(jobId);
        setJob(status);
        if (status.status === 'completed' || status.status === 'failed' || status.status === 'cancelled') {
          window.clearInterval(id);
          setPollInterval(null);
          loadJobs();
        }
      } catch {
        window.clearInterval(id);
        setPollInterval(null);
      }
    }, 1000);
    setPollInterval(id);
  };

  const handleCancel = async (jobId: string) => {
    try {
      await markdownService.cancel(jobId);
      loadJobs();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error al cancelar');
    }
  };

  useEffect(() => {
    return () => {
      if (pollInterval) window.clearInterval(pollInterval);
    };
  }, [pollInterval]);

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'text-green-400';
      case 'failed': return 'text-red-400';
      case 'cancelled': return 'text-yellow-400';
      case 'processing': return 'text-blue-400';
      case 'queued': return 'text-gray-400';
      case 'cancelling': return 'text-orange-400';
      default: return 'text-gray-400';
    }
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Ingesta de Markdown (cuadernos/vaults/libros)</h1>
        <div className="flex gap-2">
          <button
            onClick={() => setActiveTab('config')}
            className={`px-3 py-1 rounded text-sm ${activeTab === 'config' ? 'bg-primary-600' : 'bg-dark-800'}`}
          >
            Configurar
          </button>
          <button
            onClick={() => { setActiveTab('monitor'); loadJobs(); }}
            className={`px-3 py-1 rounded text-sm ${activeTab === 'monitor' ? 'bg-primary-600' : 'bg-dark-800'}`}
          >
            Monitor
          </button>
          <button
            onClick={() => { setActiveTab('history'); loadJobs(); }}
            className={`px-3 py-1 rounded text-sm ${activeTab === 'history' ? 'bg-primary-600' : 'bg-dark-800'}`}
          >
            Historial
          </button>
        </div>
      </div>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      {activeTab === 'config' && (
        <div className="space-y-6">
          <div className="bg-dark-800 p-6 rounded-lg">
            <h2 className="text-xl font-semibold mb-4">Configuración de ingesta</h2>

            <div className="mb-4">
              <label className="block text-sm font-medium mb-1">Directorio raíz (libros, vault, carpeta con *.md)</label>
              <input
                type="text"
                value={dir}
                onChange={e => setDir(e.target.value)}
                placeholder="/mnt/ws2/rescate_sdb/libros/datos_db_p_procesado/books"
                className="w-full bg-dark-900 border border-gray-700 rounded px-3 py-2 font-mono text-sm"
              />
              <p className="text-xs text-gray-500 mt-1">
                La ingesta recorrerá recursivamente todos los archivos *.md bajo este directorio.
                Cada subdirectorio de primer nivel se convierte en un "topic".
              </p>
            </div>

            <div className="mb-4">
              <label className="block text-sm font-medium mb-1">Topic opcional (sobrescribe el derivado del directorio)</label>
              <input
                type="text"
                value={topic}
                onChange={e => setTopic(e.target.value)}
                placeholder="Ej: fisica-clasica"
                className="w-full bg-dark-900 border border-gray-700 rounded px-3 py-2"
              />
            </div>

            <div className="mb-4">
              <label className="block text-sm font-medium mb-1">Máximo de archivos (límite de seguridad)</label>
              <input
                type="number"
                value={maxFiles}
                onChange={e => setMaxFiles(parseInt(e.target.value) || 5000)}
                min={1}
                max={50000}
                className="w-48 bg-dark-900 border border-gray-700 rounded px-3 py-2"
              />
            </div>

            <div className="flex gap-4">
              <button
                onClick={handleDryRun}
                disabled={loading || !dir.trim()}
                className="px-6 py-2 bg-gray-700 hover:bg-gray-600 text-white font-medium rounded transition disabled:opacity-50"
              >
                {loading ? 'Analizando...' : 'Dry-run (previsualizar)'}
              </button>
              <button
                onClick={handleStart}
                disabled={loading || !dir.trim() || dryRunResult?.files_found === 0}
                className="px-6 py-2 bg-primary-600 hover:bg-primary-500 text-white font-medium rounded transition disabled:opacity-50"
              >
                {loading ? 'Iniciando...' : 'Iniciar ingesta'}
              </button>
            </div>
          </div>

          {dryRunResult && (
            <div className="bg-dark-800 p-6 rounded-lg">
              <h2 className="text-xl font-semibold mb-4">Previsualización (dry-run)</h2>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
                <div className="bg-dark-900 p-3 rounded">
                  <p className="text-gray-500 text-sm">Archivos</p>
                  <p className="text-2xl font-bold">{dryRunResult.files_found}</p>
                </div>
                <div className="bg-dark-900 p-3 rounded">
                  <p className="text-gray-500 text-sm">Tamaño total</p>
                  <p className="text-2xl font-bold">{formatBytes(dryRunResult.total_bytes)}</p>
                </div>
                <div className="bg-dark-900 p-3 rounded">
                  <p className="text-gray-500 text-sm">Topics detectados</p>
                  <p className="text-2xl font-bold">{dryRunResult.topics.length}</p>
                </div>
                <div className="bg-dark-900 p-3 rounded">
                  <p className="text-gray-500 text-sm">Muestra</p>
                  <p className="text-2xl font-bold">{dryRunResult.sample.length}</p>
                </div>
              </div>

              <div className="max-h-96 overflow-y-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="text-left text-gray-500 border-b border-gray-700">
                      <th className="pb-2 pr-4">Archivo</th>
                      <th className="pb-2 pr-4">Topic</th>
                      <th className="pb-2 pr-4">Título</th>
                      <th className="pb-2">Tamaño</th>
                    </tr>
                  </thead>
                  <tbody>
                    {dryRunResult.sample.map((f, i) => (
                      <tr key={i} className="border-b border-gray-800 hover:bg-dark-700">
                        <td className="py-2 pr-4 font-mono text-xs truncate max-w-xs" title={f.path}>{f.path}</td>
                        <td className="py-2 pr-4">{f.topic}</td>
                        <td className="py-2 pr-4">{f.title}</td>
                        <td className="py-2 text-right">{formatBytes(f.bytes)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {dryRunResult.files_found > dryRunResult.sample.length && (
                  <p className="text-gray-500 text-sm mt-2 text-center">
                    ... y {dryRunResult.files_found - dryRunResult.sample.length} archivos más
                  </p>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      {activeTab === 'monitor' && (
        <div className="space-y-6">
          {job && (
            <div className="bg-dark-800 p-6 rounded-lg">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold">Job {job.job_id.slice(0, 8)}…</h2>
                <span className={`px-2 py-1 rounded text-xs font-medium ${getStatusColor(job.status)}`}>
                  {job.status}
                </span>
              </div>

              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
                <div className="bg-dark-900 p-3 rounded">
                  <p className="text-gray-500 text-sm">Progreso</p>
                  <p className="text-2xl font-bold">{job.progress.toFixed(1)}%</p>
                </div>
                <div className="bg-dark-900 p-3 rounded">
                  <p className="text-gray-500 text-sm">Archivos procesados</p>
                  <p className="text-2xl font-bold">{job.files_done} / {job.files_total}</p>
                </div>
                <div className="bg-dark-900 p-3 rounded">
                  <p className="text-gray-500 text-sm">Chunks indexados</p>
                  <p className="text-2xl font-bold">{job.chunks_indexed}</p>
                </div>
                <div className="bg-dark-900 p-3 rounded">
                  <p className="text-gray-500 text-sm">Topics</p>
                  <p className="text-2xl font-bold">{job.topics.length}</p>
                </div>
              </div>

              <div className="w-full bg-dark-700 rounded-full h-3 mb-4">
                <div
                  className="bg-primary-400 h-3 rounded-full transition-all duration-300"
                  style={{ width: `${Math.min(job.progress, 100)}%` }}
                />
              </div>

              {(job.current_file || job.current_topic) && (
                <div className="text-sm text-gray-400 space-y-1">
                  {job.current_topic && <p>Topic actual: <span className="font-mono">{job.current_topic}</span></p>}
                  {job.current_file && <p>Archivo: <span className="font-mono truncate max-w-md">{job.current_file}</span></p>}
                </div>
              )}

              <p className="text-sm text-gray-400 mt-2">{job.message || job.status}</p>

              {(job.status === 'processing' || job.status === 'queued') && (
                <button
                  onClick={() => handleCancel(job.job_id)}
                  className="mt-4 px-4 py-2 bg-red-600 hover:bg-red-500 text-white font-medium rounded transition"
                >
                  Cancelar
                </button>
              )}
            </div>
          )}

          <div className="bg-dark-800 p-6 rounded-lg">
            <h2 className="text-xl font-semibold mb-4">Jobs activos / recientes</h2>
            {jobs.length === 0 ? (
              <p className="text-gray-500 text-center py-8">No hay jobs recientes</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="text-left text-gray-500 border-b border-gray-700">
                      <th className="pb-2 pr-4">Job ID</th>
                      <th className="pb-2 pr-4">Directorio</th>
                      <th className="pb-2 pr-4">Estado</th>
                      <th className="pb-2 pr-4">Progreso</th>
                      <th className="pb-2 pr-4">Archivos</th>
                      <th className="pb-2 pr-4">Chunks</th>
                      <th className="pb-2">Creado</th>
                    </tr>
                  </thead>
                  <tbody>
                    {jobs.map((j, i) => (
                      <tr key={i} className="border-b border-gray-800 hover:bg-dark-700">
                        <td className="py-2 pr-4 font-mono text-xs">{j.job_id.slice(0, 12)}…</td>
                        <td className="py-2 pr-4 truncate max-w-xs" title={j.dir}>{j.dir}</td>
                        <td className="py-2 pr-4">
                          <span className={`px-2 py-0.5 rounded text-xs ${getStatusColor(j.status)}`}>
                            {j.status}
                          </span>
                        </td>
                        <td className="py-2 pr-4">{j.progress.toFixed(1)}%</td>
                        <td className="py-2 pr-4">{j.files_done} / {j.files_total}</td>
                        <td className="py-2 pr-4">{j.chunks_indexed}</td>
                        <td className="py-2 pr-4 text-gray-400">{new Date(j.created_at).toLocaleString()}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>
      )}

      {activeTab === 'history' && (
        <div className="bg-dark-800 p-6 rounded-lg">
          <h2 className="text-xl font-semibold mb-4">Historial de ingesta</h2>
          {jobs.length === 0 ? (
            <p className="text-gray-500 text-center py-8">No hay historial</p>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-left text-gray-500 border-b border-gray-700">
                    <th className="pb-2 pr-4">Job ID</th>
                    <th className="pb-2 pr-4">Directorio</th>
                    <th className="pb-2 pr-4">Estado final</th>
                    <th className="pb-2 pr-4">Archivos indexados</th>
                    <th className="pb-2 pr-4">Chunks</th>
                    <th className="pb-2 pr-4">Topics</th>
                    <th className="pb-2">Creado</th>
                  </tr>
                </thead>
                <tbody>
                  {jobs.map((j, i) => (
                    <tr key={i} className="border-b border-gray-800 hover:bg-dark-700">
                      <td className="py-2 pr-4 font-mono text-xs">{j.job_id.slice(0, 12)}…</td>
                      <td className="py-2 pr-4 truncate max-w-xs" title={j.dir}>{j.dir}</td>
                      <td className="py-2 pr-4">
                        <span className={`px-2 py-0.5 rounded text-xs ${getStatusColor(j.status)}`}>
                          {j.status}
                        </span>
                      </td>
                      <td className="py-2 pr-4">{j.files_indexed}</td>
                      <td className="py-2 pr-4">{j.chunks_indexed}</td>
                      <td className="py-2 pr-4">{j.topics.join(', ')}</td>
                      <td className="py-2 pr-4 text-gray-400">{new Date(j.created_at).toLocaleString()}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  );
}