import { useState, useEffect } from 'react';
import axios from 'axios';

const API_BASE = import.meta.env.VITE_API_URL || '';

interface LLMStatus {
  loaded: boolean;
  backend: string;
}

export function LLMStatusIndicator() {
  const [status, setStatus] = useState<LLMStatus | null>(null);

  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const res = await axios.get<LLMStatus>(`${API_BASE}/api/v1/llm/status`);
        setStatus(res.data);
      } catch {
        // Silently ignore - server might be down
      }
    };

    fetchStatus();
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  if (!status) return null;

  return (
    <div
      className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium cursor-pointer transition ${
        status.loaded
          ? 'bg-green-900/40 text-green-400 border border-green-700/50 hover:bg-green-900/60'
          : 'bg-red-900/40 text-red-400 border border-red-700/50 hover:bg-red-900/60'
      }`}
      title={status.loaded ? `LLM cargado (${status.backend})` : 'LLM no cargado — ir a Configuración'}
      onClick={() => window.location.href = '/settings'}
    >
      <span className="w-2 h-2 rounded-full bg-current animate-pulse" />
      LLM {status.loaded ? 'ON' : 'OFF'}
    </div>
  );
}
