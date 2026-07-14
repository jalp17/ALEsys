//! GraphRAG - Graph + Retrieval Augmented Generation
//!
//! Combina búsqueda vectorial (pgvector) con traversales de grafo (petgraph)

use crate::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// Manager para GraphRAG
pub struct GraphRAG {
    db: PgPool,
    graph: DiGraph<DocumentNode, EdgeType>,
    node_map: HashMap<i32, NodeIndex>,
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
        let (graph, node_map) = Self::load_graph_from_db(&db).await?;
        Ok(Self {
            db,
            graph,
            node_map,
        })
    }

    pub async fn reload_graph(&mut self) -> Result<()> {
        let (graph, node_map) = Self::load_graph_from_db(&self.db).await?;
        self.graph = graph;
        self.node_map = node_map;
        Ok(())
    }

    async fn load_graph_from_db(
        db: &PgPool,
    ) -> Result<(DiGraph<DocumentNode, EdgeType>, HashMap<i32, NodeIndex>)> {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        let doc_rows = sqlx::query("SELECT id, ruta_relativa, tipo FROM documentos")
            .fetch_all(db)
            .await?;

        for row in doc_rows {
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

        let enlace_rows =
            sqlx::query("SELECT origen_id, destino_id, tipo_enlace, contexto FROM enlaces")
                .fetch_all(db)
                .await?;

        for row in enlace_rows {
            let origen_id: i32 = row.get("origen_id");
            let destino_id: i32 = row.get("destino_id");
            let tipo_enlace: Option<String> = row.get("tipo_enlace");
            let contexto: Option<String> = row.get("contexto");

            if let (Some(&src), Some(&dst)) = (node_map.get(&origen_id), node_map.get(&destino_id))
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

        Ok((graph, node_map))
    }

    pub async fn vector_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
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
        .await?;

        let results = rows
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

        for doc_id in &expanded {
            if !doc_ids.contains(doc_id) {
                if let Some(fragments) = self.load_fragments_for_document(*doc_id).await? {
                    for (frag_id, frag_content) in fragments {
                        results.push(SearchResult {
                            fragment_id: frag_id,
                            document_id: *doc_id,
                            content: frag_content,
                            similarity: 0.3,
                            source: SearchResultSource::Graph,
                            doc_path: self
                                .node_map
                                .get(doc_id)
                                .and_then(|&idx| self.graph.node_weight(idx))
                                .map(|n| n.path.clone()),
                        });
                    }
                }
            }
        }

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(results)
    }

    async fn load_fragments_for_document(&self, doc_id: i32) -> Result<Option<Vec<(i32, String)>>> {
        let rows = sqlx::query(
            "SELECT id, contenido FROM fragmentos WHERE documento_id = $1 ORDER BY indice_orden",
        )
        .bind(doc_id)
        .fetch_all(&self.db)
        .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let fragments = rows
            .into_iter()
            .map(|row| (row.get("id"), row.get("contenido")))
            .collect();

        Ok(Some(fragments))
    }

    fn expand_with_graph(&self, doc_ids: &[i32], degrees: usize) -> Vec<i32> {
        let mut expanded = std::collections::HashSet::new();
        let mut queue: std::collections::VecDeque<(i32, usize)> =
            doc_ids.iter().map(|&id| (id, 0)).collect();

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= degrees {
                continue;
            }

            if let Some(&node_idx) = self.node_map.get(&current_id) {
                for neighbor in self.graph.neighbors_undirected(node_idx) {
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
        let rows = sqlx::query(
            r#"
            SELECT f.id, f.documento_id, f.contenido, d.ruta_relativa
            FROM fragmentos f
            JOIN documentos d ON d.id = f.documento_id
            WHERE d.ruta_relativa LIKE $1
            LIMIT 10
            "#,
        )
        .bind(format!("%{}%", path_pattern))
        .fetch_all(&self.db)
        .await?;

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
