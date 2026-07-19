use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    pub id: String,
    pub key: String,
    pub value: String,
    pub entry_type: ContextType,
    pub timestamp: i64,
    pub relevance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextType {
    FilePattern,
    Language,
    ProjectStructure,
    UserPreference,
    SessionHistory,
}

pub struct ContextualMemory {
    entries: Vec<ContextEntry>,
    patterns: HashMap<String, Vec<String>>,
}

impl ContextualMemory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            patterns: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: String, value: String, entry_type: ContextType) -> ContextEntry {
        let entry = ContextEntry {
            id: uuid::Uuid::new_v4().to_string(),
            key: key.clone(),
            value: value.clone(),
            entry_type,
            timestamp: chrono::Utc::now().timestamp(),
            relevance: 1.0,
        };
        self.entries.push(entry.clone());
        self.patterns
            .entry(key)
            .or_default()
            .push(value);
        entry
    }

    pub fn query(&self, key: &str) -> Vec<&ContextEntry> {
        self.entries
            .iter()
            .filter(|e| e.key == key)
            .collect()
    }

    pub fn query_by_type(&self, entry_type: &ContextType) -> Vec<&ContextEntry> {
        self.entries
            .iter()
            .filter(|e| e.entry_type == *entry_type)
            .collect()
    }

    pub fn get_recent(&self, count: usize) -> Vec<&ContextEntry> {
        let mut sorted = self.entries.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted.into_iter().take(count).collect()
    }

    pub fn get_patterns(&self, key: &str) -> Vec<&String> {
        self.patterns.get(key).map(|v| v.iter().collect()).unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ContextualMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_query() {
        let mut memory = ContextualMemory::new();
        memory.store(
            "language".to_string(),
            "rust".to_string(),
            ContextType::Language,
        );
        memory.store(
            "language".to_string(),
            "typescript".to_string(),
            ContextType::Language,
        );
        assert_eq!(memory.len(), 2);
        assert_eq!(memory.query("language").len(), 2);
    }

    #[test]
    fn test_query_by_type() {
        let mut memory = ContextualMemory::new();
        memory.store(
            "lang".to_string(),
            "rust".to_string(),
            ContextType::Language,
        );
        memory.store(
            "pattern".to_string(),
            "*.rs".to_string(),
            ContextType::FilePattern,
        );
        assert_eq!(memory.query_by_type(&ContextType::Language).len(), 1);
        assert_eq!(memory.query_by_type(&ContextType::FilePattern).len(), 1);
    }

    #[test]
    fn test_get_recent() {
        let mut memory = ContextualMemory::new();
        for i in 0..5 {
            memory.store(
                format!("key-{}", i),
                format!("val-{}", i),
                ContextType::SessionHistory,
            );
        }
        let recent = memory.get_recent(3);
        assert_eq!(recent.len(), 3);
    }
}
