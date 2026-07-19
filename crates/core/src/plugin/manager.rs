//! Plugin Manager - Dynamic loading and lifecycle management

use super::api::{Plugin, PluginConfig, PluginContext, PluginMetadata, PluginResult};
use super::registry::PluginRegistry;
use super::security::SecuritySandbox;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Loaded plugin with metadata
struct LoadedPlugin {
    #[allow(dead_code)]
    metadata: PluginMetadata,
    instance: Box<dyn Plugin>,
    #[allow(dead_code)]
    sandbox: SecuritySandbox,
}

/// Manages plugin loading, lifecycle, and execution
pub struct PluginManager {
    plugins: RwLock<HashMap<String, LoadedPlugin>>,
    registry: PluginRegistry,
    plugin_dir: PathBuf,
    config: PluginConfig,
}

impl PluginManager {
    /// Create a new PluginManager
    pub fn new(plugin_dir: PathBuf, db: &sqlx::PgPool) -> Self {
        let registry = PluginRegistry::new(db.clone());
        let config = Self::load_config(&plugin_dir);

        Self {
            plugins: RwLock::new(HashMap::new()),
            registry,
            plugin_dir,
            config,
        }
    }

    /// Load configuration from plugins.toml
    fn load_config(plugin_dir: &Path) -> PluginConfig {
        let config_path = plugin_dir.join("plugins.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            toml::from_str(&content).unwrap_or(PluginConfig { plugins: vec![] })
        } else {
            PluginConfig { plugins: vec![] }
        }
    }

    /// Initialize all enabled plugins
    pub async fn init_all(&self) -> Result<(), String> {
        let config = self.config.clone();

        for entry in &config.plugins {
            if entry.enabled {
                match self.load_plugin(entry).await {
                    Ok(_) => {
                        tracing::info!("Plugin '{}' loaded successfully", entry.id);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load plugin '{}': {}", entry.id, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single plugin
    async fn load_plugin(&self, entry: &super::api::PluginEntry) -> Result<(), String> {
        let lib_path = &entry.path;

        if !lib_path.exists() {
            return Err(format!("Plugin library not found: {:?}", lib_path));
        }

        // Create sandbox with permissions from metadata
        let sandbox = SecuritySandbox::new(vec![]);

        // For now, use a stub plugin implementation
        // In production, this would use libloading to load .so/.dll
        let plugin = self.create_stub_plugin(&entry.id)?;

        let metadata = plugin.metadata();

        // Register in database
        self.registry
            .register_plugin(&metadata, lib_path)
            .await
            .map_err(|e| format!("Failed to register plugin: {}", e))?;

        let mut plugins = self.plugins.write().await;
        plugins.insert(
            entry.id.clone(),
            LoadedPlugin {
                metadata,
                instance: plugin,
                sandbox,
            },
        );

        Ok(())
    }

    /// Create a stub plugin for testing
    fn create_stub_plugin(&self, id: &str) -> Result<Box<dyn Plugin>, String> {
        // This is a temporary stub - in production, load from .so/.dll
        match id {
            "git-integration" => Ok(Box::new(GitIntegrationPlugin::new())),
            "test-runner" => Ok(Box::new(TestRunnerPlugin::new())),
            "docker-runner" => Ok(Box::new(DockerRunnerPlugin::new())),
            _ => Err(format!("Unknown built-in plugin: {}", id)),
        }
    }

    /// Execute a plugin command
    pub async fn execute(
        &self,
        plugin_id: &str,
        command: &str,
        args: &[String],
        context: &PluginContext,
    ) -> Result<PluginResult, String> {
        let plugins = self.plugins.read().await;
        let plugin = plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

        if !plugin.instance.can_handle(command) {
            return Err(format!(
                "Plugin '{}' cannot handle command '{}'",
                plugin_id, command
            ));
        }

        // Execute with timeout
        let instance = &plugin.instance;
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            async { instance.execute(command, args, context) },
        )
        .await
        .map_err(|_| "Plugin execution timed out".to_string())?;

        result
    }

    /// Shutdown all plugins
    pub async fn shutdown_all(&self) -> Result<(), String> {
        let mut plugins = self.plugins.write().await;

        for (id, mut plugin) in plugins.drain() {
            if let Err(e) = plugin.instance.shutdown() {
                tracing::warn!("Failed to shutdown plugin '{}': {}", id, e);
            }
        }

        Ok(())
    }

    /// Get list of loaded plugins
    pub async fn list_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins.values().map(|p| p.metadata.clone()).collect()
    }

    /// Check if a plugin is loaded
    pub async fn is_loaded(&self, plugin_id: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins.contains_key(plugin_id)
    }
}

// Built-in plugins (pub for testing)

pub struct GitIntegrationPlugin {
    metadata: PluginMetadata,
}

impl GitIntegrationPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "git-integration".to_string(),
                name: "Git Integration".to_string(),
                version: "0.1.0".to_string(),
                author: "ALEsys".to_string(),
                description: "Git integration for ALEsys".to_string(),
                permissions: vec![super::api::PluginPermission::Execute {
                    allowed_commands: vec!["git".to_string()],
                }],
                min_alesys_version: "1.16.0".to_string(),
                hooks: vec!["pre-commit".to_string(), "post-commit".to_string()],
            },
        }
    }
}

impl Plugin for GitIntegrationPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _context: &PluginContext) -> Result<(), String> {
        Ok(())
    }

    fn execute(
        &self,
        command: &str,
        args: &[String],
        _context: &PluginContext,
    ) -> Result<PluginResult, String> {
        match command {
            "git.status" => Ok(PluginResult {
                success: true,
                output: Some("On branch master".to_string()),
                error: None,
                metadata: HashMap::new(),
            }),
            "git.diff" => Ok(PluginResult {
                success: true,
                output: Some("No changes".to_string()),
                error: None,
                metadata: HashMap::new(),
            }),
            _ => Err(format!("Unknown command: {}", command)),
        }
    }

    fn shutdown(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn can_handle(&self, command: &str) -> bool {
        command.starts_with("git.")
    }

    fn supported_commands(&self) -> Vec<String> {
        vec![
            "git.status".to_string(),
            "git.diff".to_string(),
            "git.log".to_string(),
        ]
    }
}

pub struct TestRunnerPlugin {
    metadata: PluginMetadata,
}

impl TestRunnerPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "test-runner".to_string(),
                name: "Test Runner".to_string(),
                version: "0.1.0".to_string(),
                author: "ALEsys".to_string(),
                description: "Run tests automatically".to_string(),
                permissions: vec![super::api::PluginPermission::Execute {
                    allowed_commands: vec![
                        "cargo".to_string(),
                        "pytest".to_string(),
                        "jest".to_string(),
                    ],
                }],
                min_alesys_version: "1.16.0".to_string(),
                hooks: vec!["post-generate".to_string()],
            },
        }
    }
}

impl Plugin for TestRunnerPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _context: &PluginContext) -> Result<(), String> {
        Ok(())
    }

    fn execute(
        &self,
        command: &str,
        _args: &[String],
        _context: &PluginContext,
    ) -> Result<PluginResult, String> {
        match command {
            "test.run" => Ok(PluginResult {
                success: true,
                output: Some("All tests passed".to_string()),
                error: None,
                metadata: HashMap::new(),
            }),
            _ => Err(format!("Unknown command: {}", command)),
        }
    }

    fn shutdown(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn can_handle(&self, command: &str) -> bool {
        command.starts_with("test.")
    }

    fn supported_commands(&self) -> Vec<String> {
        vec!["test.run".to_string(), "test.report".to_string()]
    }
}

pub struct DockerRunnerPlugin {
    metadata: PluginMetadata,
}

impl DockerRunnerPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "docker-runner".to_string(),
                name: "Docker Runner".to_string(),
                version: "0.1.0".to_string(),
                author: "ALEsys".to_string(),
                description: "Run code in Docker containers".to_string(),
                permissions: vec![super::api::PluginPermission::Execute {
                    allowed_commands: vec!["docker".to_string()],
                }],
                min_alesys_version: "1.16.0".to_string(),
                hooks: vec![],
            },
        }
    }
}

impl Plugin for DockerRunnerPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn init(&mut self, _context: &PluginContext) -> Result<(), String> {
        Ok(())
    }

    fn execute(
        &self,
        command: &str,
        _args: &[String],
        _context: &PluginContext,
    ) -> Result<PluginResult, String> {
        match command {
            "docker.run" => Ok(PluginResult {
                success: true,
                output: Some("Container executed".to_string()),
                error: None,
                metadata: HashMap::new(),
            }),
            _ => Err(format!("Unknown command: {}", command)),
        }
    }

    fn shutdown(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn can_handle(&self, command: &str) -> bool {
        command.starts_with("docker.")
    }

    fn supported_commands(&self) -> Vec<String> {
        vec!["docker.run".to_string(), "docker.build".to_string()]
    }
}
