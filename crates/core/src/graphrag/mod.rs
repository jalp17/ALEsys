//! GraphRAG - Graph + Retrieval Augmented Generation
//!
//! Combina búsqueda vectorial (pgvector) con traversales de grafo (petgraph)

pub mod algorithms;
pub mod api;
pub mod query;

use crate::Result;
use moka::sync::Cache;
use petgraph::graph::{DiGraph, NodeIndex};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// Manager para GraphRAG
pub struct GraphRAG {
    db: PgPool,
    graph: DiGraph<DocumentNode, EdgeType>,
    node_map: HashMap<i32, NodeIndex>,
    /// Cache de resultados de búsqueda (embedding_hash -> results)
    search_cache: Cache<u64, Vec<SearchResult>>,
}

#[derive(Debug, Clone)]
pub struct DocumentNode {
    pub id: i32,
    pub path: String,
    pub doc_type: String,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    WikiLink { context: String },
    Backlink { context: String },
    Reference { context: String },
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub fragment_id: i32,
    pub document_id: i32,
    pub content: String,
    pub similarity: f32,
    pub source: SearchResultSource,
    pub doc_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchResultSource {
    Vector,
    Graph,
}

impl GraphRAG {
    pub async fn new(db: PgPool) -> Result<Self> {
        tracing::info!("Inicializando GraphRAG...");
        let (graph, node_map) = Self::load_graph_from_db(&db).await?;
        tracing::info!(
            "GraphRAG inicializado: {} nodos, {} edges",
            graph.node_count(),
            graph.edge_count()
        );
        Ok(Self {
            db,
            graph,
            node_map,
            search_cache: Cache::new(1000),
        })
    }

    pub async fn reload_graph(&mut self) -> Result<()> {
        tracing::info!("Recargando grafo desde DB...");
        let (graph, node_map) = Self::load_graph_from_db(&self.db).await?;
        tracing::info!(
            "Grafo recargado: {} nodos, {} edges",
            graph.node_count(),
            graph.edge_count()
        );
        self.graph = graph;
        self.node_map = node_map;
        Ok(())
    }

    async fn load_graph_from_db(
        db: &PgPool,
    ) -> Result<(DiGraph<DocumentNode, EdgeType>, HashMap<i32, NodeIndex>)> {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        // Paginated load using cursor-based pagination (OFFSET-free)
        let batch_size: i64 = 500;
        let mut last_id: i32 = 0;
        let mut total_docs = 0usize;

        loop {
            let rows = sqlx::query(
                "SELECT id, ruta_relativa, tipo FROM documentos WHERE id > $1 ORDER BY id LIMIT $2",
            )
            .bind(last_id)
            .bind(batch_size)
            .fetch_all(db)
            .await
            .map_err(|e| {
                tracing::error!("DB error cargando documentos (batch desde id={}): {}", last_id, e);
                crate::AlesysError::Database(e)
            })?;

            if rows.is_empty() {
                break;
            }

            total_docs += rows.len();
            for row in &rows {
                let id: i32 = row.get("id");
                let ruta_relativa: String = row.get("ruta_relativa");
                let tipo: String = row.get("tipo");
                let idx = graph.add_node(DocumentNode {
                    id,
                    path: ruta_relativa,
                    doc_type: tipo,
                });
                node_map.insert(id, idx);
            }

            if let Some(last) = rows.last() {
                last_id = last.get("id");
            } else {
                break;
            }
        }

        // Paginated edge load
        let mut last_origen: i32 = 0;
        let mut total_edges = 0usize;

        loop {
            let rows = sqlx::query(
                "SELECT origen_id, destino_id, tipo_enlace, contexto FROM enlaces WHERE origen_id > $1 ORDER BY origen_id LIMIT $2",
            )
            .bind(last_origen)
            .bind(batch_size)
            .fetch_all(db)
            .await
            .map_err(|e| {
                tracing::error!("DB error cargando enlaces (batch desde id={}): {}", last_origen, e);
                crate::AlesysError::Database(e)
            })?;

            if rows.is_empty() {
                break;
            }

            total_edges += rows.len();
            for row in &rows {
                let origen_id: i32 = row.get("origen_id");
                let destino_id: i32 = row.get("destino_id");
                let tipo_enlace: Option<String> = row.get("tipo_enlace");
                let contexto: Option<String> = row.get("contexto");

                if let (Some(&src), Some(&dst)) =
                    (node_map.get(&origen_id), node_map.get(&destino_id))
                {
                    let ctx = contexto.unwrap_or_default();
                    let edge_type = match tipo_enlace.as_deref() {
                        Some("wiki_link") => EdgeType::WikiLink { context: ctx },
                        Some("backlink") => EdgeType::Backlink { context: ctx },
                        _ => EdgeType::Reference { context: ctx },
                    };
                    graph.add_edge(src, dst, edge_type);
                }
            }

            if let Some(last) = rows.last() {
                last_origen = last.get("origen_id");
            } else {
                break;
            }
        }

        tracing::debug!(
            "Grafo cargado: {} documentos, {} enlaces",
            total_docs,
            total_edges
        );
        Ok((graph, node_map))
    }

    pub async fn vector_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Check cache first
        let cache_key = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            for v in query_embedding.iter().step_by(4) {
                v.to_bits().hash(&mut hasher);
            }
            limit.hash(&mut hasher);
            hasher.finish()
        };

        if let Some(cached) = self.search_cache.get(&cache_key) {
            tracing::debug!("vector_search: cache hit for key {}", cache_key);
            return Ok(cached);
        }

        let embedding_str = format!(
            "[{}]",
            query_embedding
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let rows = sqlx::query(
            r#"
            SELECT f.id, f.documento_id, f.contenido,
                   (f.embedding <=> $1::vector) AS distancia,
                   d.ruta_relativa
            FROM fragmentos f
            JOIN documentos d ON d.id = f.documento_id
            ORDER BY distancia
            LIMIT $2
            "#,
        )
        .bind(&embedding_str)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("DB error en vector_search: {}", e);
            crate::AlesysError::Database(e)
        })?;

        tracing::debug!("vector_search: {} resultados", rows.len());
        let results: Vec<SearchResult> = rows
            .into_iter()
            .map(|row| SearchResult {
                fragment_id: row.get("id"),
                document_id: row.get("documento_id"),
                content: row.get("contenido"),
                similarity: {
                    let dist: f64 = row.get("distancia");
                    1.0 - dist as f32
                },
                source: SearchResultSource::Vector,
                doc_path: row.get("ruta_relativa"),
            })
            .collect();

        // Cache the results
        self.search_cache.insert(cache_key, results.clone());

        Ok(results)
    }

    pub async fn hybrid_search(
        &self,
        query_embedding: &[f32],
        vector_limit: usize,
        graph_degrees: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut results = self.vector_search(query_embedding, vector_limit).await?;

        let doc_ids: Vec<i32> = results
            .iter()
            .map(|r| r.document_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let expanded = self.expand_with_graph(&doc_ids, graph_degrees);

        let new_doc_ids: Vec<i32> = expanded
            .iter()
            .filter(|id| !doc_ids.contains(id))
            .copied()
            .collect();

        if !new_doc_ids.is_empty() {
            let rows = sqlx::query(
                "SELECT id, documento_id, contenido FROM fragmentos WHERE documento_id = ANY($1) ORDER BY documento_id, indice_orden",
            )
            .bind(&new_doc_ids)
            .fetch_all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("DB error cargando fragmentos expandidos: {}", e);
                crate::AlesysError::Database(e)
            })?;

            tracing::debug!(
                "hybrid_search: {} docs vectoriales, {} expandidos, {} fragmentos graph",
                doc_ids.len(),
                new_doc_ids.len(),
                rows.len()
            );

            for row in rows {
                let frag_id: i32 = row.get("id");
                let doc_id: i32 = row.get("documento_id");
                let frag_content: String = row.get("contenido");
                results.push(SearchResult {
                    fragment_id: frag_id,
                    document_id: doc_id,
                    content: frag_content,
                    similarity: 0.3,
                    source: SearchResultSource::Graph,
                    doc_path: self
                        .node_map
                        .get(&doc_id)
                        .and_then(|&idx| self.graph.node_weight(idx))
                        .map(|n| n.path.clone()),
                });
            }
        }

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        tracing::debug!("hybrid_search total: {} resultados", results.len());
        Ok(results)
    }

    fn expand_with_graph(&self, doc_ids: &[i32], degrees: usize) -> Vec<i32> {
        let max_expanded = 50; // Limitar expansion para evitar O(N)
        let mut expanded = std::collections::HashSet::new();
        let mut queue: std::collections::VecDeque<(i32, usize)> =
            doc_ids.iter().map(|&id| (id, 0)).collect();

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= degrees || expanded.len() >= max_expanded {
                continue;
            }

            if let Some(&node_idx) = self.node_map.get(&current_id) {
                for neighbor in self.graph.neighbors_undirected(node_idx) {
                    if expanded.len() >= max_expanded {
                        break;
                    }
                    if let Some(neighbor_id) = self.graph.node_weight(neighbor).map(|n| n.id) {
                        if !expanded.contains(&neighbor_id) && !doc_ids.contains(&neighbor_id) {
                            expanded.insert(neighbor_id);
                            queue.push_back((neighbor_id, depth + 1));
                        }
                    }
                }
            }
        }

        expanded.into_iter().collect()
    }

    pub async fn search_by_path(&self, path_pattern: &str) -> Result<Vec<SearchResult>> {
        let escaped = path_pattern
            .replace('%', "\\%")
            .replace('_', "\\_");
        let rows = sqlx::query(
            r#"
            SELECT f.id, f.documento_id, f.contenido, d.ruta_relativa
            FROM fragmentos f
            JOIN documentos d ON d.id = f.documento_id
            WHERE d.ruta_relativa LIKE $1 ESCAPE '\'
            LIMIT 10
            "#,
        )
        .bind(format!("%{}%", escaped))
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("DB error en search_by_path: {}", e);
            crate::AlesysError::Database(e)
        })?;

        tracing::debug!("search_by_path '{}': {} resultados", path_pattern, rows.len());
        let results = rows
            .into_iter()
            .map(|row| SearchResult {
                fragment_id: row.get("id"),
                document_id: row.get("documento_id"),
                content: row.get("contenido"),
                similarity: 1.0,
                source: SearchResultSource::Vector,
                doc_path: row.get("ruta_relativa"),
            })
            .collect();

        Ok(results)
    }

    pub fn graph_stats(&self) -> GraphStats {
        GraphStats {
            nodes: self.graph.node_count(),
            edges: self.graph.edge_count(),
        }
    }

    // =========================================================================
    // Graph API Methods (Phase 5)
    // =========================================================================

    /// Obtener subgrafo como API response (con paginación, filtros, métricas)
    pub async fn get_graph_api(
        &self,
        query: &api::GraphQuery,
        user_id: i32,
    ) -> Result<api::GraphResponse> {
        let limit = query.limit.unwrap_or(500).min(1000);
        let cursor = query
            .cursor
            .as_ref()
            .and_then(|c| c.parse::<i32>().ok());

        // 1. Verificar permisos
        let accessible_ids = query::get_accessible_doc_ids(&self.db, user_id).await?;

        // 2. Cargar nodos (con filtro de tipo y paginación)
        let raw_nodes = if let Some(center_id) = query.center_node_id {
            let depth = query.depth.unwrap_or(2).min(5);
            query::load_neighbor_nodes(&self.db, center_id, depth).await?
        } else {
            query::load_nodes_paginated(&self.db, cursor, limit, query.doc_type.as_deref())
                .await?
                .nodes
        };

        // 3. Filtrar por permisos
        let filtered_nodes: Vec<query::RawNode> = raw_nodes
            .into_iter()
            .filter(|n| accessible_ids.is_empty() || accessible_ids.contains(&n.id))
            .collect();

        // 4. Cargar aristas entre esos nodos
        let node_ids: Vec<i32> = filtered_nodes.iter().map(|n| n.id).collect();
        let raw_edges =
            query::load_edges_for_nodes(&self.db, &node_ids, query.edge_type.as_deref()).await?;

        // 5. Construir API response
        let mut api_nodes: Vec<api::ApiNode> = filtered_nodes
            .iter()
            .map(|n| {
                let node_id = n.id;
                let path = &n.ruta_relativa;
                let degree = raw_edges
                    .iter()
                    .filter(|e| e.origen_id == node_id || e.destino_id == node_id)
                    .count();

                api::ApiNode::from_document(
                    node_id,
                    path,
                    &n.tipo,
                    degree,
                    None,
                    None,
                    None,
                )
            })
            .collect();

        let api_edges: Vec<api::ApiEdge> = raw_edges
            .iter()
            .filter_map(|e| {
                let edge_type = match e.tipo_enlace.as_deref() {
                    Some("wiki_link") => EdgeType::WikiLink {
                        context: e.contexto.clone().unwrap_or_default(),
                    },
                    Some("backlink") => EdgeType::Backlink {
                        context: e.contexto.clone().unwrap_or_default(),
                    },
                    Some("reference") => EdgeType::Reference {
                        context: e.contexto.clone().unwrap_or_default(),
                    },
                    _ => return None,
                };
                Some(api::ApiEdge::from_edge(e.origen_id, e.destino_id, &edge_type))
            })
            .collect();

        // 6. Calcular métricas si se pide (usando self.graph que es DiGraph<DocumentNode, EdgeType>)
        if query.include_metrics.unwrap_or(false) {
            let pagerank = algorithms::pagerank(&self.graph, 0.85, 100, 1e-6);
            let betweenness = algorithms::betweenness_centrality(&self.graph);

            for node in &mut api_nodes {
                if let Some(id_str) = node.id.strip_prefix("doc:") {
                    if let Ok(id) = id_str.parse::<i32>() {
                        node.pagerank = pagerank.get(&id).copied();
                        node.betweenness = betweenness.get(&id).copied();
                    }
                }
            }
        }

        // 7. Stats
        let total_nodes_count = query::count_nodes(&self.db).await?;
        let total_edges_count = query::count_edges(&self.db).await?;
        let communities = algorithms::label_propagation(&self.graph, 10);

        let stats = api::GraphStats {
            total_nodes: total_nodes_count,
            total_edges: total_edges_count,
            density: if total_nodes_count > 1 {
                2.0 * total_edges_count as f64
                    / (total_nodes_count as f64 * (total_nodes_count as f64 - 1.0))
            } else {
                0.0
            },
            avg_degree: if total_nodes_count > 0 {
                2.0 * total_edges_count as f64 / total_nodes_count as f64
            } else {
                0.0
            },
            num_communities: communities.len(),
        };

        let has_more = api_nodes.len() >= limit;
        let pagination = Some(api::PaginationInfo {
            cursor: api_nodes.last().and_then(|n| {
                n.id.strip_prefix("doc:").map(|s| s.to_string())
            }),
            has_more,
            returned_nodes: api_nodes.len(),
            total_available: total_nodes_count,
        });

        Ok(api::GraphResponse {
            nodes: api_nodes,
            edges: api_edges,
            stats,
            pagination,
        })
    }

    /// Calcular centralidad de un grafo completo o filtrado
    pub async fn get_centrality(
        &self,
        query: &api::CentralityQuery,
    ) -> Result<api::CentralityResponse> {
        let metric = query.metric.as_deref().unwrap_or("pagerank");
        let top_k = query.top_k.unwrap_or(10).min(50);

        match metric {
            "pagerank" => {
                let scores = algorithms::pagerank(&self.graph, 0.85, 100, 1e-6);
                let mut values: Vec<api::CentralityValue> = scores
                    .iter()
                    .map(|(node_id, score)| api::CentralityValue {
                        node_id: format!("doc:{}", node_id),
                        score: *score,
                    })
                    .filter(|v| query.threshold.is_none_or(|t| v.score >= t))
                    .collect();
                values.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                values.truncate(top_k);
                let top_ids: Vec<String> = values.iter().map(|v| v.node_id.clone()).collect();
                Ok(api::CentralityResponse { metric: "pagerank".to_string(), values, top_nodes: top_ids, threshold: query.threshold })
            }
            "betweenness" => {
                let scores = algorithms::betweenness_centrality(&self.graph);
                let mut values: Vec<api::CentralityValue> = scores
                    .iter()
                    .map(|(node_id, score)| api::CentralityValue {
                        node_id: format!("doc:{}", node_id),
                        score: *score,
                    })
                    .filter(|v| query.threshold.is_none_or(|t| v.score >= t))
                    .collect();
                values.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                values.truncate(top_k);
                let top_ids: Vec<String> = values.iter().map(|v| v.node_id.clone()).collect();
                Ok(api::CentralityResponse { metric: "betweenness".to_string(), values, top_nodes: top_ids, threshold: query.threshold })
            }
            "degree" => {
                let scores = algorithms::degree_centrality(&self.graph);
                let mut values: Vec<api::CentralityValue> = scores
                    .iter()
                    .map(|(node_id, deg)| api::CentralityValue {
                        node_id: format!("doc:{}", node_id),
                        score: deg.total_degree as f64,
                    })
                    .filter(|v| query.threshold.is_none_or(|t| v.score >= t))
                    .collect();
                values.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                values.truncate(top_k);
                let top_ids: Vec<String> = values.iter().map(|v| v.node_id.clone()).collect();
                Ok(api::CentralityResponse { metric: "degree".to_string(), values, top_nodes: top_ids, threshold: query.threshold })
            }
            _ => Err(crate::AlesysError::ApiError(format!("Métrica desconocida: {}", metric))),
        }
    }

    /// Obtener comunidades del grafo
    pub async fn get_communities(
        &self,
        query: &api::CommunitiesQuery,
    ) -> Result<api::CommunitiesResponse> {
        let max_iter = query.max_iterations.unwrap_or(10).min(50);
        let communities = algorithms::label_propagation(&self.graph, max_iter);

        let scores = algorithms::pagerank(&self.graph, 0.85, 100, 1e-6);

        let community_infos: Vec<api::CommunityInfo> = communities
            .iter()
            .enumerate()
            .map(|(comm_id, community)| {
                let member_ids: Vec<String> = community
                    .members
                    .iter()
                    .map(|id| format!("doc:{}", id))
                    .collect();

                let avg_pagerank = if community.members.is_empty() {
                    0.0
                } else {
                    let sum: f64 = community
                        .members
                        .iter()
                        .filter_map(|id| scores.get(id))
                        .sum();
                    sum / community.members.len() as f64
                };

                api::CommunityInfo {
                    id: comm_id,
                    size: community.size,
                    members: member_ids,
                    avg_pagerank,
                    label: format!("Comunidad {}", comm_id),
                }
            })
            .collect();

        Ok(api::CommunitiesResponse {
            communities: community_infos,
            algorithm: "label_propagation".to_string(),
            iterations: max_iter,
        })
    }

    /// Encontrar camino más corto entre dos nodos
    pub async fn get_shortest_path(
        &self,
        query: &api::PathQuery,
    ) -> Result<api::PathResponse> {
        let result = algorithms::shortest_path(&self.graph, query.source_id, query.target_id);
        Ok(api::PathResponse {
            source: format!("doc:{}", query.source_id),
            target: format!("doc:{}", query.target_id),
            path: result
                .path
                .iter()
                .map(|id| format!("doc:{}", id))
                .collect(),
            distance: result.distance,
            found: result.found,
            path_length: result.path.len(),
        })
    }

    /// Buscar documentos en el grafo
    pub async fn search_graph(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<api::ApiNode>> {
        let raw_nodes = query::search_nodes(&self.db, query, limit).await?;
        let node_ids: Vec<i32> = raw_nodes.iter().map(|n| n.id).collect();
        let raw_edges = query::load_edges_for_nodes(&self.db, &node_ids, None).await?;

        let api_nodes = raw_nodes
            .iter()
            .map(|n| {
                let node_id = n.id;
                let degree = raw_edges
                    .iter()
                    .filter(|e| e.origen_id == node_id || e.destino_id == node_id)
                    .count();
                api::ApiNode::from_document(node_id, &n.ruta_relativa, &n.tipo, degree, None, None, None)
            })
            .collect();

        Ok(api_nodes)
    }

    pub fn get_connected_documents(&self, doc_id: i32, max_depth: usize) -> Vec<DocumentNode> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();

        if let Some(&start_idx) = self.node_map.get(&doc_id) {
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((start_idx, 0));

            while let Some((idx, depth)) = queue.pop_front() {
                if depth >= max_depth || visited.contains(&idx) {
                    continue;
                }
                visited.insert(idx);

                if let Some(node) = self.graph.node_weight(idx) {
                    if idx != start_idx {
                        result.push(node.clone());
                    }
                }

                for neighbor in self.graph.neighbors_undirected(idx) {
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }

        result
    }
}

#[derive(Debug)]
pub struct GraphStats {
    pub nodes: usize,
    pub edges: usize,
}

pub fn build_rag_context(results: &[SearchResult], max_tokens: usize) -> String {
    let mut context = String::new();
    let mut current_tokens = 0;

    context.push_str("=== Contexto (GraphRAG) ===\n\n");

    for (i, result) in results.iter().enumerate() {
        let token_estimate = result.content.len() / 4;

        if current_tokens + token_estimate > max_tokens {
            context.push_str("\n[... más resultados truncados por límite de tokens ...]\n");
            break;
        }

        let source_label = match result.source {
            SearchResultSource::Vector => "búsqueda semántica",
            SearchResultSource::Graph => "relación en grafo",
        };

        let doc_path = result.doc_path.as_deref().unwrap_or("desconocido");

        context.push_str(&format!(
            "[Fragmento {}] (similitud: {:.2}, fuente: {}, documento: {})\n{}\n\n",
            i + 1,
            result.similarity,
            source_label,
            doc_path,
            result.content
        ));

        current_tokens += token_estimate;
    }

    context
}
