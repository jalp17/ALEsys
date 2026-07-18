import React from 'react';
import { Link } from 'react-router-dom';

interface WebLayoutProps {
  children: React.ReactNode;
}

export function WebLayout({ children }: WebLayoutProps) {
  return (
    <div className="min-h-screen bg-dark-900 text-white flex flex-col">
      {/* Header */}
      <header className="h-16 bg-dark-800 border-b border-gray-700 flex items-center justify-between px-6">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-bold text-primary-400">ALEsys</h1>
          <span className="text-xs text-gray-400 bg-gray-700 px-2 py-1 rounded">
            GraphRAG-PG
          </span>
        </div>
        
        <nav className="flex gap-6 text-sm">
          <Link to="/chat" className="text-gray-300 hover:text-primary-400 transition">
            Chat
          </Link>
          <Link to="/generate" className="text-gray-300 hover:text-primary-400 transition">
            Generar
          </Link>
          <Link to="/sessions" className="text-gray-300 hover:text-primary-400 transition">
            Sesiones
          </Link>
          <Link to="/graph" className="text-gray-300 hover:text-primary-400 transition">
            Grafo
          </Link>
          <Link to="/search" className="text-gray-300 hover:text-primary-400 transition">
            Búsqueda
          </Link>
          <Link to="/settings" className="text-gray-300 hover:text-primary-400 transition">
            Configuración
          </Link>
        </nav>

        <div className="flex items-center gap-4">
          <span className="text-sm text-gray-400">Usuario</span>
          <button className="px-3 py-1.5 bg-red-600 rounded hover:bg-red-700 text-sm transition">
            Cerrar sesión
          </button>
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 p-6 overflow-auto">
        {children}
      </main>

      {/* Footer */}
      <footer className="h-8 bg-dark-800 border-t border-gray-700 flex items-center justify-between px-6 text-xs text-gray-500">
        <span>ALEsys v{import.meta.env.VITE_APP_VERSION || '0.1.0'}</span>
        <span>GraphRAG-PG Engine</span>
      </footer>
    </div>
  );
}