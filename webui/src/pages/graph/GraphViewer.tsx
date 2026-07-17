import { useState, useCallback, useEffect } from 'react';
import { GraphCanvas, type LayoutName } from './GraphCanvas';
import { GraphSidebar } from './GraphSidebar';
import { GraphToolbar } from './GraphToolbar';
import {
  fetchGraph,
  fetchCentrality,
  fetchCommunities,
  searchGraph,
  type ApiNode,
  type ApiEdge,
  type GraphStats,
  type CentralityResponse,
  type CommunitiesResponse,
} from './graphService';

export function GraphViewer() {
  const [nodes, setNodes] = useState<ApiNode[]>([]);
  const [edges, setEdges] = useState<ApiEdge[]>([]);
  const [stats, setStats] = useState<GraphStats | null>(null);
  const [layout, setLayout] = useState<LayoutName>('cose-bilkent');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedMetric, setSelectedMetric] = useState<
    'pagerank' | 'betweenness' | 'degree' | null
  >(null);
  const [centrality, setCentrality] = useState<CentralityResponse | null>(null);
  const [communities, setCommunities] = useState<CommunitiesResponse | null>(null);
  const [highlightPath, setHighlightPath] = useState<string[]>([]);
  const [docTypeFilter, setDocTypeFilter] = useState('');
  const [edgeTypeFilter, setEdgeTypeFilter] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<ApiNode[]>([]);

  const loadGraph = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchGraph({
        docType: docTypeFilter || undefined,
        edgeType: edgeTypeFilter || undefined,
        limit: 500,
        includeMetrics: true,
      });
      setNodes(data.nodes);
      setEdges(data.edges);
      setStats(data.stats);
    } catch (e: any) {
      setError(e.message || 'Error cargando grafo');
    } finally {
      setLoading(false);
    }
  }, [docTypeFilter, edgeTypeFilter]);

  const loadCentrality = useCallback(async (metric: string) => {
    try {
      const data = await fetchCentrality({ metric, topK: 20 });
      setCentrality(data);
    } catch (e) {
      console.error('Error loading centrality:', e);
    }
  }, []);

  const loadCommunities = useCallback(async () => {
    try {
      const data = await fetchCommunities({ maxIterations: 15 });
      setCommunities(data);
    } catch (e) {
      console.error('Error loading communities:', e);
    }
  }, []);

  useEffect(() => {
    loadGraph();
    loadCommunities();
  }, [loadGraph, loadCommunities]);

  useEffect(() => {
    if (selectedMetric) {
      loadCentrality(selectedMetric);
    }
  }, [selectedMetric, loadCentrality]);

  const handleMetricChange = (metric: 'pagerank' | 'betweenness' | 'degree' | null) => {
    setSelectedMetric(metric);
    if (metric) {
      loadCentrality(metric);
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      setSearchResults([]);
      return;
    }
    try {
      const data = await searchGraph(searchQuery, 20);
      setSearchResults(data.nodes);
    } catch {
      setSearchResults([]);
    }
  };

  const handleNodeClick = (nodeId: string) => {
    console.log('Node clicked:', nodeId);
    // Future: show node detail panel
  };

  const handleCommunityClick = (communityId: number) => {
    if (!communities) return;
    const comm = communities.communities[communityId];
    if (comm) {
      setHighlightPath(comm.members);
      setTimeout(() => setHighlightPath([]), 5000);
    }
  };

  const handleNodeFocus = (nodeId: string) => {
    const node = nodes.find((n) => n.id === nodeId);
    if (node) {
      setHighlightPath([nodeId]);
      setTimeout(() => setHighlightPath([]), 3000);
    }
  };

  const getCy = () => (window as any).__graphCy;

  const handleZoomIn = () => getCy()?.zoom({ level: getCy().zoom() * 1.3, renderedPosition: { x: 0, y: 0 } });
  const handleZoomOut = () => getCy()?.zoom({ level: getCy().zoom() / 1.3, renderedPosition: { x: 0, y: 0 } });
  const handleFit = () => getCy()?.fit(undefined, 50);

  const handleExportPNG = () => {
    const cy = getCy();
    if (!cy) return;
    const png = cy.png({ bg: '#1a1a2e', full: true, scale: 2 });
    const link = document.createElement('a');
    link.href = png;
    link.download = 'alesys-graph.png';
    link.click();
  };

  const handleExportSVG = () => {
    const cy = getCy();
    if (!cy) return;
    const svg = cy.svg({ bg: '#1a1a2e', full: true, scale: 2 });
    const blob = new Blob([svg], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'alesys-graph.svg';
    link.click();
    URL.revokeObjectURL(url);
  };

  const handleExportJSON = () => {
    const data = { nodes, edges, stats, exported_at: new Date().toISOString() };
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'alesys-graph.json';
    link.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="flex h-[calc(100vh-6rem)]">
      <GraphSidebar
        stats={stats}
        centrality={centrality}
        communities={communities}
        selectedMetric={selectedMetric}
        onMetricChange={handleMetricChange}
        onCommunityClick={handleCommunityClick}
        onNodeFocus={handleNodeFocus}
      />

      <div className="flex-1 flex flex-col">
        <GraphToolbar
          layout={layout}
          onLayoutChange={setLayout}
          onZoomIn={handleZoomIn}
          onZoomOut={handleZoomOut}
          onFit={handleFit}
          onReload={loadGraph}
          loading={loading}
          docTypeFilter={docTypeFilter}
          onDocTypeFilterChange={setDocTypeFilter}
          edgeTypeFilter={edgeTypeFilter}
          onEdgeTypeFilterChange={setEdgeTypeFilter}
          onExportPNG={handleExportPNG}
          onExportSVG={handleExportSVG}
          onExportJSON={handleExportJSON}
        />

        {/* Search bar */}
        <div className="px-4 py-2 bg-dark-800 border-b border-gray-700 flex items-center gap-2">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Buscar documentos en el grafo..."
            className="flex-1 bg-dark-700 border border-gray-600 rounded px-3 py-1.5 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-primary-400"
          />
          <button
            onClick={handleSearch}
            className="px-3 py-1.5 bg-primary-600 hover:bg-primary-500 rounded text-sm text-white transition"
          >
            Buscar
          </button>
          {searchResults.length > 0 && (
            <div className="absolute top-full left-0 right-0 bg-dark-700 border border-gray-600 rounded shadow-lg z-50 max-h-60 overflow-y-auto mt-1 mx-4">
              {searchResults.map((node) => (
                <button
                  key={node.id}
                  onClick={() => {
                    handleNodeFocus(node.id);
                    setSearchResults([]);
                    setSearchQuery(node.label);
                  }}
                  className="w-full px-3 py-2 text-xs text-left text-gray-300 hover:bg-dark-600 transition flex items-center gap-2"
                >
                  <div
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: node.color || '#757575' }}
                  />
                  <span>{node.label}</span>
                  <span className="text-gray-500 ml-auto">{node.docType}</span>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Graph canvas */}
        <div className="flex-1 relative">
          {error && (
            <div className="absolute inset-0 flex items-center justify-center bg-dark-950 z-10">
              <div className="text-center">
                <p className="text-red-400 mb-4">{error}</p>
                <button
                  onClick={loadGraph}
                  className="px-4 py-2 bg-primary-600 hover:bg-primary-500 rounded text-sm text-white transition"
                >
                  Reintentar
                </button>
              </div>
            </div>
          )}
          {!loading && !error && nodes.length === 0 && (
            <div className="absolute inset-0 flex items-center justify-center bg-dark-950 z-10">
              <div className="text-center text-gray-400">
                <p className="text-lg mb-2">No hay documentos en el grafo</p>
                <p className="text-sm">Indexa documentos para visualizar el grafo de conocimiento</p>
              </div>
            </div>
          )}
          <GraphCanvas
            nodes={nodes}
            edges={edges}
            layout={layout}
            onNodeClick={handleNodeClick}
            highlightPath={highlightPath}
            selectedMetric={selectedMetric}
          />
        </div>
      </div>
    </div>
  );
}
