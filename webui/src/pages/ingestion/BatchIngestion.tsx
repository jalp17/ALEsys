import { useState, useEffect } from 'react';

interface IngestionJob {
  id: string;
  pdf_path: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress: number;
  message?: string;
  output_dir?: string;
}

const ingestionService = {
  async ingestBatch(pdfPaths: string[], config: any): Promise<IngestionJob[]> {
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
};

export function BatchIngestion() {
  const [files, setFiles] = useState<File[]>([]);
  const [jobs, setJobs] = useState<IngestionJob[]>([]);
  const [loading, setLoading] = useState(false);
  const [topic, setTopic] = useState('');
  const [mode, setMode] = useState('auto');

  const handleDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    const dropped = Array.from(e.dataTransfer.files).filter(f => f.type === 'application/pdf');
    setFiles(prev => [...prev, ...dropped]);
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
  };

  const removeFile = (index: number) => {
    setFiles(prev => prev.filter((_, i) => i !== index));
  };

  const handleSubmit = async () => {
    if (files.length === 0) return;
    setLoading(true);
    try {
      const pdfPaths = files.map(f => URL.createObjectURL(f));
      const batchJobs = await ingestionService.ingestBatch(pdfPaths, { topic, mode });
      setJobs(prev => [...prev, ...batchJobs]);
      setFiles([]);
    } catch (err) {
      console.error('Batch ingestion failed', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    const interval = setInterval(async () => {
      const updated = await Promise.all(
        jobs.map(j => ingestionService.getStatus(j.id).catch(() => j))
      );
      setJobs(updated);
    }, 2000);
    return () => clearInterval(interval);
  }, [jobs]);

  return (
    <div className="max-w-5xl mx-auto p-6">
      <h1 className="text-2xl font-bold mb-6">Ingesta por Lotes</h1>

      <div
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        className="border-2 border-dashed border-gray-600 rounded-lg p-8 text-center cursor-pointer hover:border-primary-400 transition"
      >
        <p className="text-gray-400 mb-4">Arrastra múltiples PDFs aquí</p>
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

      <div className="mt-4 flex gap-4">
        <input
          type="text"
          value={topic}
          onChange={e => setTopic(e.target.value)}
          placeholder="Topic (opcional)"
          className="flex-1 bg-dark-800 border border-gray-700 rounded px-3 py-2"
        />
        <select
          value={mode}
          onChange={e => setMode(e.target.value)}
          className="bg-dark-800 border border-gray-700 rounded px-3 py-2"
        >
          <option value="auto">Auto</option>
          <option value="mineru">MinerU</option>
          <option value="pymupdf">PyMuPDF</option>
        </select>
        <button
          onClick={handleSubmit}
          disabled={loading || files.length === 0}
          className="bg-primary-600 hover:bg-primary-500 disabled:bg-gray-600 text-white font-medium px-4 py-2 rounded transition"
        >
          {loading ? 'Encolando...' : `Ingestar ${files.length} PDFs`}
        </button>
      </div>

      <div className="mt-6 space-y-3">
        {jobs.map(job => (
          <div key={job.id} className="bg-dark-800 p-4 rounded">
            <div className="flex justify-between items-center mb-2">
              <span className="font-medium truncate">{job.pdf_path}</span>
              <span className={`text-xs px-2 py-1 rounded ${
                job.status === 'completed' ? 'bg-green-900 text-green-300' :
                job.status === 'failed' ? 'bg-red-900 text-red-300' :
                'bg-yellow-900 text-yellow-300'
              }`}>
                {job.status}
              </span>
            </div>
            <div className="w-full bg-dark-700 rounded-full h-2">
              <div
                className="bg-primary-400 h-2 rounded-full transition"
                style={{ width: `${job.progress}%` }}
              />
            </div>
            {job.message && <p className="text-xs text-gray-400 mt-1">{job.message}</p>}
            {job.output_dir && (
              <a href={job.output_dir} className="text-primary-400 hover:underline text-xs mt-1 block">
                Ver output
              </a>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
