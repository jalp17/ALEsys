//! Gestión de sesiones multi-usuario
//!
//! Cada sesión aísla:
//! - Historial de chat
//! - Contexto de RAG
//! - Archivos trabajados

use crate::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionManager {
    db: PgPool,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub sources: Option<Vec<String>>,
}

impl SessionManager {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_session(&self, user_id: i32, name: Option<String>) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let session_name =
            name.unwrap_or_else(|| format!("Sesión {}", Utc::now().format("%Y-%m-%d %H:%M")));

        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, name, created_at, last_activity, is_active)
            VALUES ($1, $2, $3, NOW(), NOW(), true)
            "#,
        )
        .bind(&session_id)
        .bind(user_id)
        .bind(&session_name)
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("DB error creando sesión: {}", e);
            crate::AlesysError::Session(format!("Error creando sesión: {}", e))
        })?;

        tracing::info!("Sesión creada: {} (user={})", session_id, user_id);
        Ok(session_id)
    }

    pub async fn get_active_sessions(&self, user_id: i32) -> Result<Vec<Session>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, name, created_at, last_activity, is_active
            FROM user_sessions
            WHERE user_id = $1 AND is_active = true
            ORDER BY last_activity DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("DB error listando sesiones activas: {}", e);
            crate::AlesysError::Session(format!("Error listando sesiones: {}", e))
        })?;

        tracing::debug!("Sesiones activas para user={}: {}", user_id, rows.len());
        let sessions = rows
            .iter()
            .map(|row| Session {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                created_at: row.get("created_at"),
                last_activity: row.get("last_activity"),
                is_active: row.get("is_active"),
            })
            .collect();

        Ok(sessions)
    }

    pub async fn get_by_id(&self, session_id: &str) -> Result<Option<Session>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, name, created_at, last_activity, is_active
            FROM user_sessions
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("DB error obteniendo sesión {}: {}", session_id, e);
            crate::AlesysError::Session(format!("Error obteniendo sesión: {}", e))
        })?;

        Ok(row.map(|r| Session {
            id: r.get("id"),
            user_id: r.get("user_id"),
            name: r.get("name"),
            created_at: r.get("created_at"),
            last_activity: r.get("last_activity"),
            is_active: r.get("is_active"),
        }))
    }

    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let result = sqlx::query(
            "UPDATE user_sessions SET is_active = false, closed_at = NOW() WHERE id = $1",
        )
        .bind(session_id)
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("DB error cerrando sesión {}: {}", session_id, e);
            crate::AlesysError::Session(format!("Error cerrando sesión: {}", e))
        })?;

        if result.rows_affected() == 0 {
            tracing::warn!("Sesión {} no encontrada para cerrar", session_id);
            return Err(crate::AlesysError::Session(format!(
                "Sesión {} no encontrada",
                session_id
            )));
        }

        tracing::info!("Sesión cerrada: {}", session_id);
        Ok(())
    }

    pub async fn add_message(&self, session_id: &str, message: &ChatMessage) -> Result<()> {
        let mut tx = self.db.begin().await.map_err(|e| {
            tracing::error!("DB error iniciando transacción para mensaje: {}", e);
            crate::AlesysError::Session(format!("Error iniciando transacción: {}", e))
        })?;

        sqlx::query(
            r#"
            INSERT INTO session_messages (session_id, role, content, timestamp, sources)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(session_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(message.timestamp)
        .bind(
            message
                .sources
                .as_ref()
                .and_then(|s| serde_json::to_value(s).ok()),
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB error insertando mensaje en sesión {}: {}", session_id, e);
            crate::AlesysError::Session(format!("Error guardando mensaje: {}", e))
        })?;

        sqlx::query("UPDATE user_sessions SET last_activity = NOW() WHERE id = $1")
            .bind(session_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("DB error actualizando last_activity sesión {}: {}", session_id, e);
                crate::AlesysError::Session(format!("Error actualizando actividad: {}", e))
            })?;

        tx.commit().await.map_err(|e| {
            tracing::error!("DB error commitiendo transacción sesión {}: {}", session_id, e);
            crate::AlesysError::Session(format!("Error confirmando mensaje: {}", e))
        })?;

        tracing::debug!("Mensaje guardado en sesión {} (role={})", session_id, message.role);
        Ok(())
    }

    pub async fn get_session_history(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>> {
        let rows = sqlx::query(
            r#"
            SELECT role, content, timestamp, sources
            FROM session_messages
            WHERE session_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(session_id)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("DB error cargando historial sesión {}: {}", session_id, e);
            crate::AlesysError::Session(format!("Error cargando historial: {}", e))
        })?;

        tracing::debug!("Historial sesión {}: {} mensajes", session_id, rows.len());
        let messages = rows
            .iter()
            .rev()
            .map(|row| ChatMessage {
                role: row.get("role"),
                content: row.get("content"),
                timestamp: row.get("timestamp"),
                sources: row.get("sources"),
            })
            .collect();

        Ok(messages)
    }

    pub async fn get_session_context(&self, session_id: &str) -> Result<serde_json::Value> {
        let result = sqlx::query("SELECT context_data FROM session_context WHERE session_id = $1")
            .bind(session_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("DB error cargando contexto sesión {}: {}", session_id, e);
                crate::AlesysError::Session(format!("Error cargando contexto: {}", e))
            })?;

        Ok(result
            .and_then(|r| r.get::<Option<serde_json::Value>, _>("context_data"))
            .unwrap_or(serde_json::json!({})))
    }

    pub async fn update_session_context(
        &self,
        session_id: &str,
        context: &serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO session_context (session_id, context_data)
            VALUES ($1, $2)
            ON CONFLICT (session_id) DO UPDATE SET context_data = $2
            "#,
        )
        .bind(session_id)
        .bind(context)
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("DB error actualizando contexto sesión {}: {}", session_id, e);
            crate::AlesysError::Session(format!("Error actualizando contexto: {}", e))
        })?;

        tracing::debug!("Contexto sesión {} actualizado", session_id);
        Ok(())
    }
}
