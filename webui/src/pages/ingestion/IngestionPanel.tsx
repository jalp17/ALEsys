import { useState, useCallback } from 'react';

interface IngestionConfig {
  topic: string;
  mode: 'mineru' | 'pymupdf' | 'auto';
  ocr: boolean;
  formulas: boolean;
  output_dir?: string;
}

interface IngestionJob {
  id: string;
  pdf_path: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress: number;
  message?: string;
  output_dir?: string;
}

const ingestionService = {
  async ingestPdf(pdfPath: string, config: IngestionConfig): Promise<IngestionJob> {
    const res = await fetch('/api/v1/ingestion/pdf', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pdf_path: pdfPath, ...config }),
    });
    return res.json();
  },

  async ingestBatch(pdfPaths: string[], config: IngestionConfig): Promise<IngestionJob[]> {
    const res = await fetch('/api/v1/ingestion/batch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pdf_paths: pdfPaths, ...config }),
    });
    return res.json();
  },

  async getStatus(jobId: string): Promise<IngestionJob> {
    const res = await fetch(`/api/v1/ingestion/status/${jobId}`);
    return res.json();
  },

  async listHistory(): Promise<IngestionJob[]> {
    const res = await fetch('/api/v1/ingestion/history');
    return res.json();
  },
};

export function IngestionPanel() {
  const [files, setFiles] = useState<File[]>([]);
  const [topic, setTopic] = useState('');
  const [mode, setMode] = useState<IngestionConfig['mode']>('auto');
  const [ocr, setOcr] = useState(true);
  const [formulas, setFormulas] = useState(true);
  const [job, setJob] = useState<IngestionJob | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleDrop = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    const dropped = Array.from(e.dataTransfer.files).filter(f => f.type === 'application/pdf');
    setFiles(prev => [...prev, ...dropped]);
  }, []);

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
  };

  const removeFile = (index: number) => {
    setFiles(prev => prev.filter((_, i) => i !== index));
  };

  const handleSubmit = async () => {
    if (files.length === 0) {
      setError('Selecciona al menos un PDF');
      return;
    }
    if (!topic.trim()) {
      setError('Ingresa un topic');
      return;
    }
    setLoading(true);
    setError('');
    try {
      const pdfPath = URL.createObjectURL(files[0]);
      const result = await ingestionService.ingestPdf(pdfPath, {
        topic,
        mode,
        ocr,
        formulas,
      });
      setJob(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error al iniciar ingesta');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <h1 className="text-2xl font-bold mb-6">Ingesta de PDFs</h1>

      <div
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        className="border-2 border-dashed border-gray-600 rounded-lg p-8 text-center cursor-pointer hover:border-primary-400 transition"
      >
        <p className="text-gray-400 mb-4">Arrastra PDFs aquí o haz clic para seleccionar</p>
        <input
          type="file"
          accept="application/pdf"
          multiple
          onChange={e => setFiles(prev => [...prev, ...Array.from(e.target.files || [])])}
          className="hidden"
        />
        {files.length > 0 && (
          <ul className="text-left mt-4 space-y-2">
            {files.map((f, i) => (
              <li key={i} className="flex justify-between items-center bg-dark-800 p-2 rounded">
                <span>{f.name}</span>
                <button onClick={() => removeFile(i)} className="text-red-400 hover:text-red-300">×</button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="mt-6 space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Topic</label>
          <input
            type="text"
            value={topic}
            onChange={e => setTopic(e.target.value)}
            placeholder="Ej: machine-learning"
            className="w-full bg-dark-800 border border-gray-700 rounded px-3 py-2"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Modo de ingesta</label>
          <select
            value={mode}
            onChange={e => setMode(e.target.value as IngestionConfig['mode'])}
            className="w-full bg-dark-800 border border-gray-700 rounded px-3 py-2"
          >
            <option value="auto">Auto</option>
            <option value="mineru">MinerU</option>
            <option value="pymupdf">PyMuPDF</option>
          </select>
        </div>

        <div className="flex gap-4">
          <label className="flex items-center gap-2">
            <input type="checkbox" checked={ocr} onChange={e => setOcr(e.target.checked)} />
            <span>OCR</span>
          </label>
          <label className="flex items-center gap-2">
            <input type="checkbox" checked={formulas} onChange={e => setFormulas(e.target.checked)} />
            <span>Fórmulas LaTeX</span>
          </label>
        </div>

        {error && <p className="text-red-400 text-sm">{error}</p>}

        <button
          onClick={handleSubmit}
          disabled={loading}
          className="w-full bg-primary-600 hover:bg-primary-500 disabled:bg-gray-600 text-white font-medium py-2 rounded transition"
        >
          {loading ? 'Procesando...' : 'Iniciar Ingesta'}
        </button>
      </div>

      {job && (
        <div className="mt-6 bg-dark-800 p-4 rounded">
          <h2 className="font-semibold mb-2">Job {job.id}</h2>
          <div className="w-full bg-dark-700 rounded-full h-2">
            <div
              className="bg-primary-400 h-2 rounded-full transition"
              style={{ width: `${job.progress}%` }}
            />
          </div>
          <p className="text-sm text-gray-400 mt-2">{job.message || job.status}</p>
          {job.output_dir && (
            <a href={job.output_dir} className="text-primary-400 hover:underline text-sm mt-2 block">
              Ver output
            </a>
          )}
        </div>
      )}
    </div>
  );
}
