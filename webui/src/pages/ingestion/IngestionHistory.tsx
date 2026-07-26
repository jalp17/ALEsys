import { useState, useEffect } from 'react';

interface IngestionJob {
  id: string;
  pdf_path: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress: number;
  message?: string;
  output_dir?: string;
  created_at?: string;
}

const ingestionService = {
  async listHistory(): Promise<IngestionJob[]> {
    const res = await fetch('/api/v1/ingestion/history');
    return res.json();
  },

  async getStatus(jobId: string): Promise<IngestionJob> {
    const res = await fetch(`/api/v1/ingestion/status/${jobId}`);
    return res.json();
  },
};

export function IngestionHistory() {
  const [jobs, setJobs] = useState<IngestionJob[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    ingestionService.listHistory()
      .then(data => { setJobs(data); setLoading(false); })
      .catch(() => setLoading(false));
  }, []);

  useEffect(() => {
    const interval = setInterval(async () => {
      const updated = await Promise.all(
        jobs.map(j => ingestionService.getStatus(j.id).catch(() => j))
      );
      setJobs(updated);
    }, 3000);
    return () => clearInterval(interval);
  }, [jobs]);

  if (loading) {
    return <div className="p-6 text-gray-400">Cargando historial...</div>;
  }

  return (
    <div className="max-w-5xl mx-auto p-6">
      <h1 className="text-2xl font-bold mb-6">Historial de Ingesta</h1>

      {jobs.length === 0 ? (
        <p className="text-gray-400">No hay jobs de ingesta aún.</p>
      ) : (
        <div className="space-y-3">
          {jobs.map(job => (
            <div key={job.id} className="bg-dark-800 p-4 rounded flex flex-col gap-2">
              <div className="flex justify-between items-start">
                <div className="flex-1 min-w-0">
                  <p className="font-medium truncate">{job.pdf_path}</p>
                  {job.created_at && (
                    <p className="text-xs text-gray-500">{new Date(job.created_at).toLocaleString()}</p>
                  )}
                </div>
                <span className={`text-xs px-2 py-1 rounded ml-4 ${
                  job.status === 'completed' ? 'bg-green-900 text-green-300' :
                  job.status === 'failed' ? 'bg-red-900 text-red-300' :
                  job.status === 'processing' ? 'bg-yellow-900 text-yellow-300' :
                  'bg-gray-700 text-gray-300'
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

              {job.message && (
                <p className="text-sm text-gray-400">{job.message}</p>
              )}

              <div className="flex gap-4">
                {job.output_dir && (
                  <a
                    href={job.output_dir}
                    className="text-primary-400 hover:underline text-sm"
                  >
                    Ver output
                  </a>
                )}
                <span className="text-xs text-gray-500">ID: {job.id}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
