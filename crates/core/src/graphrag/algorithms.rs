//! Algoritmos de grafo para análisis de conocimiento
//!
//! Implementaciones de:
//! - PageRank (importancia de nodos)
//! - Betweenness Centrality (nodos puente)
//! - Label Propagation (comunidades)
//! - Dijkstra (camino más corto)

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::dijkstra;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::{HashMap, VecDeque};

use super::{DocumentNode, EdgeType};

/// Resultado de PageRank para un nodo
#[derive(Debug, Clone)]
pub struct PageRankResult {
    pub node_id: i32,
    pub score: f64,
}

/// Resultado de Betweenness Centrality para un nodo
#[derive(Debug, Clone)]
pub struct BetweennessResult {
    pub node_id: i32,
    pub score: f64,
}

/// Resultado de degree centrality
#[derive(Debug, Clone)]
pub struct DegreeResult {
    pub node_id: i32,
    pub in_degree: usize,
    pub out_degree: usize,
    pub total_degree: usize,
}

/// Comunidad detectada
#[derive(Debug, Clone)]
pub struct Community {
    pub id: usize,
    pub members: Vec<i32>,
    pub size: usize,
}

/// Resultado de shortest path
#[derive(Debug, Clone)]
pub struct ShortestPathResult {
    pub path: Vec<i32>,
    pub distance: f64,
    pub found: bool,
}

/// Resultado consolidado de todos los algoritmos
#[derive(Debug, Clone)]
pub struct GraphAnalysis {
    pub pagerank: HashMap<i32, f64>,
    pub betweenness: HashMap<i32, f64>,
    pub degree: HashMap<i32, DegreeResult>,
    pub communities: Vec<Community>,
}

// =============================================================================
// PageRank
// =============================================================================

/// PageRank iterativo (power iteration)
///
/// - damping_factor: factor de amortiguación (default 0.85)
/// - max_iterations: máximo de iteraciones (default 100)
/// - tolerance: convergencia (default 1e-6)
pub fn pagerank(
    graph: &DiGraph<DocumentNode, EdgeType>,
    damping_factor: f64,
    max_iterations: usize,
    tolerance: f64,
) -> HashMap<i32, f64> {
    let n = graph.node_count() as f64;
    if n == 0.0 {
        return HashMap::new();
    }

    let mut scores: HashMap<NodeIndex, f64> = HashMap::new();
    let mut new_scores: HashMap<NodeIndex, f64> = HashMap::new();

    // Inicializar con 1/n
    for node in graph.node_indices() {
        scores.insert(node, 1.0 / n);
    }

    for _ in 0..max_iterations {
        new_scores.clear();

        let mut dangling_sum = 0.0;
        for node in graph.node_indices() {
            if graph.edges_directed(node, Direction::Outgoing).count() == 0 {
                dangling_sum += scores[&node];
            }
        }

        for node in graph.node_indices() {
            let mut incoming_sum = 0.0;
            for edge in graph.edges_directed(node, Direction::Incoming) {
                let source = edge.source();
                let out_degree = graph.edges_directed(source, Direction::Outgoing).count() as f64;
                if out_degree > 0.0 {
                    incoming_sum += scores[&source] / out_degree;
                }
            }

            let new_score =
                (1.0 - damping_factor) / n + damping_factor * (incoming_sum + dangling_sum / n);
            new_scores.insert(node, new_score);
        }

        // Verificar convergencia
        let mut diff = 0.0;
        for node in graph.node_indices() {
            let old = scores.get(&node).copied().unwrap_or(0.0);
            let new = new_scores.get(&node).copied().unwrap_or(0.0);
            diff += (new - old).abs();
        }

        scores = new_scores.clone();

        if diff < tolerance {
            break;
        }
    }

    // Convertir a node_id -> score
    let mut result = HashMap::new();
    for (idx, score) in scores {
        if let Some(node) = graph.node_weight(idx) {
            result.insert(node.id, score);
        }
    }
    result
}

// =============================================================================
// Betweenness Centrality (Brandes simplificado)
// =============================================================================

/// Betweenness Centrality usando algoritmo de Brandes
///
/// Mide cuántos caminos más cortos pasan por cada nodo.
/// Nodos alto betweenness son "puentes" entre comunidades.
pub fn betweenness_centrality(
    graph: &DiGraph<DocumentNode, EdgeType>,
) -> HashMap<i32, f64> {
    let n = graph.node_count();
    if n == 0 {
        return HashMap::new();
    }

    let mut centrality: HashMap<NodeIndex, f64> = HashMap::new();
    for node in graph.node_indices() {
        centrality.insert(node, 0.0);
    }

    // Para cada nodo fuente, ejecutar BFS para shortest paths
    for source in graph.node_indices() {
        let (paths, sigma, _distance) = bfs_shortest_paths(graph, source);
        let delta = backward_dependency(graph, &paths, &sigma, source);

        for (node, d) in &delta {
            if let Some(c) = centrality.get_mut(node) {
                *c += d;
            }
        }
    }

    // Normalizar: entre 0 y 1
    let normalization = if n > 2 {
        (n - 1) as f64 * (n - 2) as f64
    } else {
        1.0
    };

    let mut result = HashMap::new();
    for (idx, score) in centrality {
        let normalized = score / normalization;
        if let Some(node) = graph.node_weight(idx) {
            result.insert(node.id, normalized);
        }
    }
    result
}

#[allow(clippy::type_complexity)]
fn bfs_shortest_paths(
    graph: &DiGraph<DocumentNode, EdgeType>,
    source: NodeIndex,
) -> (HashMap<NodeIndex, Vec<Vec<NodeIndex>>>, HashMap<NodeIndex, f64>, HashMap<NodeIndex, f64>) {
    let mut paths: HashMap<NodeIndex, Vec<Vec<NodeIndex>>> = HashMap::new();
    let mut sigma: HashMap<NodeIndex, f64> = HashMap::new();
    let mut distance: HashMap<NodeIndex, f64> = HashMap::new();

    let mut queue = VecDeque::new();
    let mut visited = std::collections::HashSet::new();

    paths.insert(source, vec![vec![source]]);
    sigma.insert(source, 1.0);
    distance.insert(source, 0.0);
    queue.push_back(source);
    visited.insert(source);

    while let Some(current) = queue.pop_front() {
        let current_dist = distance[&current];
        for edge in graph.edges_directed(current, Direction::Outgoing) {
            let neighbor = edge.target();
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                distance.insert(neighbor, current_dist + 1.0);
                queue.push_back(neighbor);
            }

            if distance[&neighbor] == current_dist + 1.0 {
                *sigma.entry(neighbor).or_insert(0.0) += sigma[&current];
                let mut new_paths = paths.get(&current).cloned().unwrap_or_default();
                for path in &mut new_paths {
                    path.push(neighbor);
                }
                paths.entry(neighbor).or_default().extend(new_paths);
            }
        }
    }

    (paths, sigma, distance)
}

fn backward_dependency(
    graph: &DiGraph<DocumentNode, EdgeType>,
    paths: &HashMap<NodeIndex, Vec<Vec<NodeIndex>>>,
    sigma: &HashMap<NodeIndex, f64>,
    source: NodeIndex,
) -> HashMap<NodeIndex, f64> {
    let mut delta: HashMap<NodeIndex, f64> = HashMap::new();

    // Obtener todos los nodos ordenados por distancia (de mayor a menor)
    let mut sorted_nodes: Vec<NodeIndex> = paths.keys().copied().collect();
    sorted_nodes.sort_by(|a, b| {
        let da = paths.get(a).map(|p| p.len()).unwrap_or(0);
        let db = paths.get(b).map(|p| p.len()).unwrap_or(0);
        db.cmp(&da)
    });

    for node in &sorted_nodes {
        if *node == source {
            continue;
        }

        let sigma_v = sigma.get(node).copied().unwrap_or(0.0);
        if sigma_v == 0.0 {
            continue;
        }

        let mut contribution = 0.0;
        for edge in graph.edges_directed(*node, Direction::Incoming) {
            let predecessor = edge.source();
            if let Some(sigma_w) = sigma.get(&predecessor) {
                if *sigma_w > 0.0 {
                    let num_paths = count_paths_through_node(paths, predecessor, *node);
                    contribution += num_paths / sigma_w;
                }
            }
        }

        let delta_v = delta.entry(*node).or_insert(0.0);
        *delta_v += contribution;
    }

    delta
}

fn count_paths_through_node(
    paths: &HashMap<NodeIndex, Vec<Vec<NodeIndex>>>,
    from: NodeIndex,
    to: NodeIndex,
) -> f64 {
    if let Some(path_list) = paths.get(&to) {
        path_list
            .iter()
            .filter(|path| path.contains(&from))
            .count() as f64
    } else {
        0.0
    }
}

// =============================================================================
// Degree Centrality
// =============================================================================

/// Degree centrality: in-degree, out-degree, total
pub fn degree_centrality(
    graph: &DiGraph<DocumentNode, EdgeType>,
) -> HashMap<i32, DegreeResult> {
    let mut result = HashMap::new();

    for node in graph.node_indices() {
        let in_degree = graph.edges_directed(node, Direction::Incoming).count();
        let out_degree = graph.edges_directed(node, Direction::Outgoing).count();

        if let Some(doc) = graph.node_weight(node) {
            result.insert(
                doc.id,
                DegreeResult {
                    node_id: doc.id,
                    in_degree,
                    out_degree,
                    total_degree: in_degree + out_degree,
                },
            );
        }
    }

    result
}

// =============================================================================
// Label Propagation (Comunidades)
// =============================================================================

/// Detección de comunidades usando Label Propagation
///
/// Algoritmo iterativo simple y eficiente:
/// 1. Cada nodo comienza con su propio label
/// 2. En cada iteración, cada nodo adopta el label más frecuente entre sus vecinos
/// 3. Repetir hasta convergencia o max_iterations
///
/// Complejidad: O(E) por iteración, típicamente 5-10 iteraciones
pub fn label_propagation(
    graph: &DiGraph<DocumentNode, EdgeType>,
    max_iterations: usize,
) -> Vec<Community> {
    let n = graph.node_count();
    if n == 0 {
        return Vec::new();
    }

    // Inicializar: cada nodo tiene su propio label
    let mut labels: HashMap<NodeIndex, usize> = HashMap::new();
    let node_ids: Vec<NodeIndex> = graph.node_indices().collect();
    for (i, &node) in node_ids.iter().enumerate() {
        labels.insert(node, i);
    }

    // Iterar
    for _ in 0..max_iterations {
        let mut changed = false;

        // Shuffle determinístico (por reproducibilidad)
        for &node in &node_ids {
            let neighbors: Vec<NodeIndex> = graph
                .neighbors_undirected(node)
                .collect();

            if neighbors.is_empty() {
                continue;
            }

            // Contar frecuencia de labels entre vecinos
            let mut label_counts: HashMap<usize, usize> = HashMap::new();
            for &neighbor in &neighbors {
                if let Some(&label) = labels.get(&neighbor) {
                    *label_counts.entry(label).or_insert(0) += 1;
                }
            }

            // Elegir label más frecuente
            if let Some((&best_label, _)) = label_counts.iter().max_by_key(|(_, &count)| count) {
                if labels.get(&node) != Some(&best_label) {
                    labels.insert(node, best_label);
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    // Agrupar nodos por label
    let mut communities_map: HashMap<usize, Vec<i32>> = HashMap::new();
    for (node, label) in &labels {
        if let Some(doc) = graph.node_weight(*node) {
            communities_map
                .entry(*label)
                .or_default()
                .push(doc.id);
        }
    }

    // Convertir a Community structs
    communities_map
        .into_iter()
        .enumerate()
        .map(|(id, (_label, members))| {
            let size = members.len();
            Community { id, members, size }
        })
        .collect()
}

// =============================================================================
// Shortest Path (Dijkstra)
// =============================================================================

/// Camino más corto entre dos nodos usando Dijkstra
///
/// El peso de cada arista se determina por tipo:
/// - wiki_link: 1.0 (fuerte)
/// - backlink: 1.5
/// - reference: 2.0 (débil)
///
/// Retorna la lista de node_ids en el camino y la distancia total
pub fn shortest_path(
    graph: &DiGraph<DocumentNode, EdgeType>,
    source_id: i32,
    target_id: i32,
) -> ShortestPathResult {
    let node_map = build_node_id_map(graph);

    let &source = match node_map.get(&source_id) {
        Some(idx) => idx,
        None => {
            return ShortestPathResult {
                path: vec![],
                distance: f64::INFINITY,
                found: false,
            };
        }
    };

    let &target = match node_map.get(&target_id) {
        Some(idx) => idx,
        None => {
            return ShortestPathResult {
                path: vec![],
                distance: f64::INFINITY,
                found: false,
            };
        }
    };

    if source == target {
        return ShortestPathResult {
            path: vec![source_id],
            distance: 0.0,
            found: true,
        };
    }

    // Usar Dijkstra de petgraph con pesos de aristas
    let distances_hb = dijkstra(graph, source, Some(target), |edge| {
        edge_weight(edge.weight())
    });
    let distances: HashMap<NodeIndex, f64> = distances_hb.into_iter().collect();

    match distances.get(&target) {
        Some(&dist) => {
            // Reconstruir camino
            let path = reconstruct_path(graph, &distances, source, target);
            ShortestPathResult {
                path,
                distance: dist,
                found: true,
            }
        }
        None => ShortestPathResult {
            path: vec![],
            distance: f64::INFINITY,
            found: false,
        },
    }
}

/// Camino más corto desde un nodo a todos los demás
pub fn shortest_paths_from(
    graph: &DiGraph<DocumentNode, EdgeType>,
    source_id: i32,
) -> HashMap<i32, ShortestPathResult> {
    let node_map = build_node_id_map(graph);

    let &source = match node_map.get(&source_id) {
        Some(idx) => idx,
        None => return HashMap::new(),
    };

    let distances_hb = dijkstra(graph, source, None, |edge| edge_weight(edge.weight()));
    let distances: HashMap<NodeIndex, f64> = distances_hb.into_iter().collect();

    let mut result = HashMap::new();
    for (target_idx, &dist) in &distances {
        if let Some(target_node) = graph.node_weight(*target_idx) {
            let path = if dist == 0.0 {
                vec![source_id]
            } else {
                reconstruct_path(graph, &distances, source, *target_idx)
            };

            result.insert(
                target_node.id,
                ShortestPathResult {
                    path,
                    distance: dist,
                    found: true,
                },
            );
        }
    }

    result
}

// =============================================================================
// Helpers
// =============================================================================

/// Peso de arista según tipo
fn edge_weight(edge_type: &EdgeType) -> f64 {
    match edge_type {
        EdgeType::WikiLink { .. } => 1.0,
        EdgeType::Backlink { .. } => 1.5,
        EdgeType::Reference { .. } => 2.0,
    }
}

/// Construir mapa de node_id -> NodeIndex
fn build_node_id_map(graph: &DiGraph<DocumentNode, EdgeType>) -> HashMap<i32, NodeIndex> {
    let mut map = HashMap::new();
    for node in graph.node_indices() {
        if let Some(doc) = graph.node_weight(node) {
            map.insert(doc.id, node);
        }
    }
    map
}

/// Reconstruir camino desde Dijkstra (backtracking por distancias)
fn reconstruct_path(
    graph: &DiGraph<DocumentNode, EdgeType>,
    distances: &std::collections::HashMap<NodeIndex, f64>,
    source: NodeIndex,
    target: NodeIndex,
) -> Vec<i32> {
    let mut path = Vec::new();
    let mut current = target;

    while current != source {
        if let Some(doc) = graph.node_weight(current) {
            path.push(doc.id);
        }
        let current_dist = distances.get(&current).copied().unwrap_or(f64::INFINITY);

        // Find predecessor: an incoming neighbor whose distance + edge_weight == current_dist
        let mut found = false;
        for edge in graph.edges_directed(current, Direction::Incoming) {
            let predecessor = edge.source();
            let pred_dist = distances.get(&predecessor).copied().unwrap_or(f64::INFINITY);
            let w = edge_weight(edge.weight());
            if (pred_dist + w - current_dist).abs() < 1e-10 {
                current = predecessor;
                found = true;
                break;
            }
        }

        if !found {
            // Fallback: no valid predecessor found, path is broken
            break;
        }
    }

    // Add source
    if let Some(doc) = graph.node_weight(source) {
        path.push(doc.id);
    }

    path.reverse();
    path
}

/// Analizar grafo completo (todos los algoritmos)
pub fn analyze_graph(
    graph: &DiGraph<DocumentNode, EdgeType>,
) -> GraphAnalysis {
    let pagerank = pagerank(graph, 0.85, 100, 1e-6);
    let betweenness = betweenness_centrality(graph);
    let degree = degree_centrality(graph);
    let communities = label_propagation(graph, 10);

    GraphAnalysis {
        pagerank,
        betweenness,
        degree,
        communities,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> DiGraph<DocumentNode, EdgeType> {
        let mut graph = DiGraph::new();

        let n1 = graph.add_node(DocumentNode {
            id: 1,
            path: "doc1.md".to_string(),
            doc_type: "markdown".to_string(),
        });
        let n2 = graph.add_node(DocumentNode {
            id: 2,
            path: "doc2.md".to_string(),
            doc_type: "markdown".to_string(),
        });
        let n3 = graph.add_node(DocumentNode {
            id: 3,
            path: "doc3.md".to_string(),
            doc_type: "markdown".to_string(),
        });
        let n4 = graph.add_node(DocumentNode {
            id: 4,
            path: "doc4.md".to_string(),
            doc_type: "code".to_string(),
        });

        // n1 -> n2 -> n3 (lineal)
        graph.add_edge(n1, n2, EdgeType::WikiLink { context: "link1".to_string() });
        graph.add_edge(n2, n3, EdgeType::Backlink { context: "link2".to_string() });

        // n3 -> n4 (branch)
        graph.add_edge(n3, n4, EdgeType::Reference { context: "link3".to_string() });

        // n4 -> n2 (cycle)
        graph.add_edge(n4, n2, EdgeType::WikiLink { context: "link4".to_string() });

        graph
    }

    #[test]
    fn test_pagerank_basic() {
        let graph = create_test_graph();
        let result = pagerank(&graph, 0.85, 100, 1e-6);

        assert_eq!(result.len(), 4);

        // Todos los scores deben ser positivos
        for score in result.values() {
            assert!(*score > 0.0);
        }

        // Suma debe ser ~1.0 (normalizado)
        let sum: f64 = result.values().sum();
        assert!((sum - 1.0).abs() < 0.01, "PageRank sum should be ~1.0, got {}", sum);

        // n2 tiene mayorPageRank (recibe de n1, n4)
        let score_2 = result.get(&2).unwrap();
        let score_1 = result.get(&1).unwrap();
        assert!(score_2 > score_1, "Node 2 should have higher PageRank than Node 1");
    }

    #[test]
    fn test_betweenness_centrality() {
        let graph = create_test_graph();
        let result = betweenness_centrality(&graph);

        assert_eq!(result.len(), 4);

        // n2 y n3 deberían tener mayor betweenness (son puentes)
        let score_2 = result.get(&2).unwrap();
        let score_3 = result.get(&3).unwrap();
        assert!(score_2 > &0.0 || score_3 > &0.0, "At least node 2 or 3 should have non-zero betweenness");
    }

    #[test]
    fn test_degree_centrality() {
        let graph = create_test_graph();
        let result = degree_centrality(&graph);

        assert_eq!(result.len(), 4);

        let n1 = result.get(&1).unwrap();
        assert_eq!(n1.in_degree, 0);
        assert_eq!(n1.out_degree, 1);

        let n2 = result.get(&2).unwrap();
        assert_eq!(n2.in_degree, 2); // n1 -> n2, n4 -> n2
        assert_eq!(n2.out_degree, 1); // n2 -> n3
    }

    #[test]
    fn test_label_propagation() {
        let graph = create_test_graph();
        let communities = label_propagation(&graph, 10);

        assert!(!communities.is_empty());

        // Todos los nodos deben estar en alguna comunidad
        let total_members: usize = communities.iter().map(|c| c.size).sum();
        assert_eq!(total_members, 4);
    }

    #[test]
    fn test_shortest_path_found() {
        let graph = create_test_graph();
        let result = shortest_path(&graph, 1, 3);

        assert!(result.found);
        assert_eq!(result.path.first(), Some(&1));
        assert_eq!(result.path.last(), Some(&3));
        assert!(result.distance > 0.0);
    }

    #[test]
    fn test_shortest_path_same_node() {
        let graph = create_test_graph();
        let result = shortest_path(&graph, 1, 1);

        assert!(result.found);
        assert_eq!(result.path, vec![1]);
        assert_eq!(result.distance, 0.0);
    }

    #[test]
    fn test_shortest_path_not_found() {
        let graph = create_test_graph();
        let result = shortest_path(&graph, 1, 999);

        assert!(!result.found);
    }

    #[test]
    fn test_shortest_path_no_outgoing() {
        let mut graph = DiGraph::new();
        let _n1 = graph.add_node(DocumentNode {
            id: 1,
            path: "isolated.md".to_string(),
            doc_type: "markdown".to_string(),
        });
        let _n2 = graph.add_node(DocumentNode {
            id: 2,
            path: "other.md".to_string(),
            doc_type: "markdown".to_string(),
        });
        // No hay edges

        let result = shortest_path(&graph, 1, 2);
        assert!(!result.found);
    }

    #[test]
    fn test_analyze_graph() {
        let graph = create_test_graph();
        let analysis = analyze_graph(&graph);

        assert_eq!(analysis.pagerank.len(), 4);
        assert_eq!(analysis.betweenness.len(), 4);
        assert_eq!(analysis.degree.len(), 4);
        assert!(!analysis.communities.is_empty());
    }

    #[test]
    fn test_empty_graph() {
        let graph: DiGraph<DocumentNode, EdgeType> = DiGraph::new();

        let pr = pagerank(&graph, 0.85, 100, 1e-6);
        assert!(pr.is_empty());

        let bc = betweenness_centrality(&graph);
        assert!(bc.is_empty());

        let dc = degree_centrality(&graph);
        assert!(dc.is_empty());

        let comm = label_propagation(&graph, 10);
        assert!(comm.is_empty());

        let sp = shortest_path(&graph, 1, 2);
        assert!(!sp.found);
    }

    #[test]
    fn test_pagerank_converges() {
        let graph = create_test_graph();
        let pr1 = pagerank(&graph, 0.85, 10, 1e-6);
        let pr2 = pagerank(&graph, 0.85, 100, 1e-6);

        // After 100 iterations vs 10, scores should be similar (converged)
        for (node_id, score1) in &pr1 {
            let score2 = pr2.get(node_id).unwrap();
            assert!((score1 - score2).abs() < 0.05, "PageRank should converge for node {}: {} vs {}", node_id, score1, score2);
        }
    }

    #[test]
    fn test_shortest_paths_from() {
        let graph = create_test_graph();
        let results = shortest_paths_from(&graph, 1);

        // Node 1 is source, should reach all reachable nodes
        assert!(results.contains_key(&2));
        assert!(results.contains_key(&3));
        assert!(results.contains_key(&4));

        // Distance to node 1 itself should be 0
        let self_result = results.get(&1).unwrap();
        assert_eq!(self_result.distance, 0.0);
    }
}
