/**
 * Sidebar with search filters
 */

import type { SearchFilters } from './searchService';
import { getAvailableDocTypes, getAvailableAreas } from './searchService';

interface SearchSidebarProps {
  filters: SearchFilters;
  onFiltersChange: (filters: SearchFilters) => void;
  isOpen: boolean;
  onToggle: () => void;
}

const DOC_TYPES = getAvailableDocTypes();
const AREAS = getAvailableAreas();

export function SearchSidebar({
  filters,
  onFiltersChange,
  isOpen,
  onToggle,
}: SearchSidebarProps) {
  const handleDocTypeToggle = (type: string) => {
    const newTypes = filters.doc_types.includes(type)
      ? filters.doc_types.filter((t) => t !== type)
      : [...filters.doc_types, type];
    onFiltersChange({ ...filters, doc_types: newTypes });
  };

  const handleAreaToggle = (areaId: number) => {
    const newAreas = filters.areas.includes(areaId)
      ? filters.areas.filter((a) => a !== areaId)
      : [...filters.areas, areaId];
    onFiltersChange({ ...filters, areas: newAreas });
  };

  const handleDateFromChange = (value: string) => {
    onFiltersChange({ ...filters, date_from: value || undefined });
  };

  const handleDateToChange = (value: string) => {
    onFiltersChange({ ...filters, date_to: value || undefined });
  };

  const handlePatternChange = (value: string) => {
    onFiltersChange({ ...filters, content_pattern: value || undefined });
  };

  const clearFilters = () => {
    onFiltersChange({
      doc_types: [],
      areas: [],
      subareas: [],
    });
  };

  const hasActiveFilters =
    filters.doc_types.length > 0 ||
    filters.areas.length > 0 ||
    filters.date_from ||
    filters.date_to ||
    filters.content_pattern;

  return (
    <>
      {/* Mobile toggle button */}
      <button
        onClick={onToggle}
        className="lg:hidden fixed bottom-4 left-4 z-50 bg-primary-600 text-white p-3 rounded-full shadow-lg hover:bg-primary-700 transition"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
        </svg>
      </button>

      {/* Sidebar */}
      <aside
        className={`
          fixed lg:static inset-y-0 left-0 z-40
          w-72 bg-dark-800 border-r border-gray-700
          transform transition-transform duration-200
          ${isOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
          overflow-y-auto
        `}
      >
        <div className="p-4 space-y-6">
          {/* Header */}
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-gray-300 uppercase tracking-wider">
              Filtros
            </h3>
            {hasActiveFilters && (
              <button
                onClick={clearFilters}
                className="text-xs text-primary-400 hover:text-primary-300"
              >
                Limpiar
              </button>
            )}
          </div>

          {/* Document Types */}
          <div className="space-y-2">
            <label className="text-xs font-medium text-gray-400 uppercase">
              Tipo de documento
            </label>
            <div className="space-y-1">
              {DOC_TYPES.map((type) => (
                <label
                  key={type}
                  className="flex items-center gap-2 text-sm text-gray-300 hover:text-white cursor-pointer"
                >
                  <input
                    type="checkbox"
                    checked={filters.doc_types.includes(type)}
                    onChange={() => handleDocTypeToggle(type)}
                    className="rounded border-gray-600 bg-dark-700 text-primary-500 focus:ring-primary-500"
                  />
                  <span className="capitalize">{type}</span>
                </label>
              ))}
            </div>
          </div>

          {/* Areas */}
          <div className="space-y-2">
            <label className="text-xs font-medium text-gray-400 uppercase">
              Área
            </label>
            <div className="space-y-1">
              {AREAS.map((area) => (
                <label
                  key={area.id}
                  className="flex items-center gap-2 text-sm text-gray-300 hover:text-white cursor-pointer"
                >
                  <input
                    type="checkbox"
                    checked={filters.areas.includes(area.id)}
                    onChange={() => handleAreaToggle(area.id)}
                    className="rounded border-gray-600 bg-dark-700 text-primary-500 focus:ring-primary-500"
                  />
                  <span>{area.name}</span>
                </label>
              ))}
            </div>
          </div>

          {/* Date Range */}
          <div className="space-y-2">
            <label className="text-xs font-medium text-gray-400 uppercase">
              Rango de fechas
            </label>
            <div className="grid grid-cols-2 gap-2">
              <input
                type="date"
                value={filters.date_from || ''}
                onChange={(e) => handleDateFromChange(e.target.value)}
                className="bg-dark-700 border border-gray-600 rounded px-2 py-1 text-sm text-gray-300 focus:border-primary-500 focus:outline-none"
                placeholder="Desde"
              />
              <input
                type="date"
                value={filters.date_to || ''}
                onChange={(e) => handleDateToChange(e.target.value)}
                className="bg-dark-700 border border-gray-600 rounded px-2 py-1 text-sm text-gray-300 focus:border-primary-500 focus:outline-none"
                placeholder="Hasta"
              />
            </div>
          </div>

          {/* Path Pattern */}
          <div className="space-y-2">
            <label className="text-xs font-medium text-gray-400 uppercase">
              Patrón de ruta
            </label>
            <input
              type="text"
              value={filters.content_pattern || ''}
              onChange={(e) => handlePatternChange(e.target.value)}
              placeholder="ej: docs/"
              className="w-full bg-dark-700 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-300 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
            />
          </div>

          {/* Active filters summary */}
          {hasActiveFilters && (
            <div className="pt-4 border-t border-gray-700">
              <p className="text-xs text-gray-500">
                {filters.doc_types.length + filters.areas.length} filtros activos
              </p>
            </div>
          )}
        </div>
      </aside>

      {/* Overlay for mobile */}
      {isOpen && (
        <div
          className="fixed inset-0 bg-black/50 z-30 lg:hidden"
          onClick={onToggle}
        />
      )}
    </>
  );
}
