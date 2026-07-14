import React from 'react';

interface DesktopLayoutProps {
  children: React.ReactNode;
}

export function DesktopLayout({ children }: DesktopLayoutProps) {
  // Tauri-specific layout with native window controls
  return (
    <div className="h-screen bg-dark-900 text-white flex flex-col">
      {/* Title bar for Tauri (draggable) */}
      <div 
        data-tauri-drag-region
        className="h-10 bg-dark-800 flex items-center justify-between px-4 select-none"
      >
        <span className="text-sm font-semibold">ALEsys Desktop</span>
        <div className="flex gap-2">
          {/* Window controls would be handled by Tauri commands */}
          <button className="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400"></button>
          <button className="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400"></button>
          <button className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400"></button>
        </div>
      </div>

      {/* Menu bar */}
      <nav className="h-8 bg-dark-800 border-b border-gray-700 flex items-center px-2 text-sm">
        <button className="px-3 py-1 hover:bg-gray-700 rounded">Archivo</button>
        <button className="px-3 py-1 hover:bg-gray-700 rounded">Editar</button>
        <button className="px-3 py-1 hover:bg-gray-700 rounded">Ver</button>
        <button className="px-3 py-1 hover:bg-gray-700 rounded">Ayuda</button>
      </nav>

      {/* Main content */}
      <main className="flex-1 overflow-auto">
        {children}
      </main>
    </div>
  );
}