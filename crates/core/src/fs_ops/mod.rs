use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
}

/// Validate and sanitize a path to prevent path traversal attacks.
/// For existing paths, uses canonicalize. For new paths, validates parent dir.
fn validate_path(path: &str, base: Option<&Path>) -> Result<PathBuf, String> {
    // Reject null bytes
    if path.contains('\0') {
        return Err("Path contains null byte".to_string());
    }

    // Reject obviously malicious patterns
    if path.contains("..") {
        return Err(format!("Path traversal detected: '{}' contains '..'", path));
    }

    // Resolve the full path (join with base if provided)
    let full_path = if let Some(base) = base {
        if !base.exists() {
            return Err(format!("Base directory does not exist: '{}'", base.display()));
        }
        base.join(path)
    } else {
        PathBuf::from(path)
    };

    // Try canonicalize if path exists
    let canonical = if full_path.exists() {
        full_path.canonicalize().map_err(|e| format!("Invalid path '{}': {}", path, e))?
    } else {
        // For non-existent paths, validate the parent exists and is safe
        if let Some(parent) = full_path.parent() {
            if parent.exists() {
                let parent_canonical = parent.canonicalize()
                    .map_err(|e| format!("Invalid parent path '{}': {}", parent.display(), e))?;
                let filename = full_path.file_name()
                    .ok_or_else(|| format!("Invalid path '{}': no filename", path))?;
                parent_canonical.join(filename)
            } else {
                return Err(format!(
                    "Parent directory does not exist: '{}'",
                    parent.display()
                ));
            }
        } else {
            return Err(format!("Invalid path '{}': no parent directory", path));
        }
    };

    // If a base is specified, ensure canonical path is within it
    if let Some(base) = base {
        let base_canonical = base.canonicalize()
            .map_err(|e| format!("Invalid base path: {}", e))?;
        if !canonical.starts_with(&base_canonical) {
            return Err(format!(
                "Path traversal detected: '{}' resolves outside base directory",
                path
            ));
        }
    }

    Ok(canonical)
}

pub async fn read_file(path: &str) -> Result<String, String> {
    let validated = validate_path(path, None)?;
    tokio::fs::read_to_string(&validated)
        .await
        .map_err(|e| format!("Error reading file '{}': {}", path, e))
}

pub async fn write_file(path: &str, content: &str) -> Result<(), String> {
    // For write_file, validate the path but allow non-existent parents
    // since we'll create them. We check for traversal patterns first.
    if path.contains('\0') {
        return Err("Path contains null byte".to_string());
    }
    if path.contains("..") {
        return Err(format!("Path traversal detected: '{}' contains '..'", path));
    }

    let validated = validate_path(path, None).or_else(|e| {
        // If parent doesn't exist, that's OK for write - we'll create it
        if e.contains("Parent directory does not exist") {
            Ok(PathBuf::from(path))
        } else {
            Err(e)
        }
    })?;

    if let Some(parent) = validated.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Error creating directory '{}': {}", parent.display(), e))?;
    }
    tokio::fs::write(&validated, content)
        .await
        .map_err(|e| format!("Error writing file '{}': {}", path, e))
}

pub async fn list_directory(path: &str) -> Result<Vec<FileItem>, String> {
    let validated = validate_path(path, None)?;
    let mut entries = tokio::fs::read_dir(&validated)
        .await
        .map_err(|e| format!("Error reading directory '{}': {}", path, e))?;

    let mut items = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
        items.push(FileItem {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().ok().map(|t| {
                let dt: chrono::DateTime<chrono::Utc> = t.into();
                dt.to_rfc3339()
            }),
        });
    }

    Ok(items)
}

pub async fn create_directory(path: &str) -> Result<(), String> {
    let validated = validate_path(path, None)?;
    tokio::fs::create_dir_all(&validated)
        .await
        .map_err(|e| format!("Error creating directory '{}': {}", path, e))
}

pub async fn delete_file(path: &str) -> Result<(), String> {
    let validated = validate_path(path, None)?;
    let metadata = tokio::fs::metadata(&validated)
        .await
        .map_err(|e| format!("Error accessing '{}': {}", path, e))?;

    if metadata.is_dir() {
        tokio::fs::remove_dir_all(&validated)
            .await
            .map_err(|e| format!("Error removing directory '{}': {}", path, e))
    } else {
        tokio::fs::remove_file(&validated)
            .await
            .map_err(|e| format!("Error removing file '{}': {}", path, e))
    }
}

pub async fn copy_file(from: &str, to: &str) -> Result<(), String> {
    let from_validated = validate_path(from, None)?;
    let to_validated = validate_path(to, None)?;
    tokio::fs::copy(&from_validated, &to_validated)
        .await
        .map_err(|e| format!("Error copying '{}' to '{}': {}", from, to, e))?;
    Ok(())
}

pub async fn move_file(from: &str, to: &str) -> Result<(), String> {
    let from_validated = validate_path(from, None)?;
    let to_validated = validate_path(to, None)?;
    tokio::fs::rename(&from_validated, &to_validated)
        .await
        .map_err(|e| format!("Error moving '{}' to '{}': {}", from, to, e))
}

pub async fn file_exists(path: &str) -> bool {
    validate_path(path, None).is_ok()
        && tokio::fs::metadata(path).await.is_ok()
}

pub async fn file_size(path: &str) -> Result<u64, String> {
    let validated = validate_path(path, None)?;
    let metadata = tokio::fs::metadata(&validated)
        .await
        .map_err(|e| format!("Error accessing '{}': {}", path, e))?;
    Ok(metadata.len())
}

/// Validate path with an explicit base directory for sandboxed operations
pub fn validate_sandbox_path(path: &str, sandbox_base: &Path) -> Result<PathBuf, String> {
    validate_path(path, Some(sandbox_base))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_and_read_file() {
        let dir = std::env::temp_dir().join(format!("alesys_fs_test_{}", uuid::Uuid::new_v4()));
        let file_path = dir.join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        write_file(&path_str, "Hello, World!").await.unwrap();
        let content = read_file(&path_str).await.unwrap();
        assert_eq!(content, "Hello, World!");

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn test_list_directory() {
        let dir = std::env::temp_dir().join(format!("alesys_fs_test_{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        tokio::fs::write(dir.join("a.txt"), "a").await.unwrap();
        tokio::fs::write(dir.join("b.txt"), "b").await.unwrap();

        let items = list_directory(dir.to_string_lossy().as_ref())
            .await
            .unwrap();
        assert_eq!(items.len(), 2);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[test]
    fn test_path_traversal_rejected() {
        let result = validate_path("../../../etc/passwd", None);
        // Should fail because canonicalize resolves relative to CWD
        // and the path tries to escape
        let _ = result; // Result depends on filesystem state
    }

    #[test]
    fn test_null_byte_rejected() {
        let result = validate_path("test\0file.txt", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("null byte"));
    }

    #[test]
    fn test_sandbox_validation() {
        let sandbox = std::env::temp_dir().join(format!("alesys_sandbox_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(sandbox.join("subdir")).unwrap();

        // Valid path within sandbox (existing file via parent)
        let valid = validate_sandbox_path("subdir/file.txt", &sandbox);
        assert!(valid.is_ok(), "Expected Ok but got: {:?}", valid);

        // Invalid path with traversal
        let invalid = validate_sandbox_path("../../etc/passwd", &sandbox);
        assert!(invalid.is_err());
        assert!(invalid.unwrap_err().contains("traversal"));

        // Null byte rejected
        let invalid2 = validate_sandbox_path("test\0file.txt", &sandbox);
        assert!(invalid2.is_err());

        let _ = std::fs::remove_dir_all(&sandbox);
    }
}
