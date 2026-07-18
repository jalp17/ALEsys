import { useEffect, useRef, useCallback } from 'react';
import cytoscape, { Core, EventObject } from 'cytoscape';
import coseBilkent from 'cytoscape-cose-bilkent';
import type { ApiNode, ApiEdge } from './graphService';

cytoscape.use(coseBilkent);

export type LayoutName =
  | 'cose-bilkent'
  | 'circle'
  | 'concentric'
  | 'breadthfirst'
  | 'grid'
  | 'random'
  | 'spread';

export interface GraphElement {
  data: {
    id: string;
    label?: string;
    source?: string;
    target?: string;
    color?: string;
    docType?: string;
    edgeType?: string;
    weight?: number;
    degree?: number;
    pagerank?: number;
    betweenness?: number;
    community?: number;
  };
  classes?: string;
}

interface GraphCanvasProps {
  nodes: ApiNode[];
  edges: ApiEdge[];
  layout: LayoutName;
  onNodeClick?: (nodeId: string) => void;
  highlightPath?: string[];
  selectedMetric?: 'pagerank' | 'betweenness' | 'degree' | null;
}

const LAYOUT_OPTIONS: Record<LayoutName, object> = {
  'cose-bilkent': {
    name: 'cose-bilkent',
    quality: 'default',
    animate: 'end',
    animationDuration: 500,
    randomize: true,
    nodeRepulsion: 4500,
    idealEdgeLength: 50,
    edgeElasticity: 0.45,
    nestingFactor: 0.1,
    gravity: 0.25,
    numIter: 2500,
    tile: true,
    NodeRepulsion: 4500,
    IdealEdgeLength: 50,
    EdgeElasticity: 0.45,
    Gravity: 0.25,
  },
  circle: { name: 'circle', animate: 'end', animationDuration: 400 },
  concentric: {
    name: 'concentric',
    concentric: (node: cytoscape.NodeSingular) => {
      return node.data('pagerank') ?? node.data('degree') ?? 1;
    },
    levelWidth: () => 2,
    animate: 'end',
    animationDuration: 400,
  },
  breadthfirst: {
    name: 'breadthfirst',
    directed: true,
    animate: 'end',
    animationDuration: 400,
  },
  grid: { name: 'grid', animate: 'end', animationDuration: 400 },
  random: { name: 'random', animate: 'end', animationDuration: 400 },
  spread: { name: 'spread', animate: 'end', animationDuration: 400 },
};

function buildElements(nodes: ApiNode[], edges: ApiEdge[]): GraphElement[] {
  const nodeElements: GraphElement[] = nodes.map((n) => ({
    data: {
      id: n.id,
      label: n.label,
      color: n.color || '#757575',
      docType: n.docType,
      degree: n.degree,
      pagerank: n.pagerank,
      betweenness: n.betweenness,
      community: n.community,
    },
    classes: `node-${n.docType}`,
  }));

  const edgeElements: GraphElement[] = edges.map((e) => ({
    data: {
      id: e.id,
      source: e.source,
      target: e.target,
      edgeType: e.edgeType,
      color: e.color || '#555',
      weight: e.weight,
    },
    classes: `edge-${e.edgeType}`,
  }));

  return [...nodeElements, ...edgeElements];
}

// TODO: Implementar metric-based node sizing
// function getMetricSize(node: cytoscape.NodeSingular, metric: string): number {
//   switch (metric) {
//     case 'pagerank':
//       return Math.max(8, (node.data('pagerank') ?? 0.01) * 200);
//     case 'betweenness':
//       return Math.max(8, (node.data('betweenness') ?? 0) * 300);
//     case 'degree':
//       return Math.max(8, (node.data('degree') ?? 1) * 4);
//     default:
//       return 20;
//   }
// }

export function GraphCanvas({
  nodes,
  edges,
  layout,
  onNodeClick,
  highlightPath,
  selectedMetric,
}: GraphCanvasProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<Core | null>(null);

  const initCy = useCallback(() => {
    if (!containerRef.current) return;

    if (cyRef.current) {
      cyRef.current.destroy();
    }

    const elements = buildElements(nodes, edges);

    const cy = cytoscape({
      container: containerRef.current,
      elements,
      style: [
        {
          selector: 'node',
          style: {
            label: 'data(label)',
            'background-color': 'data(color)',
            color: '#fff',
            'text-valign': 'center',
            'text-halign': 'center',
            'font-size': '10px',
            width: 20,
            height: 20,
            'border-width': 2,
            'border-color': '#333',
          },
        },
        {
          selector: 'node.pagerank-active',
          style: {
            width: 'mapData(pagerank, 0, 0.1, 10, 60)',
            height: 'mapData(pagerank, 0, 0.1, 10, 60)',
          },
        },
        {
          selector: 'node.betweenness-active',
          style: {
            width: 'mapData(betweenness, 0, 0.5, 10, 60)',
            height: 'mapData(betweenness, 0, 0.5, 10, 60)',
          },
        },
        {
          selector: 'node.degree-active',
          style: {
            width: 'mapData(degree, 0, 10, 10, 60)',
            height: 'mapData(degree, 0, 10, 10, 60)',
          },
        },
        {
          selector: 'node.highlighted',
          style: {
            'border-width': 4,
            'border-color': '#FFD700',
            'z-index': 10,
          },
        },
        {
          selector: 'node.dimmed',
          style: {
            opacity: 0.2,
          },
        },
        {
          selector: 'edge',
          style: {
            width: 1.5,
            'line-color': 'data(color)',
            'target-arrow-color': 'data(color)',
            'target-arrow-shape': 'triangle',
            'curve-style': 'bezier',
            opacity: 0.6,
          },
        },
        {
          selector: 'edge.highlighted',
          style: {
            width: 3,
            opacity: 1,
            'line-color': '#FFD700',
            'z-index': 10,
          },
        },
        {
          selector: 'edge.dimmed',
          style: {
            opacity: 0.1,
          },
        },
      ],
      layout: { name: 'preset' },
      minZoom: 0.1,
      maxZoom: 5,
      wheelSensitivity: 0.2,
    });

    cy.on('tap', 'node', (evt: EventObject) => {
      if (onNodeClick) {
        onNodeClick(evt.target.id());
      }
    });

    // Apply layout
    cy.layout(LAYOUT_OPTIONS[layout] as any).run();

    cyRef.current = cy;
  }, [nodes, edges, layout]);

  // Initialize / re-init on data change
  useEffect(() => {
    initCy();
    return () => {
      cyRef.current?.destroy();
      cyRef.current = null;
    };
  }, [initCy]);

  // Handle metric visualization
  useEffect(() => {
    const cy = cyRef.current;
    if (!cy) return;

    cy.nodes().removeClass('pagerank-active betweenness-active degree-active highlighted dimmed');
    cy.edges().removeClass('highlighted dimmed');

    if (selectedMetric) {
      cy.nodes().addClass(`${selectedMetric}-active`);
    }
  }, [selectedMetric]);

  // Handle highlight path
  useEffect(() => {
    const cy = cyRef.current;
    if (!cy || !highlightPath || highlightPath.length === 0) {
      cy?.nodes().removeClass('highlighted dimmed');
      cy?.edges().removeClass('highlighted dimmed');
      return;
    }

    const pathSet = new Set(highlightPath);
    cy.nodes().forEach((node) => {
      if (pathSet.has(node.id())) {
        node.addClass('highlighted').removeClass('dimmed');
      } else {
        node.addClass('dimmed').removeClass('highlighted');
      }
    });

    cy.edges().forEach((edge) => {
      const src = edge.source().id();
      const tgt = edge.target().id();
      if (pathSet.has(src) && pathSet.has(tgt)) {
        const idx1 = highlightPath.indexOf(src);
        const idx2 = highlightPath.indexOf(tgt);
        if (Math.abs(idx1 - idx2) === 1) {
          edge.addClass('highlighted').removeClass('dimmed');
        } else {
          edge.addClass('dimmed').removeClass('highlighted');
        }
      } else {
        edge.addClass('dimmed').removeClass('highlighted');
      }
    });
  }, [highlightPath]);

  // Expose cy for toolbar actions
  useEffect(() => {
    (window as any).__graphCy = cyRef.current;
  });

  return (
    <div
      ref={containerRef}
      className="w-full h-full bg-dark-950 rounded-lg border border-gray-700"
      style={{ minHeight: 400 }}
    />
  );
}
