import { useState, useEffect, useCallback } from 'react';
import axios from 'axios';

const API_BASE = import.meta.env.VITE_API_URL || '';

// =============================================================================
// Types
// =============================================================================

interface LLMStatus {
  loaded: boolean;
  backend: string;
  state: string;
  model_path: string | null;
  message: string;
}

interface LoadLLMResponse {
  success: boolean;
  backend: string;
  model_path: string;
  estimated_ram_mb: number;
  message: string;
}

interface UnloadLLMResponse {
  success: boolean;
  message: string;
  ram_freed_mb: number | null;
}

// =============================================================================
// LLM Control Component
// =============================================================================

function LLMControl() {
  const [status, setStatus] = useState<LLMStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastAction, setLastAction] = useState<string | null>(null);

  const fetchStatus = useCallback(async () => {
    try {
      const res = await axios.get<LLMStatus>(`${API_BASE}/api/v1/llm/status`);
      setStatus(res.data);
      setError(null);
    } catch (err: any) {
      setError(`Error obteniendo estado: ${err.message}`);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000); // Poll cada 5s
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const loadModel = async () => {
    setLoading(true);
    setError(null);
    setLastAction('Cargando modelo...');
    try {
      const res = await axios.post<LoadLLMResponse>(
        `${API_BASE}/api/v1/llm/load`,
        { force: false }
      );
      setLastAction(`✅ ${res.data.message} (~${res.data.estimated_ram_mb} MB)`);
      fetchStatus();
    } catch (err: any) {
      const msg = err.response?.data?.error || err.message;
      setError(`Error cargando: ${msg}`);
      setLastAction(null);
    }
    setLoading(false);
  };

  const unloadModel = async () => {
    setLoading(true);
    setError(null);
    setLastAction('Descargando modelo...');
    try {
      const res = await axios.post<UnloadLLMResponse>(
        `${API_BASE}/api/v1/llm/unload`
      );
      if (res.data.ram_freed_mb) {
        setLastAction(`✅ ${res.data.message}`);
      } else {
        setLastAction(`ℹ️ ${res.data.message}`);
      }
      fetchStatus();
    } catch (err: any) {
      const msg = err.response?.data?.error || err.message;
      setError(`Error descargando: ${msg}`);
      setLastAction(null);
    }
    setLoading(false);
  };

  const isLoaded = status?.loaded ?? false;

  return (
    <div className="bg-dark-800 rounded-lg p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold">Modelo LLM</h2>
        <span
          className={`px-3 py-1 rounded-full text-sm font-medium ${
            isLoaded
              ? 'bg-green-900/50 text-green-400 border border-green-700'
              : 'bg-red-900/50 text-red-400 border border-red-700'
          }`}
        >
          {isLoaded ? '🟢 Cargado' : '🔴 Descargado'}
        </span>
      </div>

      {/* Status info */}
      {status && (
        <div className="grid grid-cols-2 gap-3 mb-4 text-sm">
          <div className="bg-dark-900 rounded p-3">
            <span className="text-gray-400">Backend:</span>
            <span className="ml-2 text-white font-mono">{status.backend}</span>
          </div>
          <div className="bg-dark-900 rounded p-3">
            <span className="text-gray-400">Estado:</span>
            <span className="ml-2 text-white font-mono">{status.state}</span>
          </div>
          {status.model_path && (
            <div className="col-span-2 bg-dark-900 rounded p-3">
              <span className="text-gray-400">Modelo:</span>
              <span className="ml-2 text-white font-mono text-xs break-all">
                {status.model_path}
              </span>
            </div>
          )}
        </div>
      )}

      {/* RAM estimate */}
      {status && (
        <div className="bg-dark-900 rounded p-3 mb-4 text-sm">
          <span className="text-gray-400">RAM:</span>
          <span className={`ml-2 font-mono ${isLoaded ? 'text-yellow-400' : 'text-green-400'}`}>
            {isLoaded ? '~4 GB en uso' : '~200 MB (sin modelo)'}
          </span>
        </div>
      )}

      {/* Action buttons */}
      <div className="flex gap-3">
        {!isLoaded ? (
          <button
            onClick={loadModel}
            disabled={loading}
            className="flex-1 px-4 py-3 bg-green-600 hover:bg-green-700 disabled:bg-green-800 disabled:cursor-not-allowed rounded-lg transition font-semibold text-white flex items-center justify-center gap-2"
          >
            {loading ? (
              <>
                <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Cargando...
              </>
            ) : (
              '🚀 Cargar Modelo'
            )}
          </button>
        ) : (
          <button
            onClick={unloadModel}
            disabled={loading}
            className="flex-1 px-4 py-3 bg-red-600 hover:bg-red-700 disabled:bg-red-800 disabled:cursor-not-allowed rounded-lg transition font-semibold text-white flex items-center justify-center gap-2"
          >
            {loading ? (
              <>
                <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Descargando...
              </>
            ) : (
              '🛑 Descargar Modelo (Liberar RAM)'
            )}
          </button>
        )}
        <button
          onClick={fetchStatus}
          disabled={loading}
          className="px-4 py-3 bg-dark-700 hover:bg-dark-600 disabled:cursor-not-allowed rounded-lg transition text-gray-300"
          title="Actualizar estado"
        >
          🔄
        </button>
      </div>

      {/* Messages */}
      {error && (
        <div className="mt-3 p-3 bg-red-900/30 border border-red-700 rounded text-red-400 text-sm">
          {error}
        </div>
      )}
      {lastAction && !error && (
        <div className="mt-3 p-3 bg-blue-900/30 border border-blue-700 rounded text-blue-400 text-sm">
          {lastAction}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// Settings Page
// =============================================================================

export function Settings() {
  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">Configuración</h1>

      <div className="space-y-6">
        {/* LLM Control */}
        <LLMControl />

        <div className="bg-dark-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-4">Base de Datos</h2>
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Connection String</label>
              <input
                type="text"
                defaultValue="postgresql://alesys:***@localhost:5432/alesys"
                className="w-full px-3 py-2 bg-dark-900 border border-gray-700 rounded text-white font-mono text-sm"
              />
            </div>
          </div>
        </div>

        <div className="bg-dark-800 rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-4">Embeddings</h2>
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Modelo</label>
              <input
                type="text"
                defaultValue="sentence-transformers/all-MiniLM-L6-v2"
                className="w-full px-3 py-2 bg-dark-900 border border-gray-700 rounded text-white"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Dimensión</label>
              <input
                type="number"
                defaultValue={384}
                className="w-full px-3 py-2 bg-dark-900 border border-gray-700 rounded text-white"
              />
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-3">
          <button className="px-4 py-2 text-gray-400 hover:text-white transition">
            Cancelar
          </button>
          <button className="px-6 py-2 bg-primary-600 rounded hover:bg-primary-700 transition font-semibold">
            Guardar
          </button>
        </div>
      </div>
    </div>
  );
}
