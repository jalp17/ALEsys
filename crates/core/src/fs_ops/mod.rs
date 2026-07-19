use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
}

pub async fn read_file(path: &str) -> Result<String, String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Error reading file '{}': {}", path, e))
}

pub async fn write_file(path: &str, content: &str) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Error creating directory '{}': {}", parent.display(), e))?;
    }
    tokio::fs::write(path, content)
        .await
        .map_err(|e| format!("Error writing file '{}': {}", path, e))
}

pub async fn list_directory(path: &str) -> Result<Vec<FileItem>, String> {
    let mut entries = tokio::fs::read_dir(path)
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
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|e| format!("Error creating directory '{}': {}", path, e))
}

pub async fn delete_file(path: &str) -> Result<(), String> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|e| format!("Error accessing '{}': {}", path, e))?;

    if metadata.is_dir() {
        tokio::fs::remove_dir_all(path)
            .await
            .map_err(|e| format!("Error removing directory '{}': {}", path, e))
    } else {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| format!("Error removing file '{}': {}", path, e))
    }
}

pub async fn copy_file(from: &str, to: &str) -> Result<(), String> {
    tokio::fs::copy(from, to)
        .await
        .map_err(|e| format!("Error copying '{}' to '{}': {}", from, to, e))?;
    Ok(())
}

pub async fn move_file(from: &str, to: &str) -> Result<(), String> {
    tokio::fs::rename(from, to)
        .await
        .map_err(|e| format!("Error moving '{}' to '{}': {}", from, to, e))
}

pub async fn file_exists(path: &str) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}

pub async fn file_size(path: &str) -> Result<u64, String> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|e| format!("Error accessing '{}': {}", path, e))?;
    Ok(metadata.len())
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
}
