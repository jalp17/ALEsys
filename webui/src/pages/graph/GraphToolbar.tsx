import { useState } from 'react';
import type { LayoutName } from './GraphCanvas';

interface GraphToolbarProps {
  layout: LayoutName;
  onLayoutChange: (layout: LayoutName) => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFit: () => void;
  onReload: () => void;
  loading: boolean;
  docTypeFilter: string;
  onDocTypeFilterChange: (v: string) => void;
  edgeTypeFilter: string;
  onEdgeTypeFilterChange: (v: string) => void;
  onExportPNG?: () => void;
  onExportSVG?: () => void;
  onExportJSON?: () => void;
}

const LAYOUTS: { value: LayoutName; label: string }[] = [
  { value: 'cose-bilkent', label: 'Force-directed' },
  { value: 'circle', label: 'Circulo' },
  { value: 'concentric', label: 'Concentrico' },
  { value: 'breadthfirst', label: 'Jerarquico' },
  { value: 'grid', label: 'Grilla' },
  { value: 'spread', label: 'Distribuido' },
];

export function GraphToolbar({
  layout,
  onLayoutChange,
  onZoomIn,
  onZoomOut,
  onFit,
  onReload,
  loading,
  docTypeFilter,
  onDocTypeFilterChange,
  edgeTypeFilter,
  onEdgeTypeFilterChange,
  onExportPNG,
  onExportSVG,
  onExportJSON,
}: GraphToolbarProps) {
  const [showExportMenu, setShowExportMenu] = useState(false);

  return (
    <div className="h-12 bg-dark-800 border-b border-gray-700 flex items-center px-4 gap-3">
      {/* Layout selector */}
      <div className="flex items-center gap-2">
        <label className="text-xs text-gray-400">Layout:</label>
        <select
          value={layout}
          onChange={(e) => onLayoutChange(e.target.value as LayoutName)}
          className="bg-dark-700 border border-gray-600 rounded px-2 py-1 text-xs text-white focus:outline-none focus:border-primary-400"
        >
          {LAYOUTS.map((l) => (
            <option key={l.value} value={l.value}>
              {l.label}
            </option>
          ))}
        </select>
      </div>

      <div className="w-px h-6 bg-gray-700" />

      {/* Zoom controls */}
      <div className="flex items-center gap-1">
        <button
          onClick={onZoomIn}
          className="w-7 h-7 flex items-center justify-center rounded bg-dark-700 hover:bg-dark-600 text-gray-300 text-sm transition"
          title="Zoom in"
        >
          +
        </button>
        <button
          onClick={onZoomOut}
          className="w-7 h-7 flex items-center justify-center rounded bg-dark-700 hover:bg-dark-600 text-gray-300 text-sm transition"
          title="Zoom out"
        >
          -
        </button>
        <button
          onClick={onFit}
          className="w-7 h-7 flex items-center justify-center rounded bg-dark-700 hover:bg-dark-600 text-gray-300 text-xs transition"
          title="Fit to screen"
        >
          []
        </button>
      </div>

      <div className="w-px h-6 bg-gray-700" />

      {/* Filters */}
      <div className="flex items-center gap-2">
        <label className="text-xs text-gray-400">Tipo doc:</label>
        <select
          value={docTypeFilter}
          onChange={(e) => onDocTypeFilterChange(e.target.value)}
          className="bg-dark-700 border border-gray-600 rounded px-2 py-1 text-xs text-white focus:outline-none focus:border-primary-400"
        >
          <option value="">Todos</option>
          <option value="markdown">Markdown</option>
          <option value="code">Codigo</option>
          <option value="pdf">PDF</option>
        </select>
      </div>

      <div className="flex items-center gap-2">
        <label className="text-xs text-gray-400">Tipo enlace:</label>
        <select
          value={edgeTypeFilter}
          onChange={(e) => onEdgeTypeFilterChange(e.target.value)}
          className="bg-dark-700 border border-gray-600 rounded px-2 py-1 text-xs text-white focus:outline-none focus:border-primary-400"
        >
          <option value="">Todos</option>
          <option value="wiki_link">Wiki Link</option>
          <option value="backlink">Backlink</option>
          <option value="reference">Reference</option>
        </select>
      </div>

      <div className="flex-1" />

      {/* Export + reload */}
      <div className="flex items-center gap-1 relative">
        <button
          onClick={onReload}
          disabled={loading}
          className="px-3 py-1.5 text-xs bg-dark-700 hover:bg-dark-600 rounded text-gray-300 transition disabled:opacity-50"
        >
          {loading ? 'Cargando...' : 'Recargar'}
        </button>

        <div className="relative">
          <button
            onClick={() => setShowExportMenu(!showExportMenu)}
            className="px-3 py-1.5 text-xs bg-primary-600 hover:bg-primary-500 rounded text-white transition"
          >
            Exportar
          </button>
          {showExportMenu && (
            <div className="absolute right-0 top-full mt-1 bg-dark-700 border border-gray-600 rounded shadow-lg z-50 min-w-[120px]">
              {onExportPNG && (
                <button
                  onClick={() => { onExportPNG(); setShowExportMenu(false); }}
                  className="w-full px-3 py-2 text-xs text-left text-gray-300 hover:bg-dark-600 transition"
                >
                  PNG (imagen)
                </button>
              )}
              {onExportSVG && (
                <button
                  onClick={() => { onExportSVG(); setShowExportMenu(false); }}
                  className="w-full px-3 py-2 text-xs text-left text-gray-300 hover:bg-dark-600 transition"
                >
                  SVG (vector)
                </button>
              )}
              {onExportJSON && (
                <button
                  onClick={() => { onExportJSON(); setShowExportMenu(false); }}
                  className="w-full px-3 py-2 text-xs text-left text-gray-300 hover:bg-dark-600 transition border-t border-gray-600"
                >
                  JSON (datos)
                </button>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
