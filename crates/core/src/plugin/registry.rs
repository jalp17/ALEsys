//! Plugin Registry - SQLite-based plugin tracking

use super::api::PluginMetadata;
use std::path::Path;
use sqlx::PgPool;

/// Registry for installed plugins
pub struct PluginRegistry {
    db: PgPool,
}

impl PluginRegistry {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Register a plugin in the database
    pub async fn register_plugin(
        &self,
        metadata: &PluginMetadata,
        path: &Path,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO plugins (id, name, version, author, description, path, installed_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (id) DO UPDATE SET
                version = EXCLUDED.version,
                updated_at = NOW()
            "#,
        )
        .bind(&metadata.id)
        .bind(&metadata.name)
        .bind(&metadata.version)
        .bind(&metadata.author)
        .bind(&metadata.description)
        .bind(path.to_string_lossy().to_string())
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Unregister a plugin
    pub async fn unregister_plugin(&self, plugin_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM plugins WHERE id = $1")
            .bind(plugin_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// List all installed plugins
    pub async fn list_plugins(&self) -> Result<Vec<PluginMetadata>, sqlx::Error> {
        let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, name, version, author, description FROM plugins ORDER BY name",
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, name, version, author, description)| PluginMetadata {
                id,
                name,
                version,
                author,
                description,
                permissions: vec![],
                min_alesys_version: "1.16.0".to_string(),
                hooks: vec![],
            })
            .collect())
    }

    /// Get plugin by ID
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<Option<PluginMetadata>, sqlx::Error> {
        let row: Option<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, name, version, author, description FROM plugins WHERE id = $1",
        )
        .bind(plugin_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|(id, name, version, author, description)| PluginMetadata {
            id,
            name,
            version,
            author,
            description,
            permissions: vec![],
            min_alesys_version: "1.16.0".to_string(),
            hooks: vec![],
        }))
    }
}
