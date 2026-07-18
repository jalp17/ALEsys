//! File editor with diff generation and backup support.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Backup failed: {0}")]
    BackupFailed(String),

    #[error("Diff generation failed: {0}")]
    DiffFailed(String),
}

impl Serialize for EditorError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Result of a file operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
    /// Path of the file affected
    pub path: String,
}

/// Result of a diff operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// Unified diff output
    pub diff: String,
    /// Lines added
    pub lines_added: usize,
    /// Lines removed
    pub lines_removed: usize,
    /// Old content (for preview)
    pub old_content: String,
    /// New content (for preview)
    pub new_content: String,
}

/// File tree entry for directory listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeEntry {
    /// File/directory name
    pub name: String,
    /// Relative path from root
    pub path: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// Children (only for directories)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileTreeEntry>>,
}

/// Main file editor interface.
pub struct FileEditor {
    /// Root directory for file operations
    root_dir: PathBuf,
    /// Backup directory
    backup_dir: PathBuf,
}

impl FileEditor {
    /// Create a new FileEditor with the given root directory.
    pub fn new(root_dir: PathBuf) -> Self {
        let backup_dir = root_dir.join(".alesys_backups");
        Self { root_dir, backup_dir }
    }

    /// Read file contents.
    pub fn read_file(&self, path: &str) -> Result<String, EditorError> {
        let full_path = self.resolve_path(path)?;
        if !full_path.exists() {
            return Err(EditorError::NotFound(path.to_string()));
        }
        Ok(std::fs::read_to_string(&full_path)?)
    }

    /// Write content to a file.
    pub fn write_file(&self, path: &str, content: &str) -> Result<FileOperationResult, EditorError> {
        let full_path = self.resolve_path(path)?;

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, content)?;

        Ok(FileOperationResult {
            success: true,
            message: Some(format!("Written {} bytes", content.len())),
            path: path.to_string(),
        })
    }

    /// Modify file with backup (old_content -> new_content).
    pub fn modify_file(
        &self,
        path: &str,
        old_content: &str,
        new_content: &str,
    ) -> Result<DiffResult, EditorError> {
        let full_path = self.resolve_path(path)?;

        // Verify current content matches old_content
        if full_path.exists() {
            let current = std::fs::read_to_string(&full_path)?;
            if current != old_content {
                return Err(EditorError::DiffFailed(
                    "Current file content does not match expected content".to_string(),
                ));
            }
        }

        // Create backup
        if full_path.exists() {
            self.create_backup(path)?;
        }

        // Write new content
        std::fs::write(&full_path, new_content)?;

        // Generate diff
        let diff = self.generate_diff(old_content, new_content);

        Ok(diff)
    }

    /// List directory contents.
    pub fn list_files(&self, path: &str) -> Result<Vec<FileTreeEntry>, EditorError> {
        let full_path = if path.is_empty() {
            self.root_dir.clone()
        } else {
            self.resolve_path(path)?
        };

        if !full_path.exists() {
            return Err(EditorError::NotFound(path.to_string()));
        }

        self.list_dir_recursive(&full_path, path)
    }

    /// Generate unified diff between two strings.
    fn generate_diff(&self, old: &str, new: &str) -> DiffResult {
        use similar::{ChangeTag, TextDiff};

        let diff = TextDiff::from_lines(old, new);
        let mut diff_str = String::new();
        let mut lines_added = 0;
        let mut lines_removed = 0;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Insert => {
                    lines_added += 1;
                    diff_str.push('+');
                    diff_str.push_str(change.as_str().unwrap_or(""));
                }
                ChangeTag::Delete => {
                    lines_removed += 1;
                    diff_str.push('-');
                    diff_str.push_str(change.as_str().unwrap_or(""));
                }
                ChangeTag::Equal => {
                    diff_str.push(' ');
                    diff_str.push_str(change.as_str().unwrap_or(""));
                }
            }
        }

        DiffResult {
            diff: diff_str,
            lines_added,
            lines_removed,
            old_content: old.to_string(),
            new_content: new.to_string(),
        }
    }

    /// Create a backup of a file.
    fn create_backup(&self, path: &str) -> Result<(), EditorError> {
        let full_path = self.resolve_path(path)?;
        let backup_name = format!(
            "{}_{}",
            path.replace('/', "_"),
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let backup_path = self.backup_dir.join(&backup_name);

        std::fs::create_dir_all(&self.backup_dir)
            .map_err(|e| EditorError::BackupFailed(e.to_string()))?;

        std::fs::copy(&full_path, &backup_path)
            .map_err(|e| EditorError::BackupFailed(e.to_string()))?;

        Ok(())
    }

    /// Resolve a relative path to an absolute path within root_dir.
    fn resolve_path(&self, path: &str) -> Result<PathBuf, EditorError> {
        let full_path = self.root_dir.join(path);

        // Security: ensure we don't escape root_dir
        let canonical = full_path.canonicalize().unwrap_or_else(|_| full_path.clone());
        let root_canonical = self.root_dir.canonicalize().unwrap_or_else(|_| self.root_dir.clone());

        if !canonical.starts_with(&root_canonical) {
            return Err(EditorError::NotFound(
                "Path escapes root directory".to_string(),
            ));
        }

        Ok(full_path)
    }

    /// Recursively list directory contents.
    fn list_dir_recursive(
        &self,
        dir: &Path,
        relative_path: &str,
    ) -> Result<Vec<FileTreeEntry>, EditorError> {
        let mut entries = Vec::new();

        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                let name = entry.file_name().to_string_lossy().to_string();
                let path = if relative_path.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", relative_path, name)
                };

                let is_dir = metadata.is_dir();
                let children = if is_dir {
                    Some(self.list_dir_recursive(&entry.path(), &path)?)
                } else {
                    None
                };

                entries.push(FileTreeEntry {
                    name,
                    path,
                    is_dir,
                    size: metadata.len(),
                    children,
                });
            }

            // Sort: directories first, then alphabetically
            entries.sort_by(|a, b| {
                if a.is_dir == b.is_dir {
                    a.name.cmp(&b.name)
                } else if a.is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });
        }

        Ok(entries)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("alesys_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup_test_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_read_write_file() {
        let dir = setup_test_dir();
        let editor = FileEditor::new(dir.clone());

        editor.write_file("test.txt", "Hello, World!").unwrap();
        let content = editor.read_file("test.txt").unwrap();
        assert_eq!(content, "Hello, World!");

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_modify_file_with_diff() {
        let dir = setup_test_dir();
        let editor = FileEditor::new(dir.clone());

        editor.write_file("test.txt", "Line 1\nLine 2\n").unwrap();
        let result = editor
            .modify_file("test.txt", "Line 1\nLine 2\n", "Line 1\nLine 3\n")
            .unwrap();

        assert!(result.lines_added > 0 || result.lines_removed > 0);
        assert!(!result.diff.is_empty());

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_list_files() {
        let dir = setup_test_dir();
        let editor = FileEditor::new(dir.clone());

        editor.write_file("a.txt", "a").unwrap();
        editor.write_file("b.txt", "b").unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();
        editor.write_file("subdir/c.txt", "c").unwrap();

        let entries = editor.list_files("").unwrap();
        assert_eq!(entries.len(), 3); // a.txt, b.txt, subdir

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_path_escape_prevention() {
        let dir = setup_test_dir();
        let editor = FileEditor::new(dir.clone());

        let result = editor.read_file("../etc/passwd");
        assert!(result.is_err());

        cleanup_test_dir(&dir);
    }
}
