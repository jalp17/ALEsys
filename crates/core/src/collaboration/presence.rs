//! Presence System - Track user locations and statuses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Active,
    Idle,
    Typing,
    Offline,
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    pub user_id: String,
    pub username: String,
    pub cursor_position: Option<usize>,
    pub selection: Option<(usize, usize)>,
    pub status: UserStatus,
    pub color: String,
}

/// Manages presence for a room
pub struct PresenceManager {
    users: HashMap<String, Presence>,
}

impl PresenceManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Update or add user presence
    pub fn update(&mut self, presence: Presence) {
        self.users.insert(presence.user_id.clone(), presence);
    }

    /// Remove user from presence
    pub fn remove(&mut self, user_id: &str) {
        self.users.remove(user_id);
    }

    /// Get all users in room
    pub fn list(&self) -> Vec<Presence> {
        self.users.values().cloned().collect()
    }

    /// Get specific user presence
    pub fn get(&self, user_id: &str) -> Option<&Presence> {
        self.users.get(user_id)
    }

    /// Get online count
    pub fn online_count(&self) -> usize {
        self.users
            .values()
            .filter(|u| u.status != UserStatus::Offline)
            .count()
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_presence() {
        let mut manager = PresenceManager::new();
        let presence = Presence {
            user_id: "user1".to_string(),
            username: "Alice".to_string(),
            cursor_position: Some(0),
            selection: None,
            status: UserStatus::Active,
            color: "#FF0000".to_string(),
        };
        manager.update(presence);
        assert_eq!(manager.online_count(), 1);
    }

    #[test]
    fn test_remove_presence() {
        let mut manager = PresenceManager::new();
        let presence = Presence {
            user_id: "user1".to_string(),
            username: "Alice".to_string(),
            cursor_position: Some(0),
            selection: None,
            status: UserStatus::Active,
            color: "#FF0000".to_string(),
        };
        manager.update(presence);
        manager.remove("user1");
        assert_eq!(manager.online_count(), 0);
    }
}
