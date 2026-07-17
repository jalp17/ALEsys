import { useState } from 'react';
import type { GraphStats, CentralityResponse, CommunitiesResponse } from './graphService';

interface GraphSidebarProps {
  stats: GraphStats | null;
  centrality: CentralityResponse | null;
  communities: CommunitiesResponse | null;
  selectedMetric: 'pagerank' | 'betweenness' | 'degree' | null;
  onMetricChange: (metric: 'pagerank' | 'betweenness' | 'degree' | null) => void;
  onCommunityClick?: (communityId: number) => void;
  onNodeFocus?: (nodeId: string) => void;
}

export function GraphSidebar({
  stats,
  centrality,
  communities,
  selectedMetric,
  onMetricChange,
  onCommunityClick,
  onNodeFocus,
}: GraphSidebarProps) {
  const [expandedSection, setExpandedSection] = useState<string | null>('stats');

  const toggleSection = (section: string) => {
    setExpandedSection(expandedSection === section ? null : section);
  };

  return (
    <div className="w-72 bg-dark-800 border-r border-gray-700 flex flex-col overflow-y-auto">
      {/* Header */}
      <div className="p-4 border-b border-gray-700">
        <h2 className="text-sm font-semibold text-gray-300 uppercase tracking-wide">
          Info del Grafo
        </h2>
      </div>

      {/* Stats Section */}
      <div className="border-b border-gray-700">
        <button
          onClick={() => toggleSection('stats')}
          className="w-full px-4 py-3 flex items-center justify-between text-sm text-gray-300 hover:bg-dark-700 transition"
        >
          <span>Estadisticas</span>
          <span className="text-xs text-gray-500">
            {expandedSection === stats ? 'v' : '>'}
          </span>
        </button>
        {expandedSection === 'stats' && stats && (
          <div className="px-4 pb-4 space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Nodos</span>
              <span className="text-white font-mono">{stats.total_nodes}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Aristas</span>
              <span className="text-white font-mono">{stats.total_edges}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Densidad</span>
              <span className="text-white font-mono">{stats.density.toFixed(4)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Grado promedio</span>
              <span className="text-white font-mono">{stats.avg_degree.toFixed(2)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Comunidades</span>
              <span className="text-white font-mono">{stats.num_communities}</span>
            </div>
          </div>
        )}
      </div>

      {/* Centrality Section */}
      <div className="border-b border-gray-700">
        <button
          onClick={() => toggleSection('centrality')}
          className="w-full px-4 py-3 flex items-center justify-between text-sm text-gray-300 hover:bg-dark-700 transition"
        >
          <span>Centralidad</span>
          <span className="text-xs text-gray-500">
            {expandedSection === 'centrality' ? 'v' : '>'}
          </span>
        </button>
        {expandedSection === 'centrality' && (
          <div className="px-4 pb-4 space-y-2">
            <div className="flex gap-1">
              {(['pagerank', 'betweenness', 'degree'] as const).map((m) => (
                <button
                  key={m}
                  onClick={() => onMetricChange(selectedMetric === m ? null : m)}
                  className={`px-2 py-1 text-xs rounded transition ${
                    selectedMetric === m
                      ? 'bg-primary-500 text-white'
                      : 'bg-dark-600 text-gray-400 hover:bg-dark-500'
                  }`}
                >
                  {m === 'pagerank' ? 'PR' : m === 'betweenness' ? 'BC' : 'DC'}
                </button>
              ))}
            </div>
            {centrality && (
              <div className="space-y-1 max-h-48 overflow-y-auto">
                {centrality.values.slice(0, 10).map((v) => (
                  <button
                    key={v.node_id}
                    onClick={() => onNodeFocus?.(v.node_id)}
                    className="w-full flex items-center justify-between px-2 py-1 text-xs rounded hover:bg-dark-600 transition text-left"
                  >
                    <span className="text-gray-300 truncate flex-1">
                      {v.node_id.replace('doc:', '#')}
                    </span>
                    <span className="text-primary-400 font-mono ml-2">
                      {v.score.toFixed(4)}
                    </span>
                  </button>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Communities Section */}
      <div className="flex-1">
        <button
          onClick={() => toggleSection('communities')}
          className="w-full px-4 py-3 flex items-center justify-between text-sm text-gray-300 hover:bg-dark-700 transition"
        >
          <span>Comunidades</span>
          <span className="text-xs text-gray-500">
            {expandedSection === 'communities' ? 'v' : '>'}
          </span>
        </button>
        {expandedSection === 'communities' && communities && (
          <div className="px-4 pb-4 space-y-1 max-h-64 overflow-y-auto">
            {communities.communities.map((comm) => (
              <button
                key={comm.id}
                onClick={() => onCommunityClick?.(comm.id)}
                className="w-full flex items-center justify-between px-2 py-1.5 text-xs rounded hover:bg-dark-600 transition text-left"
              >
                <div className="flex items-center gap-2">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{
                      backgroundColor: getCommunityColor(comm.id),
                    }}
                  />
                  <span className="text-gray-300">{comm.label}</span>
                </div>
                <span className="text-gray-500">{comm.size} nodos</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function getCommunityColor(id: number): string {
  const colors = [
    '#E91E63', '#9C27B0', '#3F51B5', '#03A9F4', '#009688',
    '#8BC34A', '#CDDC39', '#FFC107', '#FF5722', '#795548',
  ];
  return colors[id % colors.length];
}
