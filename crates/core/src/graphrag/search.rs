//! Búsqueda híbrida avanzada (Fase 6)
//!
//! Componentes:
//! - `AdvancedSearchQuery`: Builder para queries con filtros múltiples
//! - `rrf_fusion`: Reciprocal Rank Fusion para combinar scores
//! - `highlight_terms`: Resaltado de términos en resultados
//! - `expand_query`: Expansión de queries con co-ocurrencias
//! - `AdvancedSearchResult`: Resultado con score desglosado

use crate::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

// =============================================================================
// Request Types
// =============================================================================

/// Parámetros de vector search
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VectorParams {
    #[serde(default = "default_vector_limit")]
    pub limit: usize,
    #[serde(default = "default_vector_weight")]
    pub weight: f32,
}

fn default_vector_limit() -> usize {
    10
}
fn default_vector_weight() -> f32 {
    1.0
}

/// Parámetros de graph expansion
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GraphParams {
    #[serde(default = "default_graph_degrees")]
    pub degrees: usize,
    #[serde(default = "default_graph_weight")]
    pub weight: f32,
    #[serde(default)]
    pub centrality_boost: Option<String>,
}

fn default_graph_degrees() -> usize {
    1
}
fn default_graph_weight() -> f32 {
    0.5
}

/// Filtros de búsqueda
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SearchFilters {
    #[serde(default)]
    pub doc_types: Vec<String>,
    #[serde(default)]
    pub areas: Vec<i32>,
    #[serde(default)]
    pub subareas: Vec<i32>,
    #[serde(default)]
    pub date_from: Option<String>,
    #[serde(default)]
    pub date_to: Option<String>,
    #[serde(default)]
    pub content_pattern: Option<String>,
}

/// Parámetros de query expansion
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExpansionParams {
    #[serde(default = "default_expansion_enabled")]
    pub enabled: bool,
    #[serde(default = "default_max_terms")]
    pub max_terms: usize,
}

fn default_expansion_enabled() -> bool {
    true
}
fn default_max_terms() -> usize {
    5
}

/// Parámetros de highlighting
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HighlightParams {
    #[serde(default = "default_highlight_enabled")]
    pub enabled: bool,
    #[serde(default = "default_frag_size")]
    pub frag_size: usize,
}

fn default_highlight_enabled() -> bool {
    true
}
fn default_frag_size() -> usize {
    150
}

/// Query de búsqueda avanzada
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdvancedSearchQuery {
    /// Texto de búsqueda original
    pub query: String,

    /// Parámetros vectoriales
    #[serde(default)]
    pub vector: VectorParams,

    /// Parámetros de grafo
    #[serde(default)]
    pub graph: GraphParams,

    /// Filtros SQL
    #[serde(default)]
    pub filters: SearchFilters,

    /// Expansión de query
    #[serde(default)]
    pub expansion: ExpansionParams,

    /// Highlighting
    #[serde(default)]
    pub highlight: HighlightParams,

    /// Límite de resultados
    #[serde(default = "default_search_limit")]
    pub limit: usize,

    /// Offset para paginación
    #[serde(default)]
    pub offset: usize,
}

fn default_search_limit() -> usize {
    20
}

impl Default for AdvancedSearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            vector: VectorParams {
                limit: 10,
                weight: 1.0,
            },
            graph: GraphParams {
                degrees: 1,
                weight: 0.5,
                centrality_boost: None,
            },
            filters: SearchFilters::default(),
            expansion: ExpansionParams {
                enabled: true,
                max_terms: 5,
            },
            highlight: HighlightParams {
                enabled: true,
                frag_size: 150,
            },
            limit: 20,
            offset: 0,
        }
    }
}

// =============================================================================
// Response Types
// =============================================================================

/// Score desglosado por fuente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub vector: f32,
    pub graph: f32,
    pub rrf: f32,
}

/// Resultado de búsqueda avanzada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSearchResult {
    pub fragment_id: i32,
    pub document_id: i32,
    pub path: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlighted: Option<String>,
    pub similarity: f32,
    pub score_breakdown: ScoreBreakdown,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Respuesta completa de búsqueda avanzada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSearchResponse {
    pub results: Vec<AdvancedSearchResult>,
    pub total: usize,
    pub took_ms: u64,
    #[serde(default)]
    pub expanded_terms: Vec<String>,
}

// =============================================================================
// RRF Fusion
// =============================================================================

/// Reciprocal Rank Fusion (RRF) para combinar múltiples rankings
///
/// Score = sum_i(1 / (k + rank_i)) donde k es un parámetro de suavizado.
/// Basado en: "Reciprocal Rank Fusion outperforms Condorcet and individual
/// Rank Learning Methods" (Cormack et al., 2009)
pub fn rrf_fusion(
    vector_results: &[(i32, f32)], // (fragment_id, similarity)
    graph_results: &[(i32, f32)],  // (fragment_id, score)
    sql_results: &[(i32, f32)],    // (fragment_id, relevance)
    k: usize,
) -> Vec<(i32, f32, ScoreBreakdown)> {
    let mut scores: HashMap<i32, (f32, f32, f32, f32)> = HashMap::new();

    // Vector scores
    for (rank, &(frag_id, _)) in vector_results.iter().enumerate() {
        let entry = scores.entry(frag_id).or_insert((0.0, 0.0, 0.0, 0.0));
        let rrf_score = 1.0 / (k as f32 + rank as f32 + 1.0);
        entry.0 += rrf_score;
        entry.1 = rrf_score;
    }

    // Graph scores
    for (rank, &(frag_id, _)) in graph_results.iter().enumerate() {
        let entry = scores.entry(frag_id).or_insert((0.0, 0.0, 0.0, 0.0));
        let rrf_score = 1.0 / (k as f32 + rank as f32 + 1.0);
        entry.0 += rrf_score;
        entry.2 = rrf_score;
    }

    // SQL/full-text scores
    for (rank, &(frag_id, _)) in sql_results.iter().enumerate() {
        let entry = scores.entry(frag_id).or_insert((0.0, 0.0, 0.0, 0.0));
        let rrf_score = 1.0 / (k as f32 + rank as f32 + 1.0);
        entry.0 += rrf_score;
        entry.3 = rrf_score;
    }

    let mut results: Vec<(i32, f32, ScoreBreakdown)> = scores
        .into_iter()
        .map(|(frag_id, (total, vector, graph, _sql))| {
            (
                frag_id,
                total,
                ScoreBreakdown {
                    vector,
                    graph,
                    rrf: total,
                },
            )
        })
        .collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

// =============================================================================
// Highlighting
// =============================================================================

/// Resalta términos de búsqueda en el contenido
///
/// Extrae fragmentos alrededor de las coincidencias y envuelve en `<mark>`.
pub fn highlight_terms(content: &str, query: &str, frag_size: usize, max_frags: usize) -> String {
    let lower_content = content.to_lowercase();
    let terms: Vec<String> = query
        .split_whitespace()
        .filter(|t| t.len() > 2)
        .map(|t| t.to_lowercase())
        .collect();

    if terms.is_empty() {
        let preview = if content.len() > frag_size {
            format!("{}...", &content[..frag_size])
        } else {
            content.to_string()
        };
        return preview;
    }

    // Encontrar todas las posiciones de coincidencias
    let mut matches: Vec<(usize, usize)> = Vec::new(); // (start, end) de coincidencias
    for term in &terms {
        let mut start = 0;
        while let Some(pos) = lower_content[start..].find(term.as_str()) {
            let abs_pos = start + pos;
            matches.push((abs_pos, abs_pos + term.len()));
            start = abs_pos + 1;
        }
    }

    if matches.is_empty() {
        let preview = if content.len() > frag_size {
            format!("{}...", &content[..frag_size])
        } else {
            content.to_string()
        };
        return preview;
    }

    // Ordenar y fusionar coincidencias superpuestas
    matches.sort_by_key(|m| m.0);
    let merged = merge_overlapping(&matches);

    // Extraer fragmentos alrededor de las coincidencias
    let mut fragments: Vec<String> = Vec::new();
    let content_len = content.len();

    for (start, end) in merged.iter().take(max_frags) {
        // Ventana expandida
        let window_start = start.saturating_sub(frag_size / 3);
        let window_end = std::cmp::min(content_len, end + frag_size / 3);

        // Buscar límites de palabra
        let adjusted_start = find_word_boundary_start(content, window_start);
        let adjusted_end = find_word_boundary_end(content, window_end);

        let before = &content[adjusted_start..*start];
        let term = &content[*start..*end];
        let after = &content[*end..adjusted_end];

        let mut frag = String::new();
        if adjusted_start > 0 {
            frag.push_str("...");
        }
        frag.push_str(before);
        frag.push_str("<mark>");
        frag.push_str(term);
        frag.push_str("</mark>");
        frag.push_str(after);
        if adjusted_end < content_len {
            frag.push_str("...");
        }

        fragments.push(frag);
    }

    if fragments.is_empty() {
        if content.len() > frag_size {
            format!("{}...", &content[..frag_size])
        } else {
            content.to_string()
        }
    } else {
        fragments.join("\n")
    }
}

/// Fusiona rangos superpuestos o adyacentes
fn merge_overlapping(ranges: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if ranges.is_empty() {
        return Vec::new();
    }

    let mut merged = Vec::new();
    let mut current = ranges[0];

    for &(start, end) in &ranges[1..] {
        if start <= current.1 + 1 {
            // Adyacente o superpuesto (con margen de 1 char)
            current.1 = current.1.max(end);
        } else {
            merged.push(current);
            current = (start, end);
        }
    }
    merged.push(current);
    merged
}

/// Encuentra el límite de palabra anterior más cercano
fn find_word_boundary_start(content: &str, pos: usize) -> usize {
    let bytes = content.as_bytes();
    let mut p = pos;
    while p > 0 && bytes[p - 1] != b' ' && bytes[p - 1] != b'\n' && bytes[p - 1] != b'\t' {
        p -= 1;
    }
    p
}

/// Encuentra el límite de palabra posterior más cercano
fn find_word_boundary_end(content: &str, pos: usize) -> usize {
    let bytes = content.as_bytes();
    let mut p = pos;
    while p < bytes.len() && bytes[p] != b' ' && bytes[p] != b'\n' && bytes[p] != b'\t' {
        p += 1;
    }
    p
}

// =============================================================================
// Query Expansion
// =============================================================================

/// Expande un query con términos de co-ocurrencia
///
/// Busca en fragmentos que contienen el término original y extrae
/// palabras que co-ocurren frecuentemente.
pub async fn expand_query(db: &PgPool, query: &str, max_terms: usize) -> Result<Vec<String>> {
    let terms: Vec<String> = query
        .split_whitespace()
        .filter(|t| t.len() > 3)
        .map(|t| t.to_lowercase())
        .collect();

    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let mut expanded: Vec<String> = Vec::new();

    // Batch query: fetch all fragments matching any term in a single query
    if !terms.is_empty() {
        let like_conditions: Vec<String> = terms
            .iter()
            .enumerate()
            .map(|(i, _)| format!("LOWER(contenido) LIKE ${}", i + 1))
            .collect();
        let sql = format!(
            "SELECT contenido FROM fragmentos WHERE {} LIMIT 200",
            like_conditions.join(" OR ")
        );
        let mut query = sqlx::query(&sql);
        for term in &terms {
            query = query.bind(format!("%{}%", term));
        }
        let rows = query.fetch_all(db).await.map_err(|e| {
            tracing::error!("DB error en expand_query: {}", e);
            crate::AlesysError::Database(e)
        })?;

        let stop_words: std::collections::HashSet<&str> = [
            "el", "la", "los", "las", "un", "una", "uno", "de", "del", "al", "en", "con", "por",
            "para", "que", "es", "se", "no", "su", "como", "más", "pero", "este", "esta", "estos",
            "estas", "y", "o", "a", "e", "i", "u", "the", "a", "an", "and", "or", "but", "in",
            "on", "at", "to", "for", "of", "with", "by", "from", "is", "are", "was", "were",
        ]
        .iter()
        .copied()
        .collect();

        let mut word_counts: HashMap<String, usize> = HashMap::new();
        for row in &rows {
            let content: String = row.get("contenido");
            let words: Vec<&str> = content.split_whitespace().collect();

            for window in words.windows(5) {
                let window_lower: Vec<String> = window.iter().map(|w| w.to_lowercase()).collect();
                if window_lower.iter().any(|w| terms.contains(w)) {
                    for word in window {
                        let w_lower = word.to_lowercase();
                        if w_lower.len() > 3
                            && !terms.contains(&w_lower)
                            && !stop_words.contains(w_lower.as_str())
                            && !expanded.contains(&w_lower)
                        {
                            *word_counts.entry(w_lower).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // Tomar las más frecuentes
        let mut counts: Vec<(String, usize)> = word_counts.into_iter().collect();
        counts.sort_by_key(|b| std::cmp::Reverse(b.1));

        for (word, _) in counts.into_iter().take(2) {
            if expanded.len() < max_terms {
                expanded.push(word);
            }
        }
    }

    expanded.truncate(max_terms);
    tracing::debug!("Query expansion: '{}' -> {:?}", query, expanded);
    Ok(expanded)
}

// =============================================================================
// SQL Query Builder
// =============================================================================

/// Construye SQL dinámico para búsqueda avanzada
///
/// Retorna (sql_string, bind_values) donde bind_values es un Vec de valores
/// que se bindean secuencialmente ($1, $2, ...).
pub fn build_search_sql(query: &AdvancedSearchQuery) -> (String, Vec<String>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();
    let mut param_idx = 1;

    // Full-text search condition
    if !query.query.is_empty() {
        conditions.push(format!("LOWER(f.contenido) LIKE ${}", param_idx));
        bind_values.push(format!("%{}%", query.query.to_lowercase()));
        param_idx += 1;
    }

    // Doc type filter
    if !query.filters.doc_types.is_empty() {
        let placeholders: Vec<String> = query
            .filters
            .doc_types
            .iter()
            .map(|_| {
                let p = format!("${}", param_idx);
                param_idx += 1;
                p
            })
            .collect();
        conditions.push(format!("d.tipo IN ({})", placeholders.join(", ")));
        bind_values.extend(query.filters.doc_types.iter().cloned());
    }

    // Area filter
    if !query.filters.areas.is_empty() {
        let placeholders: Vec<String> = (0..query.filters.areas.len())
            .map(|_| {
                let p = format!("${}", param_idx);
                param_idx += 1;
                p
            })
            .collect();
        conditions.push(format!("d.area_id IN ({})", placeholders.join(", ")));
        bind_values.extend(query.filters.areas.iter().map(|a| a.to_string()));
    }

    // Subarea filter
    if !query.filters.subareas.is_empty() {
        let placeholders: Vec<String> = (0..query.filters.subareas.len())
            .map(|_| {
                let p = format!("${}", param_idx);
                param_idx += 1;
                p
            })
            .collect();
        conditions.push(format!("d.subarea_id IN ({})", placeholders.join(", ")));
        bind_values.extend(query.filters.subareas.iter().map(|s| s.to_string()));
    }

    // Date range filter
    if let Some(ref date_from) = query.filters.date_from {
        conditions.push(format!("d.creado_en >= ${}", param_idx));
        bind_values.push(date_from.clone());
        param_idx += 1;
    }
    if let Some(ref date_to) = query.filters.date_to {
        conditions.push(format!("d.creado_en <= ${}", param_idx));
        bind_values.push(date_to.clone());
        param_idx += 1;
    }

    // Content pattern filter
    if let Some(ref pattern) = query.filters.content_pattern {
        conditions.push(format!("LOWER(d.ruta_relativa) LIKE ${}", param_idx));
        bind_values.push(format!("%{}%", pattern.to_lowercase()));
        param_idx += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        r#"
        SELECT f.id as fragment_id, f.documento_id, f.contenido,
               d.ruta_relativa
        FROM fragmentos f
        JOIN documentos d ON d.id = f.documento_id
        {}
        ORDER BY f.documento_id, f.indice_orden
        LIMIT ${} OFFSET ${}
        "#,
        where_clause,
        param_idx,
        param_idx + 1
    );

    (sql, bind_values)
}

/// Construye SQL de conteo para la misma query (sin LIMIT/OFFSET)
pub fn build_search_sql_count(query: &AdvancedSearchQuery) -> (String, Vec<String>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();
    let mut param_idx = 1;

    // Full-text search condition
    if !query.query.is_empty() {
        conditions.push(format!("LOWER(f.contenido) LIKE ${}", param_idx));
        bind_values.push(format!("%{}%", query.query.to_lowercase()));
        param_idx += 1;
    }

    // Doc type filter
    if !query.filters.doc_types.is_empty() {
        let placeholders: Vec<String> = query
            .filters
            .doc_types
            .iter()
            .map(|_| {
                let p = format!("${}", param_idx);
                param_idx += 1;
                p
            })
            .collect();
        conditions.push(format!("d.tipo IN ({})", placeholders.join(", ")));
        bind_values.extend(query.filters.doc_types.iter().cloned());
    }

    // Area filter
    if !query.filters.areas.is_empty() {
        let placeholders: Vec<String> = (0..query.filters.areas.len())
            .map(|_| {
                let p = format!("${}", param_idx);
                param_idx += 1;
                p
            })
            .collect();
        conditions.push(format!("d.area_id IN ({})", placeholders.join(", ")));
        bind_values.extend(query.filters.areas.iter().map(|a| a.to_string()));
    }

    // Subarea filter
    if !query.filters.subareas.is_empty() {
        let placeholders: Vec<String> = (0..query.filters.subareas.len())
            .map(|_| {
                let p = format!("${}", param_idx);
                param_idx += 1;
                p
            })
            .collect();
        conditions.push(format!("d.subarea_id IN ({})", placeholders.join(", ")));
        bind_values.extend(query.filters.subareas.iter().map(|s| s.to_string()));
    }

    // Date range filter
    if let Some(ref date_from) = query.filters.date_from {
        conditions.push(format!("d.creado_en >= ${}", param_idx));
        bind_values.push(date_from.clone());
        param_idx += 1;
    }
    if let Some(ref date_to) = query.filters.date_to {
        conditions.push(format!("d.creado_en <= ${}", param_idx));
        bind_values.push(date_to.clone());
        param_idx += 1;
    }

    // Content pattern filter
    if let Some(ref pattern) = query.filters.content_pattern {
        conditions.push(format!("LOWER(d.ruta_relativa) LIKE ${}", param_idx));
        bind_values.push(format!("%{}%", pattern.to_lowercase()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        r#"
        SELECT COUNT(*) as cnt
        FROM fragmentos f
        JOIN documentos d ON d.id = f.documento_id
        {}
        "#,
        where_clause
    );

    (sql, bind_values)
}

// =============================================================================
// Advanced Search Executor
// =============================================================================

/// Ejecuta búsqueda avanzada completa
pub async fn advanced_search(
    db: &PgPool,
    query: &AdvancedSearchQuery,
    embedding: Option<&[f32]>,
    graphrag: Option<&crate::graphrag::GraphRAG>,
) -> Result<AdvancedSearchResponse> {
    let start = std::time::Instant::now();
    let mut expanded_terms: Vec<String> = Vec::new();

    // 1. Query expansion (only if query is non-empty)
    let search_terms = if query.expansion.enabled && !query.query.is_empty() {
        let mut terms = expand_query(db, &query.query, query.expansion.max_terms).await?;
        expanded_terms = terms.clone();
        let mut all_terms = vec![query.query.clone()];
        all_terms.append(&mut terms);
        all_terms.join(" ")
    } else {
        query.query.clone()
    };

    // 2. SQL/Full-text search with count
    let (sql, bind_values) = build_search_sql(query);

    // Execute count query first
    let (count_sql, count_binds) = build_search_sql_count(query);
    let total = {
        let mut count_query = sqlx::query(&count_sql);
        for val in &count_binds {
            count_query = count_query.bind(val);
        }
        count_query
            .fetch_one(db)
            .await
            .map(|row| {
                let cnt: i64 = row.get("cnt");
                cnt as usize
            })
            .unwrap_or(0)
    };

    let mut sqlx_query = sqlx::query(&sql);
    for val in &bind_values {
        sqlx_query = sqlx_query.bind(val);
    }
    sqlx_query = sqlx_query
        .bind(query.limit as i64)
        .bind(query.offset as i64);

    let sql_rows = sqlx_query.fetch_all(db).await.map_err(|e| {
        tracing::error!("DB error en advanced_search SQL: {}", e);
        crate::AlesysError::Database(e)
    })?;

    let sql_results: Vec<(i32, f32)> = sql_rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let frag_id: i32 = row.get("fragment_id");
            // Relevance score: higher for earlier matches, normalized 0..1
            let total_rows = sql_rows.len().max(1);
            let score = 1.0 - (i as f32 / total_rows as f32);
            (frag_id, score)
        })
        .collect();

    // 3. Vector search (si hay embedding)
    let vector_results: Vec<(i32, f32)> = if let Some(emb) = embedding {
        let vector_limit = query.vector.limit;
        let embedding_str = format!(
            "[{}]",
            emb.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let rows = sqlx::query(
            r#"
            SELECT f.id, (f.embedding <=> $1::vector) AS distancia
            FROM fragmentos f
            JOIN documentos d ON d.id = f.documento_id
            ORDER BY distancia
            LIMIT $2
            "#,
        )
        .bind(&embedding_str)
        .bind(vector_limit as i64)
        .fetch_all(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error en advanced_search vector: {}", e);
            crate::AlesysError::Database(e)
        })?;

        rows.iter()
            .map(|row| {
                let frag_id: i32 = row.get("id");
                let dist: f64 = row.get("distancia");
                let similarity = 1.0 - dist as f32;
                (frag_id, similarity)
            })
            .collect()
    } else {
        Vec::new()
    };

    // 4. Graph expansion search
    let graph_results: Vec<(i32, f32)> = if let Some(gr) = graphrag {
        if let Some(emb) = embedding {
            let mut results = gr.vector_search(emb, query.vector.limit.min(5)).await?;
            let doc_ids: Vec<i32> = results
                .iter()
                .map(|r| r.document_id)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let expanded = gr.expand_with_graph_test(&doc_ids, query.graph.degrees);

            let new_doc_ids: Vec<i32> = expanded
                .iter()
                .filter(|id| !doc_ids.contains(id))
                .copied()
                .collect();

            if !new_doc_ids.is_empty() {
                let graph_rows = sqlx::query(
                    r#"
                    SELECT f.id as frag_id, f.documento_id, f.contenido, d.ruta_relativa
                    FROM fragmentos f
                    JOIN documentos d ON d.id = f.documento_id
                    WHERE f.documento_id = ANY($1)
                    LIMIT 20
                    "#,
                )
                .bind(&new_doc_ids)
                .fetch_all(db)
                .await
                .map_err(|e| {
                    tracing::error!("DB error cargando fragmentos graph: {}", e);
                    crate::AlesysError::Database(e)
                })?;

                for row in graph_rows {
                    let frag_id: i32 = row.get("frag_id");
                    let doc_id: i32 = row.get("documento_id");
                    let content: String = row.get("contenido");
                    let path: Option<String> = row.get("ruta_relativa");
                    results.push(crate::graphrag::SearchResult {
                        fragment_id: frag_id,
                        document_id: doc_id,
                        content,
                        similarity: 0.3,
                        source: crate::graphrag::SearchResultSource::Graph,
                        doc_path: path,
                    });
                }
            }

            results
                .iter()
                .map(|r| (r.fragment_id, r.similarity))
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 5. RRF fusion
    let k = 60; // Standard RRF parameter
    let fused = rrf_fusion(&vector_results, &graph_results, &sql_results, k);

    // 6. Load full results with content
    let frag_ids: Vec<i32> = fused.iter().map(|(id, _, _)| *id).collect();
    let mut results: Vec<AdvancedSearchResult> = Vec::new();

    if !frag_ids.is_empty() {
        let rows = sqlx::query(
            r#"
            SELECT f.id as fragment_id, f.documento_id, f.contenido,
                   d.ruta_relativa
            FROM fragmentos f
            JOIN documentos d ON d.id = f.documento_id
            WHERE f.id = ANY($1)
            "#,
        )
        .bind(&frag_ids)
        .fetch_all(db)
        .await
        .map_err(|e| {
            tracing::error!("DB error cargando resultados finales: {}", e);
            crate::AlesysError::Database(e)
        })?;

        let mut content_map: HashMap<i32, (i32, String, Option<String>)> = HashMap::new();
        for row in &rows {
            let frag_id: i32 = row.get("fragment_id");
            let doc_id: i32 = row.get("documento_id");
            let content: String = row.get("contenido");
            let path: Option<String> = row.get("ruta_relativa");
            content_map.insert(frag_id, (doc_id, content, path));
        }

        for (frag_id, score, breakdown) in &fused {
            if let Some((doc_id, content, path)) = content_map.get(frag_id) {
                let highlighted = if query.highlight.enabled {
                    Some(highlight_terms(
                        content,
                        &search_terms,
                        query.highlight.frag_size,
                        3,
                    ))
                } else {
                    None
                };

                results.push(AdvancedSearchResult {
                    fragment_id: *frag_id,
                    document_id: *doc_id,
                    path: path.clone(),
                    content: content.clone(),
                    highlighted,
                    similarity: *score,
                    score_breakdown: breakdown.clone(),
                    source: None,
                });
            }
        }
    }

    let took_ms = start.elapsed().as_millis() as u64;

    tracing::info!(
        "Advanced search '{}' -> {} resultados en {}ms (expanded: {:?})",
        query.query,
        total,
        took_ms,
        expanded_terms
    );

    Ok(AdvancedSearchResponse {
        results,
        total,
        took_ms,
        expanded_terms,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion_basic() {
        let vector = vec![(1, 0.9), (2, 0.8), (3, 0.7)];
        let graph = vec![(2, 0.6), (3, 0.5), (4, 0.4)];
        let sql = vec![(1, 0.95), (4, 0.85)];

        let result = rrf_fusion(&vector, &graph, &sql, 60);

        assert!(!result.is_empty());
        // Fragment 2 appears in vector + graph, should rank high
        let frag2 = result.iter().find(|(id, _, _)| *id == 2);
        assert!(frag2.is_some());
        assert!(frag2.unwrap().1 > 0.0);

        // Fragment 1 appears in vector + sql
        let frag1 = result.iter().find(|(id, _, _)| *id == 1);
        assert!(frag1.is_some());

        // Fragment 3 appears in vector + graph
        let frag3 = result.iter().find(|(id, _, _)| *id == 3);
        assert!(frag3.is_some());

        // Fragment 4 appears in graph + sql
        let frag4 = result.iter().find(|(id, _, _)| *id == 4);
        assert!(frag4.is_some());
    }

    #[test]
    fn test_rrf_fusion_empty() {
        let result = rrf_fusion(&[], &[], &[], 60);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rrf_fusion_single_source() {
        let vector = vec![(1, 0.9), (2, 0.8)];
        let result = rrf_fusion(&vector, &[], &[], 60);

        assert_eq!(result.len(), 2);
        // Should be sorted by score
        assert!(result[0].1 >= result[1].1);
    }

    #[test]
    fn test_rrf_fusion_ranking_order() {
        // Fragment 1 appears first in all lists
        let vector = vec![(1, 0.9), (2, 0.8)];
        let graph = vec![(1, 0.7), (3, 0.6)];
        let sql = vec![(1, 0.95), (2, 0.85)];

        let result = rrf_fusion(&vector, &graph, &sql, 60);

        // Fragment 1 should be #1 (appears in all 3)
        assert_eq!(result[0].0, 1);
    }

    #[test]
    fn test_highlight_terms_basic() {
        let content = "El motor de búsqueda vectorial usa pgvector para similaridad coseno.";
        let result = highlight_terms(content, "vectorial", 150, 3);
        assert!(result.contains("<mark>vectorial</mark>"));
    }

    #[test]
    fn test_highlight_terms_multiple() {
        let content = "GraphRAG combina búsqueda vectorial con traversales de grafo.";
        let result = highlight_terms(content, "vectorial grafo", 150, 3);
        assert!(result.contains("<mark>vectorial</mark>"));
        assert!(result.contains("<mark>grafo</mark>"));
    }

    #[test]
    fn test_highlight_terms_no_match() {
        let content = "El motor de búsqueda vectorial usa pgvector.";
        let result = highlight_terms(content, "xyz123", 150, 3);
        // Should return truncated content without marks
        assert!(!result.contains("<mark>"));
    }

    #[test]
    fn test_highlight_terms_short_content() {
        let content = "GraphRAG";
        let result = highlight_terms(content, "GraphRAG", 150, 3);
        assert!(result.contains("<mark>GraphRAG</mark>"));
    }

    #[test]
    fn test_highlight_terms_long_content() {
        let content = format!(
            "{}word{}{}",
            "b ".repeat(200),
            " test ".repeat(100),
            " end".repeat(200)
        );
        let result = highlight_terms(&content, "word", 150, 3);
        // Should truncate long content with fragments around match
        assert!(result.contains("<mark>word</mark>"));
        assert!(result.len() < content.len());
    }

    #[test]
    fn test_merge_overlapping() {
        let ranges = vec![(0, 5), (3, 8), (10, 15)];
        let merged = merge_overlapping(&ranges);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], (0, 8));
        assert_eq!(merged[1], (10, 15));
    }

    #[test]
    fn test_merge_overlapping_empty() {
        let merged = merge_overlapping(&[]);
        assert!(merged.is_empty());
    }

    #[test]
    fn test_build_search_sql_basic() {
        let query = AdvancedSearchQuery {
            query: "test".to_string(),
            ..Default::default()
        };

        let (sql, bind_values) = build_search_sql(&query);
        assert!(sql.contains("LOWER(f.contenido) LIKE $1"));
        assert_eq!(bind_values.len(), 1);
        assert_eq!(bind_values[0], "%test%");
    }

    #[test]
    fn test_build_search_sql_with_filters() {
        let query = AdvancedSearchQuery {
            query: "test".to_string(),
            filters: SearchFilters {
                doc_types: vec!["markdown".to_string(), "code".to_string()],
                areas: vec![1, 2],
                date_from: Some("2026-01-01".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let (sql, bind_values) = build_search_sql(&query);
        assert!(sql.contains("d.tipo IN ($2, $3)"));
        assert!(sql.contains("d.area_id IN ($4, $5)"));
        assert!(sql.contains("d.creado_en >= $6"));
        assert_eq!(bind_values.len(), 6);
    }

    #[test]
    fn test_build_search_sql_no_query() {
        let query = AdvancedSearchQuery {
            query: String::new(),
            ..Default::default()
        };

        let (sql, bind_values) = build_search_sql(&query);
        assert!(!sql.contains("LIKE"));
        assert!(bind_values.is_empty());
    }

    #[test]
    fn test_advanced_search_query_defaults() {
        let query = AdvancedSearchQuery::default();
        assert!(query.query.is_empty());
        assert_eq!(query.vector.limit, 10);
        assert_eq!(query.vector.weight, 1.0);
        assert_eq!(query.graph.degrees, 1);
        assert!(query.expansion.enabled);
        assert!(query.highlight.enabled);
        assert_eq!(query.limit, 20);
    }

    #[test]
    fn test_advanced_search_result_serialization() {
        let result = AdvancedSearchResult {
            fragment_id: 1,
            document_id: 2,
            path: Some("docs/test.md".to_string()),
            content: "test content".to_string(),
            highlighted: Some("test <mark>content</mark>".to_string()),
            similarity: 0.85,
            score_breakdown: ScoreBreakdown {
                vector: 0.9,
                graph: 0.3,
                rrf: 0.042,
            },
            source: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"fragment_id\":1"));
        assert!(json.contains("\"document_id\":2"));
        assert!(json.contains("mark"));
    }
}
