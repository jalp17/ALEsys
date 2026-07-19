//! Collaboration Room - WebSocket room management

use super::ot::{Operation, OTEngine};
use super::presence::{Presence, PresenceManager};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabMessage {
    pub room_id: String,
    pub user_id: String,
    pub payload: CollabPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollabPayload {
    Operation(Operation),
    PresenceUpdate(Presence),
    CursorSync { position: usize },
    UserJoined { user_id: String, username: String },
    UserLeft { user_id: String },
    DocumentSync { content: String },
}

/// A collaborative editing room
pub struct CollabRoom {
    pub id: String,
    pub name: String,
    content: String,
    ot_engine: OTEngine,
    presence: PresenceManager,
}

impl CollabRoom {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            content: String::new(),
            ot_engine: OTEngine::new(),
            presence: PresenceManager::new(),
        }
    }

    /// Apply an operation from a user
    pub fn apply_operation(&mut self, op: Operation) -> Result<(), String> {
        // Apply to OT engine
        self.ot_engine.apply(op.clone())?;

        // Apply to content
        match op.action {
            super::ot::OpAction::Insert => {
                if let Some(content) = &op.content {
                    self.content.insert_str(op.position, content);
                }
            }
            super::ot::OpAction::Delete => {
                let len = op.length.unwrap_or(1);
                let end = (op.position + len).min(self.content.len());
                self.content.drain(op.position..end);
            }
            super::ot::OpAction::Retain => {}
        }

        Ok(())
    }

    /// Get current content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Update presence
    pub fn update_presence(&mut self, presence: Presence) {
        self.presence.update(presence);
    }

    /// Remove user from room
    pub fn remove_user(&mut self, user_id: &str) {
        self.presence.remove(user_id);
    }

    /// Get all users in room
    pub fn users(&self) -> Vec<Presence> {
        self.presence.list()
    }

    /// Get online count
    pub fn online_count(&self) -> usize {
        self.presence.online_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ot::OpAction;

    #[test]
    fn test_create_room() {
        let room = CollabRoom::new("Test Room".to_string());
        assert_eq!(room.name, "Test Room");
        assert_eq!(room.content(), "");
    }

    #[test]
    fn test_apply_insert() {
        let mut room = CollabRoom::new("Test".to_string());
        let op = Operation {
            id: Uuid::new_v4(),
            user_id: "user1".to_string(),
            position: 0,
            action: OpAction::Insert,
            content: Some("Hello".to_string()),
            length: None,
        };
        assert!(room.apply_operation(op).is_ok());
        assert_eq!(room.content(), "Hello");
    }

    #[test]
    fn test_apply_delete() {
        let mut room = CollabRoom::new("Test".to_string());
        // First insert
        let op1 = Operation {
            id: Uuid::new_v4(),
            user_id: "user1".to_string(),
            position: 0,
            action: OpAction::Insert,
            content: Some("Hello World".to_string()),
            length: None,
        };
        room.apply_operation(op1).unwrap();

        // Then delete
        let op2 = Operation {
            id: Uuid::new_v4(),
            user_id: "user1".to_string(),
            position: 5,
            action: OpAction::Delete,
            content: None,
            length: Some(6),
        };
        assert!(room.apply_operation(op2).is_ok());
        assert_eq!(room.content(), "Hello");
    }
}
