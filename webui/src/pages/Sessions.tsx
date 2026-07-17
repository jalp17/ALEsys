import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { API_BASE_URL } from '../utils/platform';
import { useSessionStore } from '../store/session';

export function Sessions() {
  const { sessions, activeSessionId, setSessions, addSession, removeSession, setActiveSession } =
    useSessionStore();
  const [newName, setNewName] = useState('');
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    fetchSessions();
  }, []);

  const fetchSessions = async () => {
    setLoading(true);
    try {
      const res = await fetch(`${API_BASE_URL}/api/sessions`);
      const data = await res.json();
      setSessions(data.sessions || []);
    } catch (e) {
      console.error('Error fetching sessions:', e);
    } finally {
      setLoading(false);
    }
  };

  const createSession = async () => {
    try {
      const res = await fetch(`${API_BASE_URL}/api/sessions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newName || undefined }),
      });
      const data = await res.json();
      if (data.session_id) {
        addSession({
          id: data.session_id,
          name: newName || `Sesion ${new Date().toLocaleString()}`,
          created_at: new Date().toISOString(),
          last_activity: new Date().toISOString(),
          is_active: true,
        });
        setNewName('');
      }
    } catch (e) {
      console.error('Error creating session:', e);
    }
  };

  const deleteSession = async (id: string) => {
    try {
      await fetch(`${API_BASE_URL}/api/sessions/${id}`, { method: 'DELETE' });
      removeSession(id);
    } catch (e) {
      console.error('Error deleting session:', e);
    }
  };

  const selectSession = (id: string) => {
    setActiveSession(id);
    navigate('/chat');
  };

  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleString();
    } catch {
      return iso;
    }
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-white">Sesiones</h2>
        <span className="text-sm text-gray-400">{sessions.length} activas</span>
      </div>

      {/* Create form */}
      <div className="flex gap-3 mb-6">
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="Nombre de la sesion (opcional)"
          className="flex-1 px-4 py-2 bg-dark-800 border border-gray-700 rounded-lg focus:outline-none focus:border-primary-500 text-white placeholder-gray-500"
        />
        <button
          onClick={createSession}
          className="px-4 py-2 bg-primary-600 rounded-lg hover:bg-primary-700 transition font-semibold"
        >
          Nueva sesion
        </button>
      </div>

      {/* Sessions list */}
      {loading ? (
        <div className="text-center text-gray-400 py-10">Cargando...</div>
      ) : sessions.length === 0 ? (
        <div className="text-center text-gray-400 py-10">
          <p className="text-lg mb-2">No hay sesiones activas</p>
          <p className="text-sm">Crea una nueva para empezar a chatear con historial persistente</p>
        </div>
      ) : (
        <div className="space-y-3">
          {sessions.map((session) => (
            <div
              key={session.id}
              className={`p-4 rounded-lg border transition cursor-pointer ${
                activeSessionId === session.id
                  ? 'bg-primary-900/30 border-primary-500'
                  : 'bg-dark-800 border-gray-700 hover:border-gray-500'
              }`}
              onClick={() => selectSession(session.id)}
            >
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-white font-medium">{session.name}</h3>
                  <p className="text-xs text-gray-400 mt-1">
                    Creada: {formatDate(session.created_at)} &middot; Ultima actividad:{' '}
                    {formatDate(session.last_activity)}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <span
                    className={`text-xs px-2 py-1 rounded ${
                      session.is_active ? 'bg-green-900 text-green-300' : 'bg-gray-700 text-gray-400'
                    }`}
                  >
                    {session.is_active ? 'Activa' : 'Cerrada'}
                  </span>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteSession(session.id);
                    }}
                    className="px-2 py-1 text-xs bg-red-900 text-red-300 rounded hover:bg-red-800 transition"
                  >
                    Cerrar
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
