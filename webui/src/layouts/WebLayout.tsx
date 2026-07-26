import React from 'react';
import { Link } from 'react-router-dom';
import { AgentStatusIndicator } from '../components/AgentStatusIndicator';
import { LLMStatusIndicator } from '../components/LLMStatusIndicator';

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
          <Link to="/editor" className="text-gray-300 hover:text-primary-400 transition">
            Editor
          </Link>
          <Link to="/agents" className="text-gray-300 hover:text-primary-400 transition">
            Agentes
          </Link>
          <Link to="/plugins" className="text-gray-300 hover:text-primary-400 transition">
            Plugins
          </Link>
          <Link to="/orchestrator" className="text-gray-300 hover:text-primary-400 transition">
            Orchestrator
          </Link>
          <Link to="/collaboration" className="text-gray-300 hover:text-primary-400 transition">
            Collaborate
          </Link>
          <Link to="/learning" className="text-gray-300 hover:text-primary-400 transition">
            Learning
          </Link>
          <Link to="/debug" className="text-gray-300 hover:text-primary-400 transition">
            Debug
          </Link>
          <Link to="/test-generation" className="text-gray-300 hover:text-primary-400 transition">
            Tests
          </Link>
          <Link to="/refactoring" className="text-gray-300 hover:text-primary-400 transition">
            Refactor
          </Link>
          <Link to="/kb-curation" className="text-gray-300 hover:text-primary-400 transition">
            KB Curation
          </Link>
          <Link to="/multi-agent" className="text-gray-300 hover:text-primary-400 transition">
            Agents
          </Link>
          <Link to="/analytics" className="text-gray-300 hover:text-primary-400 transition">
            Analytics
          </Link>
          <Link to="/workflows" className="text-gray-300 hover:text-primary-400 transition">
            Workflows
          </Link>
          <Link to="/advanced-search" className="text-gray-300 hover:text-primary-400 transition">
            Search
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
          <Link to="/ingestion" className="text-gray-300 hover:text-primary-400 transition">
            Ingesta
          </Link>
        </nav>

        <div className="flex items-center gap-4">
          <LLMStatusIndicator />
          <AgentStatusIndicator />
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