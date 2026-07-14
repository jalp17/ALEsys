//! Tests para el módulo llm

use alesys_core::llm::{ONNXEmbedder, LLMConfig, LLMBackendType};

#[test]
fn test_llm_config_default() {
    let config = LLMConfig::default();
    assert_eq!(config.max_tokens, 2048);
    assert_eq!(config.temperature, 0.7);
    assert_eq!(config.top_p, 0.9);
    assert_eq!(config.context_size, 4096);
    assert!(config.model_path.is_empty());
}

#[test]
fn test_llm_backend_type_from_str() {
    let backend: LLMBackendType = "llama_cpp".parse().unwrap();
    assert_eq!(backend, LLMBackendType::LlamaCpp);
    
    let backend: LLMBackendType = "mistralrs".parse().unwrap();
    assert_eq!(backend, LLMBackendType::Mistralrs);
    
    let backend: LLMBackendType = "llama.cpp".parse().unwrap();
    assert_eq!(backend, LLMBackendType::LlamaCpp);
    
    let backend: LLMBackendType = "mistral".parse().unwrap();
    assert_eq!(backend, LLMBackendType::Mistralrs);
}

#[test]
fn test_llm_backend_type_invalid() {
    let result = "invalid".parse::<LLMBackendType>();
    assert!(result.is_err());
}

#[test]
fn test_onnx_embedder_creation() {
    let embedder = ONNXEmbedder::new();
    // Model not loaded, returns dummy embedding
    let result = embedder.encode("test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 384);
}

#[test]
fn test_onnx_embedder_default() {
    let embedder = ONNXEmbedder::default();
    let result = embedder.encode("test");
    assert!(result.is_ok());
    // Dummy embeddings should be normalized
    let embedding = result.unwrap();
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.01 || norm < 0.001);
}

#[test]
fn test_onnx_embedder_batch() {
    let embedder = ONNXEmbedder::new();
    let texts = vec!["hello", "world"];
    let results = embedder.encode_batch(&texts);
    assert!(results.is_ok());
    let embeddings = results.unwrap();
    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0].len(), 384);
}

#[test]
fn test_knowledge_extraction_serialization() {
    use alesys_core::llm::{KnowledgeExtraction, Entity, Relation};

    let extraction = KnowledgeExtraction {
        entities: vec![
            Entity {
                name: "Faraday".to_string(),
                entity_type: "científico".to_string(),
            },
        ],
        relations: vec![
            Relation {
                origin: "Faraday".to_string(),
                destination: "inducción electromagnética".to_string(),
                relation_type: "descubrió".to_string(),
            },
        ],
    };

    let json = serde_json::to_string(&extraction).unwrap();
    assert!(json.contains("Faraday"));
    assert!(json.contains("inducción electromagnética"));

    let deserialized: KnowledgeExtraction = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.entities.len(), 1);
    assert_eq!(deserialized.relations.len(), 1);
}
