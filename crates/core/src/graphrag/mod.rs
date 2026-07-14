//! GraphRAG - Graph + Retrieval Augmented Generation
//! 
//! Combina búsqueda vectorial (pgvector) con traversales de grafo (petgraph)

use crate::Result;
use sqlx::PgPool;
use pgvector::Vector;
use petgraph::graph::{DiGraph, NodeIndex};

/// Manager para GraphRAG
pub struct GraphRAG {
    db: PgPool,
    graph: DiGraph<DocumentNode, EdgeType>,
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

impl GraphRAG {
    pub async fn new(db: PgPool) -> Result<Self> {
        let graph = Self::load_graph_from_db(&db).await?;
        Ok(Self { db, graph })
    }
    
    /// Cargar grafo desde PostgreSQL
    async fn load_graph_from_db(db: &PgPool) -> Result<DiGraph<DocumentNode, EdgeType>> {
        let mut graph = DiGraph::new();
        let mut node_map = std::collections::HashMap::new();
        
        // Cargar documentos
        let docs = sqlx::query!(
            "SELECT id, ruta_relativa, tipo FROM documentos"
        )
        .fetch_all(db)
        .await?;
        
        for doc in docs {
            let idx = graph.add_node(DocumentNode {
                id: doc.id,
                path: doc.ruta_relativa,
                doc_type: doc.tipo,
            });
            node_map.insert(doc.id, idx);
        }
        
        // Cargar enlaces
        let enlaces = sqlx::query!(
            "SELECT origen_id, destino_id, tipo_enlace, contexto FROM enlaces"
        )
        .fetch_all(db)
        .await?;
        
        for enlace in enlaces {
            if let (Some(&src), Some(&dst)) = (
                node_map.get(&enlace.origen_id),
                node_map.get(&enlace.destino_id),
            ) {
                let edge_type = match enlace.tipo_enlace.as_str() {
                    "wiki_link" => EdgeType::WikiLink { contexto: enlace.contexto },
                    "backlink" => EdgeType::Backlink { contexto: enlace.contexto },
                    _ => EdgeType::Reference { contexto: enlace.contexto },
                };
                graph.add_edge(src, dst, edge_type);
            }
        }
        
        Ok(graph)
    }
    
    /// Búsqueda híbrida: vector + grafo
    pub async fn hybrid_search(
        &self,
        query_embedding: Vector,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // 1. Búsqueda vectorial inicial
        let initial_results = sqlx::query!(
            r#"
            SELECT f.id, f.documento_id, f.contenido,
                   (f.embedding <=> $1::vector) AS distancia
            FROM fragmentos f
            ORDER BY distancia
            LIMIT $2
            "#,
            query_embedding.to_vec() as Vec<f32>,
            limit as i64
        )
        .fetch_all(&self.db)
        .await?;
        
        // 2. Expandir con grafo (1 grado)
        let doc_ids: Vec<i32> = initial_results
            .iter()
            .map(|r| r.documento_id)
            .collect();
        
        let expanded = self.expand_with_graph(&doc_ids, 1).await?;
        
        // 3. Combinar resultados
        let mut results = Vec::new();
        
        for initial in initial_results {
            results.push(SearchResult {
                fragment_id: initial.id,
                content: initial.contenido,
                similarity: 1.0 - initial.distancia,
                source: SearchResultSource::Vector,
            });
        }
        
        for expanded_id in expanded {
            results.push(SearchResult {
                fragment_id: expanded_id,
                content: String::new(),  // TODO: cargar contenido
                similarity: 0.5,  // TODO: calcular
                source: SearchResultSource::Graph,
            });
        }
        
        Ok(results)
    }
    
    /// Expandir resultados usando el grafo
    async fn expand_with_graph(&self, doc_ids: &[i32], degrees: usize) -> Result<Vec<i32>> {
        // TODO: Implementar traversal de grafo con petgraph
        todo!("Implementar expand_with_graph")
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub fragment_id: i32,
    pub content: String,
    pub similarity: f32,
    pub source: SearchResultSource,
}

#[derive(Debug, Clone, Copy)]
pub enum SearchResultSource {
    Vector,
    Graph,
}

/// Construir contexto para LLM desde resultados de búsqueda
pub fn build_rag_context(results: &[SearchResult]) -> String {
    let mut context = String::new();
    
    context.push_str("=== Contexto (GraphRAG) ===\n\n");
    
    for (i, result) in results.iter().enumerate() {
        context.push_str(&format!(
            "[Fragmento {}] (similitud: {:.2}, fuente: {:?})\n{}\n\n",
            i + 1,
            result.similarity,
            result.source,
            result.content
        ));
    }
    
    context
}