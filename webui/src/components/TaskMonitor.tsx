import { useState, useEffect } from 'react';
import axios from 'axios';
import { Loader2, CheckCircle, XCircle, Clock, ChevronDown, ChevronUp } from 'lucide-react';

const API_BASE = import.meta.env.VITE_API_URL || '';

interface Job {
  job_id: string;
  dir: string;
  status: string;
  progress: number;
  files_done: number;
  files_total: number;
  chunks_indexed: number;
  current_topic?: string;
  message?: string;
  error?: string;
  created_at: string;
}

export function TaskMonitor() {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [expanded, setExpanded] = useState(false);
  const [loading, setLoading] = useState(false);

  const fetchJobs = async () => {
    try {
      const token = localStorage.getItem('alesys_token');
      const res = await axios.get<{ jobs: Job[] }>(`${API_BASE}/api/v1/ingestion/markdown/jobs`, {
        headers: token ? { Authorization: `Bearer ${token}` } : {},
      });
      setJobs(res.data.jobs || []);
    } catch {
      // Silently ignore
    }
  };

  useEffect(() => {
    fetchJobs();
    const interval = setInterval(fetchJobs, 5000);
    return () => clearInterval(interval);
  }, []);

  const activeJobs = jobs.filter(j => j.status === 'processing' || j.status === 'queued' || j.status === 'cancelling');
  const completedJobs = jobs.filter(j => j.status === 'completed');
  const failedJobs = jobs.filter(j => j.status === 'failed' || j.status === 'cancelled');

  if (jobs.length === 0) return null;

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'processing':
      case 'queued':
        return <Loader2 className="h-3 w-3 animate-spin text-blue-400" />;
      case 'completed':
        return <CheckCircle className="h-3 w-3 text-green-400" />;
      case 'failed':
      case 'cancelled':
        return <XCircle className="h-3 w-3 text-red-400" />;
      default:
        return <Clock className="h-3 w-3 text-gray-400" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'processing':
      case 'queued':
        return 'text-blue-400';
      case 'completed':
        return 'text-green-400';
      case 'failed':
      case 'cancelled':
        return 'text-red-400';
      default:
        return 'text-gray-400';
    }
  };

  return (
    <div className="relative">
      <button
        onClick={() => setExpanded(!expanded)}
        className={`flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium transition border ${
          activeJobs.length > 0
            ? 'bg-blue-900/40 text-blue-400 border-blue-700/50 hover:bg-blue-900/60'
            : 'bg-gray-800/40 text-gray-400 border-gray-700/50 hover:bg-gray-800/60'
        }`}
        title={activeJobs.length > 0 ? `${activeJobs.length} tarea(s) activa(s)` : 'Tareas completadas'}
      >
        {activeJobs.length > 0 ? (
          <Loader2 className="h-3 w-3 animate-spin" />
        ) : (
          <CheckCircle className="h-3 w-3" />
        )}
        <span>Tareas {activeJobs.length > 0 ? `(${activeJobs.length})` : ''}</span>
        {expanded ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
      </button>

      {expanded && (
        <div className="absolute right-0 top-full mt-2 w-96 max-h-96 overflow-y-auto rounded-lg border border-gray-700 bg-dark-800 shadow-xl z-50">
          <div className="p-3 border-b border-gray-700">
            <h3 className="text-sm font-semibold text-gray-200">Tareas de Indexación</h3>
            <p className="text-xs text-gray-500">{activeJobs.length} activa(s), {completedJobs.length} completada(s), {failedJobs.length} fallida(s)</p>
          </div>

          {jobs.length === 0 ? (
            <div className="p-4 text-center text-gray-500 text-sm">No hay tareas</div>
          ) : (
            <div className="divide-y divide-gray-700/50">
              {jobs.map((job) => (
                <div key={job.job_id} className="p-3 hover:bg-dark-700/50">
                  <div className="flex items-center gap-2 mb-1">
                    {getStatusIcon(job.status)}
                    <span className={`text-xs font-medium ${getStatusColor(job.status)}`}>
                      {job.status}
                    </span>
                    <span className="text-xs text-gray-500 ml-auto">
                      {new Date(job.created_at).toLocaleTimeString()}
                    </span>
                  </div>

                  <div className="text-xs text-gray-400 truncate" title={job.dir}>
                    {job.dir.split('/').pop() || job.dir}
                  </div>

                  {(job.status === 'processing' || job.status === 'queued') && (
                    <div className="mt-2">
                      <div className="flex justify-between text-[10px] text-gray-500 mb-1">
                        <span>{job.files_done} / {job.files_total} archivos</span>
                        <span>{job.chunks_indexed} chunks</span>
                      </div>
                      <div className="w-full bg-dark-700 rounded-full h-1.5">
                        <div
                          className="bg-blue-400 h-1.5 rounded-full transition-all duration-300"
                          style={{ width: `${Math.min(job.progress, 100)}%` }}
                        />
                      </div>
                      {job.current_topic && (
                        <p className="text-[10px] text-gray-500 mt-1 truncate">
                          Topic: {job.current_topic}
                        </p>
                      )}
                    </div>
                  )}

                  {job.status === 'completed' && (
                    <div className="text-[10px] text-green-400 mt-1">
                      {job.files_done} archivos, {job.chunks_indexed} chunks indexados
                    </div>
                  )}

                  {job.error && (
                    <div className="text-[10px] text-red-400 mt-1 truncate">
                      {job.error}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
