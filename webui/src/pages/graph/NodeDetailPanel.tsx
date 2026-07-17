import { useState, useEffect, useCallback } from 'react';
import type { ApiNode, PathResponse, CentralityResponse } from './graphService';
import { fetchShortestPath, fetchCentrality } from './graphService';

interface NodeDetailPanelProps {
  node: ApiNode | null;
  allNodes: ApiNode[];
  onClose: () => void;
  onFindPath: (path: string[]) => void;
  onHighlightNode: (nodeId: string) => void;
}

export function NodeDetailPanel({
  node,
  allNodes,
  onClose,
  onFindPath,
  onHighlightNode,
}: NodeDetailPanelProps) {
  const [targetId, setTargetId] = useState('');
  const [pathResult, setPathResult] = useState<PathResponse | null>(null);
  const [nodeCentrality, setNodeCentrality] = useState<CentralityResponse | null>(null);
  const [loadingPath, setLoadingPath] = useState(false);

  const loadCentrality = useCallback(async () => {
    if (!node) return;
    try {
      const pr = await fetchCentrality({ metric: 'pagerank', topK: 100 });
      setNodeCentrality(pr);
    } catch {
      // ignore
    }
  }, [node]);

  useEffect(() => {
    if (node) {
      loadCentrality();
      setPathResult(null);
      setTargetId('');
    }
  }, [node, loadCentrality]);

  const handleFindPath = async () => {
    if (!node || !targetId) return;
    const sourceNum = parseInt(node.id.replace('doc:', ''));
    const targetNum = parseInt(targetId.replace('doc:', ''));
    if (isNaN(sourceNum) || isNaN(targetNum)) return;

    setLoadingPath(true);
    try {
      const result = await fetchShortestPath({
        sourceId: sourceNum,
        targetId: targetNum,
      });
      setPathResult(result);
      if (result.found && result.path.length > 0) {
        onFindPath(result.path);
      }
    } catch {
      setPathResult(null);
    } finally {
      setLoadingPath(false);
    }
  };

  if (!node) return null;

  const sourceNum = parseInt(node.id.replace('doc:', ''));
  const prValue = nodeCentrality?.values.find((v) => v.node_id === node.id);

  return (
    <div className="w-80 bg-dark-800 border-l border-gray-700 flex flex-col overflow-y-auto">
      {/* Header */}
      <div className="p-4 border-b border-gray-700 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-white truncate flex-1">{node.label}</h3>
        <button
          onClick={onClose}
          className="ml-2 w-6 h-6 flex items-center justify-center rounded hover:bg-dark-600 text-gray-400 transition"
        >
          x
        </button>
      </div>

      <div className="p-4 space-y-4 text-sm">
        {/* Properties */}
        <div className="space-y-2">
          <h4 className="text-xs font-semibold text-gray-400 uppercase">Propiedades</h4>
          <div className="space-y-1">
            <div className="flex justify-between">
              <span className="text-gray-400">ID</span>
              <span className="text-white font-mono text-xs">{node.id}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Tipo</span>
              <span className="text-white">{node.docType}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Ruta</span>
              <span className="text-white text-xs truncate max-w-[200px]">{node.path}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Grado</span>
              <span className="text-white font-mono">{node.degree}</span>
            </div>
            {node.community !== undefined && (
              <div className="flex justify-between">
                <span className="text-gray-400">Comunidad</span>
                <span className="text-white">#{node.community}</span>
              </div>
            )}
          </div>
        </div>

        {/* Metrics */}
        <div className="space-y-2">
          <h4 className="text-xs font-semibold text-gray-400 uppercase">Metricas</h4>
          <div className="space-y-1">
            {node.pagerank !== undefined && (
              <div className="flex justify-between">
                <span className="text-gray-400">PageRank</span>
                <span className="text-primary-400 font-mono">{node.pagerank.toFixed(6)}</span>
              </div>
            )}
            {node.betweenness !== undefined && (
              <div className="flex justify-between">
                <span className="text-gray-400">Betweenness</span>
                <span className="text-primary-400 font-mono">{node.betweenness.toFixed(6)}</span>
              </div>
            )}
            {prValue && !node.pagerank && (
              <div className="flex justify-between">
                <span className="text-gray-400">PageRank (global)</span>
                <span className="text-primary-400 font-mono">{prValue.score.toFixed(6)}</span>
              </div>
            )}
          </div>
        </div>

        {/* Shortest path finder */}
        <div className="space-y-2">
          <h4 className="text-xs font-semibold text-gray-400 uppercase">Encontrar camino</h4>
          <div className="flex gap-1">
            <input
              type="text"
              value={targetId}
              onChange={(e) => setTargetId(e.target.value)}
              placeholder="Target node ID (e.g. doc:5)"
              className="flex-1 bg-dark-700 border border-gray-600 rounded px-2 py-1 text-xs text-white placeholder-gray-500 focus:outline-none focus:border-primary-400"
            />
            <button
              onClick={handleFindPath}
              disabled={loadingPath || !targetId}
              className="px-2 py-1 bg-primary-600 hover:bg-primary-500 rounded text-xs text-white transition disabled:opacity-50"
            >
              {loadingPath ? '...' : 'Ir'}
            </button>
          </div>
          {pathResult && (
            <div className={`px-2 py-1 rounded text-xs ${pathResult.found ? 'bg-green-900/30 text-green-400' : 'bg-red-900/30 text-red-400'}`}>
              {pathResult.found
                ? `Camino: ${pathResult.path_length} pasos, distancia ${pathResult.distance.toFixed(2)}`
                : 'No se encontro camino'}
            </div>
          )}
        </div>

        {/* Related nodes */}
        <div className="space-y-2">
          <h4 className="text-xs font-semibold text-gray-400 uppercase">Nodos relacionados</h4>
          <div className="space-y-1 max-h-40 overflow-y-auto">
            {allNodes
              .filter((n) => n.id !== node.id)
              .slice(0, 10)
              .map((n) => (
                <button
                  key={n.id}
                  onClick={() => onHighlightNode(n.id)}
                  className="w-full flex items-center gap-2 px-2 py-1 text-xs rounded hover:bg-dark-600 transition text-left"
                >
                  <div
                    className="w-2 h-2 rounded-full flex-shrink-0"
                    style={{ backgroundColor: n.color || '#757575' }}
                  />
                  <span className="text-gray-300 truncate">{n.label}</span>
                </button>
              ))}
          </div>
        </div>
      </div>
    </div>
  );
}
