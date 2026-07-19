//! Plugin API types and traits

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Granular permissions for plugins
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginPermission {
    /// Read files within allowed directories
    FilesystemRead { allowed_paths: Vec<String> },
    /// Write files within allowed directories
    FilesystemWrite { allowed_paths: Vec<String> },
    /// Make network requests to allowed hosts
    Network { allowed_hosts: Vec<String> },
    /// Execute specific commands
    Execute { allowed_commands: Vec<String> },
    /// Access database (read-only)
    DatabaseRead,
    /// Access database (read-write)
    DatabaseWrite,
}

/// Metadata about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub permissions: Vec<PluginPermission>,
    pub min_alesys_version: String,
    pub hooks: Vec<String>,
}

/// Context passed to plugins during execution
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Working directory for the plugin
    pub work_dir: PathBuf,
    /// Allowed paths for filesystem access
    pub allowed_paths: Vec<String>,
    /// Plugin-specific configuration
    pub config: HashMap<String, String>,
    /// Request ID for logging
    pub request_id: String,
}

/// Result of plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin with context
    fn init(&mut self, context: &PluginContext) -> Result<(), String>;

    /// Execute a command
    fn execute(&self, command: &str, args: &[String], context: &PluginContext)
        -> Result<PluginResult, String>;

    /// Shutdown the plugin gracefully
    fn shutdown(&mut self) -> Result<(), String>;

    /// Check if plugin can handle a command
    fn can_handle(&self, command: &str) -> bool;

    /// Get list of supported commands
    fn supported_commands(&self) -> Vec<String>;
}

/// Plugin configuration from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugins: Vec<PluginEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntry {
    pub id: String,
    pub enabled: bool,
    pub path: PathBuf,
    #[serde(default)]
    pub config: HashMap<String, String>,
}
