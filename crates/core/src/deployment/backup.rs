use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub backup_dir: String,
    pub max_backups: usize,
    pub compress: bool,
    pub include_logs: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: "./backups".to_string(),
            max_backups: 7,
            compress: true,
            include_logs: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub id: String,
    pub timestamp: u64,
    pub size_bytes: u64,
    pub success: bool,
    pub message: String,
}

pub struct BackupManager {
    config: BackupConfig,
    backups: Vec<BackupResult>,
}

impl BackupManager {
    pub fn new(config: BackupConfig) -> Self {
        Self { config, backups: Vec::new() }
    }

    pub fn create_backup(&mut self, data: &[u8]) -> BackupResult {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!("backup-{}", timestamp);
        let size = data.len() as u64;

        let result = BackupResult {
            id: id.clone(),
            timestamp,
            size_bytes: size,
            success: true,
            message: format!("Backup created: {} bytes", size),
        };

        self.backups.push(result.clone());

        if self.backups.len() > self.config.max_backups {
            self.backups.remove(0);
        }

        result
    }

    pub fn list_backups(&self) -> &[BackupResult] {
        &self.backups
    }

    pub fn get_latest(&self) -> Option<&BackupResult> {
        self.backups.last()
    }

    pub fn delete_backup(&mut self, id: &str) -> bool {
        if let Some(pos) = self.backups.iter().position(|b| b.id == id) {
            self.backups.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_backup() {
        let mut manager = BackupManager::new(BackupConfig::default());
        let result = manager.create_backup(b"test data");
        assert!(result.success);
        assert_eq!(result.size_bytes, 9);
    }

    #[test]
    fn test_list_backups() {
        let mut manager = BackupManager::new(BackupConfig::default());
        manager.create_backup(b"data1");
        manager.create_backup(b"data2");
        assert_eq!(manager.list_backups().len(), 2);
    }

    #[test]
    fn test_max_backups() {
        let config = BackupConfig { max_backups: 2, ..Default::default() };
        let mut manager = BackupManager::new(config);
        manager.create_backup(b"1");
        manager.create_backup(b"2");
        manager.create_backup(b"3");
        assert_eq!(manager.list_backups().len(), 2);
    }

    #[test]
    fn test_delete_backup() {
        let mut manager = BackupManager::new(BackupConfig::default());
        let result = manager.create_backup(b"data");
        assert!(manager.delete_backup(&result.id));
        assert!(!manager.delete_backup("nonexistent"));
    }
}