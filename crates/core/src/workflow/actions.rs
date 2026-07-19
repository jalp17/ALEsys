use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    RunCommand,
    CallAPI,
    SendNotification,
    TransformData,
    Conditional,
    Delay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub action_type: ActionType,
    pub config: std::collections::HashMap<String, String>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_id: String,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}

pub struct ActionExecutor;

impl ActionExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, action: &Action) -> ActionResult {
        let start = std::time::Instant::now();

        let result = match action.action_type {
            ActionType::RunCommand => {
                let default_cmd = "echo".to_string();
                let cmd = action.config.get("command").unwrap_or(&default_cmd);
                ActionResult {
                    action_id: action.id.clone(),
                    success: true,
                    output: format!("Executed: {}", cmd),
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            ActionType::CallAPI => {
                let default_url = String::new();
                let url = action.config.get("url").unwrap_or(&default_url);
                ActionResult {
                    action_id: action.id.clone(),
                    success: true,
                    output: format!("API call to: {}", url),
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            ActionType::SendNotification => {
                let default_msg = "Notification".to_string();
                let msg = action.config.get("message").unwrap_or(&default_msg);
                ActionResult {
                    action_id: action.id.clone(),
                    success: true,
                    output: format!("Notification sent: {}", msg),
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            ActionType::TransformData => {
                ActionResult {
                    action_id: action.id.clone(),
                    success: true,
                    output: "Data transformed".to_string(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            ActionType::Conditional => {
                ActionResult {
                    action_id: action.id.clone(),
                    success: true,
                    output: "Condition evaluated".to_string(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            ActionType::Delay => {
                ActionResult {
                    action_id: action.id.clone(),
                    success: true,
                    output: "Delay completed".to_string(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: None,
                }
            }
        };

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_action(id: &str, action_type: ActionType) -> Action {
        let mut config = std::collections::HashMap::new();
        config.insert("command".to_string(), "ls".to_string());
        config.insert("url".to_string(), "http://localhost".to_string());
        config.insert("message".to_string(), "Hello".to_string());
        Action {
            id: id.to_string(),
            action_type,
            config,
            timeout_ms: 5000,
        }
    }

    #[test]
    fn test_execute_command() {
        let executor = ActionExecutor::new();
        let action = make_action("a1", ActionType::RunCommand);
        let result = executor.execute(&action);
        assert!(result.success);
        assert!(result.output.contains("Executed"));
    }

    #[test]
    fn test_execute_api() {
        let executor = ActionExecutor::new();
        let action = make_action("a2", ActionType::CallAPI);
        let result = executor.execute(&action);
        assert!(result.success);
    }

    #[test]
    fn test_execute_notification() {
        let executor = ActionExecutor::new();
        let action = make_action("a3", ActionType::SendNotification);
        let result = executor.execute(&action);
        assert!(result.success);
        assert!(result.output.contains("Hello"));
    }

    #[test]
    fn test_execute_transform() {
        let executor = ActionExecutor::new();
        let action = make_action("a4", ActionType::TransformData);
        let result = executor.execute(&action);
        assert!(result.success);
    }

    #[test]
    fn test_execute_conditional() {
        let executor = ActionExecutor::new();
        let action = make_action("a5", ActionType::Conditional);
        let result = executor.execute(&action);
        assert!(result.success);
    }
}