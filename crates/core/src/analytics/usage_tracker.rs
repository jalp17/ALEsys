use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub id: String,
    pub event_type: String,
    pub user_id: String,
    pub timestamp: String,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct UsageTracker {
    events: Vec<UsageEvent>,
}

impl UsageTracker {
    pub fn new() -> Self {
        Self { events: vec![] }
    }

    pub fn track(&mut self, event_type: &str, user_id: &str) -> UsageEvent {
        let event = UsageEvent {
            id: format!("event-{}", self.events.len()),
            event_type: event_type.to_string(),
            user_id: user_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: std::collections::HashMap::new(),
        };
        self.events.push(event.clone());
        event
    }

    pub fn get_events(&self) -> &[UsageEvent] {
        &self.events
    }

    pub fn get_events_by_type(&self, event_type: &str) -> Vec<&UsageEvent> {
        self.events.iter().filter(|e| e.event_type == event_type).collect()
    }

    pub fn get_events_by_user(&self, user_id: &str) -> Vec<&UsageEvent> {
        self.events.iter().filter(|e| e.user_id == user_id).collect()
    }

    pub fn get_stats(&self) -> UsageStats {
        let mut by_type = std::collections::HashMap::new();
        let mut by_user = std::collections::HashMap::new();

        for event in &self.events {
            *by_type.entry(event.event_type.clone()).or_insert(0) += 1;
            *by_user.entry(event.user_id.clone()).or_insert(0) += 1;
        }

        UsageStats {
            total_events: self.events.len(),
            unique_users: by_user.len(),
            events_by_type: by_type,
            events_by_user: by_user,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_events: usize,
    pub unique_users: usize,
    pub events_by_type: std::collections::HashMap<String, usize>,
    pub events_by_user: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_event() {
        let mut tracker = UsageTracker::new();
        let event = tracker.track("chat", "user-1");
        assert_eq!(event.event_type, "chat");
        assert_eq!(event.user_id, "user-1");
    }

    #[test]
    fn test_get_events_by_type() {
        let mut tracker = UsageTracker::new();
        tracker.track("chat", "user-1");
        tracker.track("search", "user-1");
        tracker.track("chat", "user-2");
        let chat_events = tracker.get_events_by_type("chat");
        assert_eq!(chat_events.len(), 2);
    }

    #[test]
    fn test_get_events_by_user() {
        let mut tracker = UsageTracker::new();
        tracker.track("chat", "user-1");
        tracker.track("search", "user-2");
        let user1_events = tracker.get_events_by_user("user-1");
        assert_eq!(user1_events.len(), 1);
    }

    #[test]
    fn test_stats() {
        let mut tracker = UsageTracker::new();
        tracker.track("chat", "user-1");
        tracker.track("chat", "user-1");
        tracker.track("search", "user-2");
        let stats = tracker.get_stats();
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.unique_users, 2);
    }

    #[test]
    fn test_empty_tracker() {
        let tracker = UsageTracker::new();
        let stats = tracker.get_stats();
        assert_eq!(stats.total_events, 0);
    }
}