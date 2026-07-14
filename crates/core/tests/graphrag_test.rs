//! Tests para el módulo graphrag

use alesys_core::graphrag::{build_rag_context, SearchResult, SearchResultSource};

#[test]
fn test_build_rag_context_empty() {
    let results = vec![];
    let context = build_rag_context(&results, 4096);
    assert!(context.contains("GraphRAG"));
}

#[test]
fn test_build_rag_context_with_results() {
    let results = vec![
        SearchResult {
            fragment_id: 1,
            document_id: 1,
            content: "Contenido de prueba 1".to_string(),
            similarity: 0.95,
            source: SearchResultSource::Vector,
            doc_path: Some("test/doc1.md".to_string()),
        },
        SearchResult {
            fragment_id: 2,
            document_id: 2,
            content: "Contenido de prueba 2".to_string(),
            similarity: 0.80,
            source: SearchResultSource::Graph,
            doc_path: Some("test/doc2.md".to_string()),
        },
    ];

    let context = build_rag_context(&results, 4096);

    assert!(context.contains("Fragmento 1"));
    assert!(context.contains("Fragmento 2"));
    assert!(context.contains("búsqueda semántica"));
    assert!(context.contains("relación en grafo"));
    assert!(context.contains("test/doc1.md"));
    assert!(context.contains("test/doc2.md"));
}

#[test]
fn test_build_rag_context_token_limit() {
    let results: Vec<SearchResult> = (0..100)
        .map(|i| SearchResult {
            fragment_id: i,
            document_id: i,
            content: "x".repeat(1000),
            similarity: 0.5,
            source: SearchResultSource::Vector,
            doc_path: None,
        })
        .collect();

    let context = build_rag_context(&results, 500);
    assert!(context.contains("truncados"));
}

#[test]
fn test_search_result_source_display() {
    let v = SearchResultSource::Vector;
    let g = SearchResultSource::Graph;

    assert_eq!(format!("{:?}", v), "Vector");
    assert_eq!(format!("{:?}", g), "Graph");
    assert_ne!(v, g);
}