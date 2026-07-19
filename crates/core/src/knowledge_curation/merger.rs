use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    Concatenate,
    Interleave,
    Smart,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub merged_content: String,
    pub sources_count: usize,
    pub conflicts: Vec<MergeConflict>,
    pub success: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    pub field: String,
    pub source_a: String,
    pub source_b: String,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct DocumentMerger {
    strategy: MergeStrategy,
}

impl DocumentMerger {
    pub fn new(strategy: MergeStrategy) -> Self {
        Self { strategy }
    }

    pub fn merge(&self, documents: &[Document]) -> MergeResult {
        if documents.is_empty() {
            return MergeResult {
                merged_content: String::new(),
                sources_count: 0,
                conflicts: vec![],
                success: false,
                warnings: vec!["No documents to merge".to_string()],
            };
        }

        if documents.len() == 1 {
            return MergeResult {
                merged_content: documents[0].content.clone(),
                sources_count: 1,
                conflicts: vec![],
                success: true,
                warnings: vec![],
            };
        }

        let merged = match self.strategy {
            MergeStrategy::Concatenate => self.concatenate(documents),
            MergeStrategy::Interleave => self.interleave(documents),
            MergeStrategy::Smart => self.smart_merge(documents),
            MergeStrategy::Manual => self.manual_merge(documents),
        };

        merged
    }

    fn concatenate(&self, documents: &[Document]) -> MergeResult {
        let mut content = String::new();
        for (i, doc) in documents.iter().enumerate() {
            if i > 0 {
                content.push_str("\n\n---\n\n");
            }
            content.push_str(&doc.content);
        }

        MergeResult {
            merged_content: content,
            sources_count: documents.len(),
            conflicts: vec![],
            success: true,
            warnings: vec![],
        }
    }

    fn interleave(&self, documents: &[Document]) -> MergeResult {
        let mut content = String::new();
        let max_lines = documents.iter().map(|d| d.content.lines().count()).max().unwrap_or(0);

        for line_idx in 0..max_lines {
            for (i, doc) in documents.iter().enumerate() {
                if let Some(line) = doc.content.lines().nth(line_idx) {
                    if i > 0 {
                        content.push_str(" | ");
                    }
                    content.push_str(line);
                }
            }
            content.push('\n');
        }

        MergeResult {
            merged_content: content,
            sources_count: documents.len(),
            conflicts: vec![],
            success: true,
            warnings: vec!["Interleaved merge may disrupt code structure".to_string()],
        }
    }

    fn smart_merge(&self, documents: &[Document]) -> MergeResult {
        let mut merged_content = String::new();
        let mut conflicts = Vec::new();
        let mut warnings = Vec::new();

        let mut all_tags: Vec<String> = documents.iter().flat_map(|d| d.tags.clone()).collect();
        all_tags.sort();
        all_tags.dedup();

        merged_content.push_str("---\ntitle: Merged Document\ntags:\n");
        for tag in &all_tags {
            merged_content.push_str(&format!("  - {}\n", tag));
        }
        merged_content.push_str("---\n\n");

        for (i, doc) in documents.iter().enumerate() {
            merged_content.push_str(&format!("## Source: {}\n\n", doc.title));
            merged_content.push_str(&doc.content);
            merged_content.push_str("\n\n");
        }

        MergeResult {
            merged_content,
            sources_count: documents.len(),
            conflicts,
            success: true,
            warnings,
        }
    }

    fn manual_merge(&self, documents: &[Document]) -> MergeResult {
        MergeResult {
            merged_content: String::new(),
            sources_count: documents.len(),
            conflicts: vec![],
            success: false,
            warnings: vec!["Manual merge requires user intervention".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(id: &str, title: &str, content: &str) -> Document {
        Document {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            tags: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_merge_empty() {
        let merger = DocumentMerger::new(MergeStrategy::Concatenate);
        let result = merger.merge(&[]);
        assert!(!result.success);
        assert_eq!(result.sources_count, 0);
    }

    #[test]
    fn test_merge_single() {
        let merger = DocumentMerger::new(MergeStrategy::Concatenate);
        let docs = vec![make_doc("1", "A", "content")];
        let result = merger.merge(&docs);
        assert!(result.success);
        assert_eq!(result.merged_content, "content");
    }

    #[test]
    fn test_concatenate() {
        let merger = DocumentMerger::new(MergeStrategy::Concatenate);
        let docs = vec![
            make_doc("1", "A", "Hello"),
            make_doc("2", "B", "World"),
        ];
        let result = merger.merge(&docs);
        assert!(result.success);
        assert!(result.merged_content.contains("Hello"));
        assert!(result.merged_content.contains("World"));
        assert_eq!(result.sources_count, 2);
    }

    #[test]
    fn test_smart_merge_tags() {
        let merger = DocumentMerger::new(MergeStrategy::Smart);
        let mut doc1 = make_doc("1", "A", "Content A");
        doc1.tags = vec!["rust".to_string(), "code".to_string()];
        let mut doc2 = make_doc("2", "B", "Content B");
        doc2.tags = vec!["python".to_string(), "code".to_string()];
        let result = merger.merge(&[doc1, doc2]);
        assert!(result.success);
        assert!(result.merged_content.contains("rust"));
        assert!(result.merged_content.contains("python"));
        assert!(result.merged_content.contains("code"));
    }

    #[test]
    fn test_manual_merge() {
        let merger = DocumentMerger::new(MergeStrategy::Manual);
        let docs = vec![make_doc("1", "A", "content"), make_doc("2", "B", "other")];
        let result = merger.merge(&docs);
        assert!(!result.success);
    }
}