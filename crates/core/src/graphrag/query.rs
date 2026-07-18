//! Consultas SQL optimizadas para el grafo
//!
//! - Carga de subgrafos paginada
//! - Filtrado por tipo de documento/enlace
//! - Soporte de permisos por usuario

use crate::Result;
use sqlx::{PgPool, Row};

/// Nodo crudo desde SQL
#[derive(Debug, Clone)]
pub struct RawNode {
    pub id: i32,
    pub ruta_relativa: String,
    pub tipo: String,
}

/// Arista cruda desde SQL
#[derive(Debug, Clone)]
pub struct RawEdge {
    pub origen_id: i32,
    pub destino_id: i32,
    pub tipo_enlace: Option<String>,
    pub contexto: Option<String>,
}

/// Resultado de carga de subgrafo
#[derive(Debug)]
pub struct SubgraphData {
    pub nodes: Vec<RawNode>,
    pub edges: Vec<RawEdge>,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub has_more: bool,
}

// =============================================================================
// Node Queries
// =============================================================================

/// Cargar documentos paginados (cursor-based, sin OFFSET)
pub async fn load_nodes_paginated(
    db: &PgPool,
    cursor: Option<i32>,
    limit: usize,
    doc_type_filter: Option<&str>,
) -> Result<SubgraphData> {
    let cursor_id = cursor.unwrap_or(0);
    let limit_i64 = limit as i64;

    let rows = if let Some(dt) = doc_type_filter {
        sqlx::query(
            "SELECT id, ruta_relativa, tipo FROM documentos
             WHERE id > $1 AND tipo = $2
             ORDER BY id LIMIT $3",
        )
        .bind(cursor_id)
        .bind(dt)
        .bind(limit_i64)
        .fetch_all(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error cargando nodos filtrados: {}", e);
            crate::AlesysError::Database(e)
        })?
    } else {
        sqlx::query(
            "SELECT id, ruta_relativa, tipo FROM documentos
             WHERE id > $1
             ORDER BY id LIMIT $2",
        )
        .bind(cursor_id)
        .bind(limit_i64)
        .fetch_all(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error cargando nodos: {}", e);
            crate::AlesysError::Database(e)
        })?
    };

    let total_nodes = rows.len();
    let has_more = total_nodes == limit;

    let nodes: Vec<RawNode> = rows
        .iter()
        .map(|row| RawNode {
            id: row.get("id"),
            ruta_relativa: row.get("ruta_relativa"),
            tipo: row.get("tipo"),
        })
        .collect();

    Ok(SubgraphData {
        nodes,
        edges: Vec::new(), // Se carga por separado
        total_nodes,
        total_edges: 0,
        has_more,
    })
}

/// Cargar documentos con IDs específicos
pub async fn load_nodes_by_ids(db: &PgPool, ids: &[i32]) -> Result<Vec<RawNode>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query(
        "SELECT id, ruta_relativa, tipo FROM documentos WHERE id = ANY($1) ORDER BY id",
    )
    .bind(ids)
    .fetch_all(db)
    .await
    .map_err(|e| {
        tracing::error!("DB error cargando nodos por IDs: {}", e);
        crate::AlesysError::Database(e)
    })?;

    let nodes = rows
        .iter()
        .map(|row| RawNode {
            id: row.get("id"),
            ruta_relativa: row.get("ruta_relativa"),
            tipo: row.get("tipo"),
        })
        .collect();

    Ok(nodes)
}

/// Cargar documentos vecinos de un nodo (para subgrafo local)
pub async fn load_neighbor_nodes(
    db: &PgPool,
    center_id: i32,
    depth: usize,
) -> Result<Vec<RawNode>> {
    // BFS en SQL: cargar nodos alcanzables desde center_id
    let mut visited_ids = std::collections::HashSet::new();
    visited_ids.insert(center_id);
    let mut current_ids = vec![center_id];

    for _ in 0..depth {
        if current_ids.is_empty() {
            break;
        }

        let rows = sqlx::query(
            "SELECT DISTINCT
                CASE WHEN origen_id = ANY($1) THEN destino_id ELSE origen_id END AS neighbor_id
             FROM enlaces
             WHERE origen_id = ANY($1) OR destino_id = ANY($1)",
        )
        .bind(&current_ids)
        .fetch_all(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error cargando vecinos: {}", e);
            crate::AlesysError::Database(e)
        })?;

        current_ids.clear();
        for row in &rows {
            let neighbor_id: i32 = row.get("neighbor_id");
            if !visited_ids.contains(&neighbor_id) {
                visited_ids.insert(neighbor_id);
                current_ids.push(neighbor_id);
            }
        }
    }

    let ids: Vec<i32> = visited_ids.into_iter().collect();
    load_nodes_by_ids(db, &ids).await
}

// =============================================================================
// Edge Queries
// =============================================================================

/// Cargar aristas entre un conjunto de nodos
pub async fn load_edges_for_nodes(
    db: &PgPool,
    node_ids: &[i32],
    edge_type_filter: Option<&str>,
) -> Result<Vec<RawEdge>> {
    if node_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = if let Some(et) = edge_type_filter {
        sqlx::query(
            "SELECT origen_id, destino_id, tipo_enlace, contexto
             FROM enlaces
             WHERE (origen_id = ANY($1) OR destino_id = ANY($1))
               AND tipo_enlace = $2",
        )
        .bind(node_ids)
        .bind(et)
        .fetch_all(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error cargando aristas filtradas: {}", e);
            crate::AlesysError::Database(e)
        })?
    } else {
        sqlx::query(
            "SELECT origen_id, destino_id, tipo_enlace, contexto
             FROM enlaces
             WHERE origen_id = ANY($1) OR destino_id = ANY($1)",
        )
        .bind(node_ids)
        .fetch_all(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error cargando aristas: {}", e);
            crate::AlesysError::Database(e)
        })?
    };

    let edges: Vec<RawEdge> = rows
        .iter()
        .map(|row| RawEdge {
            origen_id: row.get("origen_id"),
            destino_id: row.get("destino_id"),
            tipo_enlace: row.get("tipo_enlace"),
            contexto: row.get("contexto"),
        })
        .collect();

    Ok(edges)
}

/// Cargar aristas de un nodo específico (para degree calculation)
pub async fn load_edges_for_node(db: &PgPool, node_id: i32) -> Result<(usize, usize)> {
    let row_in = sqlx::query("SELECT COUNT(*) as cnt FROM enlaces WHERE destino_id = $1")
        .bind(node_id)
        .fetch_one(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error contando in-edges: {}", e);
            crate::AlesysError::Database(e)
        })?;

    let row_out = sqlx::query("SELECT COUNT(*) as cnt FROM enlaces WHERE origen_id = $1")
        .bind(node_id)
        .fetch_one(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error contando out-edges: {}", e);
            crate::AlesysError::Database(e)
        })?;

    let in_degree: i64 = row_in.get("cnt");
    let out_degree: i64 = row_out.get("cnt");

    Ok((in_degree as usize, out_degree as usize))
}

/// Contar total de documentos
pub async fn count_nodes(db: &PgPool) -> Result<usize> {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM documentos")
        .fetch_one(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error contando nodos: {}", e);
            crate::AlesysError::Database(e)
        })?;

    let count: i64 = row.get("cnt");
    Ok(count as usize)
}

/// Contar total de enlaces
pub async fn count_edges(db: &PgPool) -> Result<usize> {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM enlaces")
        .fetch_one(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error contando aristas: {}", e);
            crate::AlesysError::Database(e)
        })?;

    let count: i64 = row.get("cnt");
    Ok(count as usize)
}

/// Contar enlaces por tipo
pub async fn count_edges_by_type(db: &PgPool) -> Result<std::collections::HashMap<String, usize>> {
    let rows = sqlx::query("SELECT tipo_enlace, COUNT(*) as cnt FROM enlaces GROUP BY tipo_enlace")
        .fetch_all(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error contando aristas por tipo: {}", e);
            crate::AlesysError::Database(e)
        })?;

    let mut counts = std::collections::HashMap::new();
    for row in rows {
        let tipo: Option<String> = row.get("tipo_enlace");
        let cnt: i64 = row.get("cnt");
        counts.insert(tipo.unwrap_or_else(|| "unknown".to_string()), cnt as usize);
    }

    Ok(counts)
}

// =============================================================================
// Permission Queries
// =============================================================================

/// Verificar si un usuario tiene acceso a un documento
pub async fn check_document_permission(db: &PgPool, user_id: i32, doc_id: i32) -> Result<bool> {
    // Admin (user_id = 0) tiene acceso a todo
    if user_id == 0 {
        return Ok(true);
    }

    let row = sqlx::query(
        "SELECT EXISTS(
            SELECT 1 FROM graph_permissions
            WHERE user_id = $1 AND doc_id = $2
        ) as has_access",
    )
    .bind(user_id)
    .bind(doc_id)
    .fetch_one(db)
    .await
    .map_err(|e| {
        tracing::error!("DB error verificando permiso: {}", e);
        crate::AlesysError::Database(e)
    })?;

    let has_access: bool = row.get("has_access");
    Ok(has_access)
}

/// Obtener IDs de documentos accesibles para un usuario
pub async fn get_accessible_doc_ids(db: &PgPool, user_id: i32) -> Result<Vec<i32>> {
    // Admin (user_id = 0) ve todos
    if user_id == 0 {
        let rows = sqlx::query("SELECT id FROM documentos ORDER BY id")
            .fetch_all(db)
            .await
            .map_err(|e| {
                tracing::error!("DB error obteniendo todos los docs: {}", e);
                crate::AlesysError::Database(e)
            })?;
        return Ok(rows.iter().map(|r| r.get("id")).collect());
    }

    let rows =
        sqlx::query("SELECT doc_id FROM graph_permissions WHERE user_id = $1 ORDER BY doc_id")
            .bind(user_id)
            .fetch_all(db)
            .await
            .map_err(|e| {
                tracing::error!("DB error obteniendo docs accesibles: {}", e);
                crate::AlesysError::Database(e)
            })?;

    Ok(rows.iter().map(|r| r.get("doc_id")).collect())
}

/// Buscar documentos por nombre (para search en grafo)
pub async fn search_nodes(db: &PgPool, query: &str, limit: usize) -> Result<Vec<RawNode>> {
    let pattern = format!("%{}%", query.to_lowercase());
    let rows = sqlx::query(
        "SELECT id, ruta_relativa, tipo
         FROM documentos
         WHERE LOWER(ruta_relativa) LIKE $1
         ORDER BY id
         LIMIT $2",
    )
    .bind(&pattern)
    .bind(limit as i64)
    .fetch_all(db)
    .await
    .map_err(|e| {
        tracing::error!("DB error buscando nodos: {}", e);
        crate::AlesysError::Database(e)
    })?;

    let nodes = rows
        .iter()
        .map(|row| RawNode {
            id: row.get("id"),
            ruta_relativa: row.get("ruta_relativa"),
            tipo: row.get("tipo"),
        })
        .collect();

    Ok(nodes)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_node_creation() {
        let node = RawNode {
            id: 1,
            ruta_relativa: "docs/readme.md".to_string(),
            tipo: "markdown".to_string(),
        };
        assert_eq!(node.id, 1);
        assert_eq!(node.tipo, "markdown");
    }

    #[test]
    fn test_raw_edge_creation() {
        let edge = RawEdge {
            origen_id: 1,
            destino_id: 2,
            tipo_enlace: Some("wiki_link".to_string()),
            contexto: Some("context".to_string()),
        };
        assert_eq!(edge.origen_id, 1);
        assert_eq!(edge.tipo_enlace.as_deref(), Some("wiki_link"));
    }
}
