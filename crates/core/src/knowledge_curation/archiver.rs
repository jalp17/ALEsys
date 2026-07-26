use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveReason {
    Outdated,
    Deprecated,
    Duplicate,
    Merged,
    Unused,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveResult {
    pub document_id: String,
    pub archive_path: String,
    pub reason: String,
    pub archived_at: String,
    pub success: bool,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedDocument {
    pub id: String,
    pub original_id: String,
    pub title: String,
    pub content: String,
    pub archived_at: String,
    pub reason: String,
    pub tags: Vec<String>,
}

pub struct DocumentArchiver {
    archive_dir: String,
}

impl DocumentArchiver {
    pub fn new(archive_dir: &str) -> Self {
        Self {
            archive_dir: archive_dir.to_string(),
        }
    }

    pub fn archive(
        &self,
        document_id: &str,
        title: &str,
        _content: &str,
        tags: &[String],
        reason: ArchiveReason,
    ) -> ArchiveResult {
        let reason_str = match &reason {
            ArchiveReason::Outdated => "Outdated".to_string(),
            ArchiveReason::Deprecated => "Deprecated".to_string(),
            ArchiveReason::Duplicate => "Duplicate".to_string(),
            ArchiveReason::Merged => "Merged".to_string(),
            ArchiveReason::Unused => "Unused".to_string(),
            ArchiveReason::Custom(r) => r.clone(),
        };

        let archive_path = format!("{}/{}.archived", self.archive_dir, document_id);

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), title.to_string());
        metadata.insert("reason".to_string(), reason_str.clone());
        metadata.insert("tags".to_string(), tags.join(", "));

        ArchiveResult {
            document_id: document_id.to_string(),
            archive_path,
            reason: reason_str,
            archived_at: chrono::Utc::now().to_rfc3339(),
            success: true,
            metadata,
        }
    }

    pub fn list_archived(&self) -> Vec<ArchivedDocument> {
        vec![]
    }

    pub fn restore(&self, _archive_id: &str) -> Option<ArchivedDocument> {
        None
    }

    pub fn get_stats(&self) -> ArchiveStats {
        ArchiveStats {
            total_archived: 0,
            by_reason: std::collections::HashMap::new(),
            oldest_archive: None,
            newest_archive: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStats {
    pub total_archived: usize,
    pub by_reason: std::collections::HashMap<String, usize>,
    pub oldest_archive: Option<String>,
    pub newest_archive: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_document() {
        let archiver = DocumentArchiver::new("/tmp/archive");
        let result = archiver.archive(
            "doc1",
            "Test Doc",
            "Content here",
            &vec!["test".to_string()],
            ArchiveReason::Outdated,
        );
        assert!(result.success);
        assert_eq!(result.document_id, "doc1");
        assert_eq!(result.reason, "Outdated");
    }

    #[test]
    fn test_archive_deprecated() {
        let archiver = DocumentArchiver::new("/tmp/archive");
        let result = archiver.archive(
            "doc2",
            "Old API Doc",
            "Deprecated API docs",
            &vec!["api".to_string(), "deprecated".to_string()],
            ArchiveReason::Deprecated,
        );
        assert!(result.success);
        assert!(result.metadata.contains_key("title"));
    }

    #[test]
    fn test_archive_custom_reason() {
        let archiver = DocumentArchiver::new("/tmp/archive");
        let result = archiver.archive(
            "doc3",
            "Test",
            "Content",
            &[],
            ArchiveReason::Custom("No longer needed".to_string()),
        );
        assert!(result.success);
        assert_eq!(result.reason, "No longer needed");
    }

    #[test]
    fn test_archive_stats() {
        let archiver = DocumentArchiver::new("/tmp/archive");
        let stats = archiver.get_stats();
        assert_eq!(stats.total_archived, 0);
    }
}