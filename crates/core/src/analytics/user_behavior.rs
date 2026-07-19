use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAction {
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub timestamp: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub pattern_name: String,
    pub frequency: usize,
    pub users: Vec<String>,
    pub description: String,
}

pub struct BehaviorAnalyzer {
    actions: Vec<UserAction>,
}

impl BehaviorAnalyzer {
    pub fn new() -> Self {
        Self { actions: vec![] }
    }

    pub fn record_action(&mut self, user_id: &str, action: &str, resource: &str) -> UserAction {
        let user_action = UserAction {
            user_id: user_id.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: None,
        };
        self.actions.push(user_action.clone());
        user_action
    }

    pub fn get_actions(&self) -> &[UserAction] {
        &self.actions
    }

    pub fn get_user_actions(&self, user_id: &str) -> Vec<&UserAction> {
        self.actions.iter().filter(|a| a.user_id == user_id).collect()
    }

    pub fn get_action_frequency(&self) -> std::collections::HashMap<String, usize> {
        let mut freq = std::collections::HashMap::new();
        for action in &self.actions {
            *freq.entry(action.action.clone()).or_insert(0) += 1;
        }
        freq
    }

    pub fn detect_patterns(&self) -> Vec<BehaviorPattern> {
        let freq = self.get_action_frequency();
        let mut patterns = Vec::new();

        for (action, count) in &freq {
            if *count >= 3 {
                let users: Vec<String> = self.actions.iter()
                    .filter(|a| a.action == *action)
                    .map(|a| a.user_id.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                patterns.push(BehaviorPattern {
                    pattern_name: format!("frequent_{}", action),
                    frequency: *count,
                    users,
                    description: format!("Action '{}' performed {} times", action, count),
                });
            }
        }

        patterns
    }

    pub fn get_stats(&self) -> BehaviorStats {
        let unique_users: std::collections::HashSet<&str> = self.actions.iter().map(|a| a.user_id.as_str()).collect();
        let unique_actions: std::collections::HashSet<&str> = self.actions.iter().map(|a| a.action.as_str()).collect();

        BehaviorStats {
            total_actions: self.actions.len(),
            unique_users: unique_users.len(),
            unique_actions: unique_actions.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorStats {
    pub total_actions: usize,
    pub unique_users: usize,
    pub unique_actions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_action() {
        let mut analyzer = BehaviorAnalyzer::new();
        let action = analyzer.record_action("user-1", "search", "/docs");
        assert_eq!(action.user_id, "user-1");
    }

    #[test]
    fn test_action_frequency() {
        let mut analyzer = BehaviorAnalyzer::new();
        analyzer.record_action("user-1", "search", "/docs");
        analyzer.record_action("user-2", "search", "/api");
        analyzer.record_action("user-1", "chat", "/chat");
        let freq = analyzer.get_action_frequency();
        assert_eq!(freq.get("search"), Some(&2));
    }

    #[test]
    fn test_detect_patterns() {
        let mut analyzer = BehaviorAnalyzer::new();
        for i in 0..5 {
            analyzer.record_action(&format!("user-{}", i), "search", "/docs");
        }
        let patterns = analyzer.detect_patterns();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].frequency, 5);
    }

    #[test]
    fn test_stats() {
        let mut analyzer = BehaviorAnalyzer::new();
        analyzer.record_action("user-1", "search", "/docs");
        analyzer.record_action("user-2", "chat", "/chat");
        let stats = analyzer.get_stats();
        assert_eq!(stats.total_actions, 2);
        assert_eq!(stats.unique_users, 2);
    }

    #[test]
    fn test_empty_analyzer() {
        let analyzer = BehaviorAnalyzer::new();
        let patterns = analyzer.detect_patterns();
        assert!(patterns.is_empty());
    }
}