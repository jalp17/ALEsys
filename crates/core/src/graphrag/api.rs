//! Tipos de serialización para la API del grafo
//!
//! Define los formatos JSON para:
//! - GET /api/v1/graph (nodos + aristas)
//! - GET /api/v1/graph/centrality
//! - GET /api/v1/graph/communities
//! - GET /api/v1/graph/path
//! - POST /api/v1/graph/export

use serde::{Deserialize, Serialize};

// =============================================================================
// API Response Types
// =============================================================================

/// Nodo en formato API (para Cytoscape.js)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiNode {
    /// ID único: "doc:{id}"
    pub id: String,
    /// Label para mostrar
    pub label: String,
    /// Tipo de documento
    #[serde(rename = "docType")]
    pub doc_type: String,
    /// Ruta relativa
    pub path: String,
    /// Grado total
    pub degree: usize,
    /// Score de PageRank (opcional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagerank: Option<f64>,
    /// Score de Betweenness (opcional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub betweenness: Option<f64>,
    /// ID de comunidad (opcional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub community: Option<usize>,
    /// Color según tipo (para CSS de Cytoscape)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Arista en formato API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEdge {
    /// ID único: "e:{origen_id}-{destino_id}-{tipo}"
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Tipo de enlace
    #[serde(rename = "edgeType")]
    pub edge_type: String,
    /// Contexto del enlace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Peso (1.0 wiki_link, 1.5 backlink, 2.0 reference)
    pub weight: f64,
    /// Color según tipo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Respuesta de GET /api/v1/graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse {
    pub nodes: Vec<ApiNode>,
    pub edges: Vec<ApiEdge>,
    pub stats: GraphStats,
    pub pagination: Option<PaginationInfo>,
}

/// Estadísticas del grafo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub density: f64,
    pub avg_degree: f64,
    pub num_communities: usize,
}

/// Info de paginación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub cursor: Option<String>,
    pub has_more: bool,
    pub returned_nodes: usize,
    pub total_available: usize,
}

/// Respuesta de centrality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityResponse {
    pub metric: String,
    pub values: Vec<CentralityValue>,
    pub top_nodes: Vec<String>,
    pub threshold: Option<f64>,
}

/// Valor de centralidad para un nodo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityValue {
    pub node_id: String,
    pub score: f64,
}

/// Respuesta de comunidades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunitiesResponse {
    pub communities: Vec<CommunityInfo>,
    pub algorithm: String,
    pub iterations: usize,
}

/// Info de comunidad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityInfo {
    pub id: usize,
    pub size: usize,
    pub members: Vec<String>,
    pub avg_pagerank: f64,
    pub label: String,
}

/// Respuesta de shortest path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResponse {
    pub source: String,
    pub target: String,
    pub path: Vec<String>,
    pub distance: f64,
    pub found: bool,
    pub path_length: usize,
}

// =============================================================================
// Request Types
// =============================================================================

/// Query para GET /api/v1/graph
#[derive(Debug, Clone, Deserialize)]
pub struct GraphQuery {
    /// Filtro por tipo de documento
    #[serde(rename = "docType")]
    pub doc_type: Option<String>,
    /// Filtro por tipo de enlace (wiki_link, backlink, reference)
    #[serde(rename = "edgeType")]
    pub edge_type: Option<String>,
    /// Profundidad máxima (default 2)
    pub depth: Option<usize>,
    /// Límite de nodos (default 500)
    pub limit: Option<usize>,
    /// Cursor para paginación
    pub cursor: Option<String>,
    /// Centro del subgrafo (node_id)
    #[serde(rename = "centerNodeId")]
    pub center_node_id: Option<i32>,
    /// Incluir métricas de centralidad
    #[serde(rename = "includeMetrics")]
    pub include_metrics: Option<bool>,
}

/// Query para GET /api/v1/graph/centrality
#[derive(Debug, Clone, Deserialize)]
pub struct CentralityQuery {
    /// Métrica: pagerank, betweenness, degree
    pub metric: Option<String>,
    /// Top K nodos (default 10)
    #[serde(rename = "topK")]
    pub top_k: Option<usize>,
    /// Threshold mínimo
    pub threshold: Option<f64>,
}

/// Query para GET /api/v1/graph/communities
#[derive(Debug, Clone, Deserialize)]
pub struct CommunitiesQuery {
    /// Máximo de iteraciones (default 10)
    pub max_iterations: Option<usize>,
}

/// Query para GET /api/v1/graph/path
#[derive(Debug, Clone, Deserialize)]
pub struct PathQuery {
    /// ID del nodo origen
    #[serde(rename = "sourceId")]
    pub source_id: i32,
    /// ID del nodo destino
    #[serde(rename = "targetId")]
    pub target_id: i32,
}

/// Query para POST /api/v1/graph/export
#[derive(Debug, Clone, Deserialize)]
pub struct ExportQuery {
    /// Formato: json, graphml, png, svg
    pub format: String,
    /// IDs de nodos a exportar (opcional, default: todos)
    #[serde(rename = "nodeIds")]
    pub node_ids: Option<Vec<String>>,
    /// Incluir métricas
    #[serde(rename = "includeMetrics")]
    pub include_metrics: Option<bool>,
}

// =============================================================================
// Helpers
// =============================================================================

impl ApiNode {
    /// Crear desde DocumentNode + metadatos
    pub fn from_document(
        id: i32,
        path: &str,
        doc_type: &str,
        degree: usize,
        pagerank: Option<f64>,
        betweenness: Option<f64>,
        community: Option<usize>,
    ) -> Self {
        let color = node_color(doc_type, community);
        Self {
            id: format!("doc:{}", id),
            label: std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(path)
                .to_string(),
            doc_type: doc_type.to_string(),
            path: path.to_string(),
            degree,
            pagerank,
            betweenness,
            community,
            color: Some(color),
        }
    }
}

impl ApiEdge {
    /// Crear desde EdgeType + IDs
    pub fn from_edge(
        origen_id: i32,
        destino_id: i32,
        edge_type: &super::EdgeType,
    ) -> Self {
        let (tipo_str, context, weight, color) = match edge_type {
            super::EdgeType::WikiLink { context } => (
                "wiki_link".to_string(),
                Some(context.clone()),
                1.0,
                "#4CAF50".to_string(), // green
            ),
            super::EdgeType::Backlink { context } => (
                "backlink".to_string(),
                Some(context.clone()),
                1.5,
                "#2196F3".to_string(), // blue
            ),
            super::EdgeType::Reference { context } => (
                "reference".to_string(),
                Some(context.clone()),
                2.0,
                "#FF9800".to_string(), // orange
            ),
        };

        Self {
            id: format!("e:{}-{}-{}", origen_id, destino_id, tipo_str),
            source: format!("doc:{}", origen_id),
            target: format!("doc:{}", destino_id),
            edge_type: tipo_str,
            context,
            weight,
            color: Some(color),
        }
    }
}

/// Color de nodo según tipo y comunidad
fn node_color(doc_type: &str, community: Option<usize>) -> String {
    // Si tiene comunidad, usar color de comunidad
    if let Some(comm_id) = community {
        let colors = [
            "#E91E63", "#9C27B0", "#3F51B5", "#03A9F4", "#009688",
            "#8BC34A", "#CDDC39", "#FFC107", "#FF5722", "#795548",
        ];
        return colors[comm_id % colors.len()].to_string();
    }

    // Fallback por doc_type
    match doc_type {
        "markdown" | "md" => "#607D8B".to_string(),
        "code" | "rust" | "python" | "javascript" | "typescript" => "#9C27B0".to_string(),
        "pdf" => "#F44336".to_string(),
        "image" => "#FF9800".to_string(),
        _ => "#757575".to_string(),
    }
}

impl GraphStats {
    /// Calcular densidad del grafo
    pub fn calculate(
        total_nodes: usize,
        total_edges: usize,
        num_communities: usize,
    ) -> Self {
        let density = if total_nodes > 1 {
            2.0 * total_edges as f64 / (total_nodes as f64 * (total_nodes as f64 - 1.0))
        } else {
            0.0
        };

        let avg_degree = if total_nodes > 0 {
            2.0 * total_edges as f64 / total_nodes as f64
        } else {
            0.0
        };

        Self {
            total_nodes,
            total_edges,
            density,
            avg_degree,
            num_communities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_node_serialization() {
        let node = ApiNode::from_document(1, "docs/readme.md", "markdown", 5, None, None, Some(0));
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"id\":\"doc:1\""));
        assert!(json.contains("\"label\":\"readme\""));
        assert!(json.contains("\"docType\":\"markdown\""));
    }

    #[test]
    fn test_api_edge_serialization() {
        let edge = ApiEdge::from_edge(
            1,
            2,
            &super::super::EdgeType::WikiLink { context: "test".to_string() },
        );
        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("\"source\":\"doc:1\""));
        assert!(json.contains("\"target\":\"doc:2\""));
        assert!(json.contains("\"edgeType\":\"wiki_link\""));
    }

    #[test]
    fn test_graph_stats_calculation() {
        let stats = GraphStats::calculate(100, 250, 5);
        assert_eq!(stats.total_nodes, 100);
        assert_eq!(stats.total_edges, 250);
        assert!(stats.density > 0.0 && stats.density < 1.0);
        assert_eq!(stats.avg_degree, 5.0);
    }

    #[test]
    fn test_node_color_by_type() {
        let color_md = node_color("markdown", None);
        let color_code = node_color("code", None);
        assert_ne!(color_md, color_code);
    }

    #[test]
    fn test_node_color_by_community() {
        let color_comm0 = node_color("markdown", Some(0));
        let color_comm1 = node_color("markdown", Some(1));
        assert_ne!(color_comm0, color_comm1);
    }

    #[test]
    fn test_graph_query_defaults() {
        let query = GraphQuery {
            doc_type: None,
            edge_type: None,
            depth: None,
            limit: None,
            cursor: None,
            center_node_id: None,
            include_metrics: None,
        };
        assert!(query.doc_type.is_none());
        assert!(query.depth.is_none());
    }
}
