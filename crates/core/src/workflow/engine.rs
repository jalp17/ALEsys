use serde::{Deserialize, Serialize};
use super::builder::Workflow;
use super::actions::ActionExecutor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub step_id: String,
    pub step_name: String,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_id: String,
    pub success: bool,
    pub logs: Vec<ExecutionLog>,
    pub total_duration_ms: u64,
}

pub struct WorkflowEngine {
    executor: ActionExecutor,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            executor: ActionExecutor::new(),
        }
    }

    pub fn execute(&self, workflow: &Workflow) -> WorkflowResult {
        let start = std::time::Instant::now();
        let mut logs = Vec::new();
        let mut all_success = true;

        for step in &workflow.steps {
            let result = self.executor.execute(&step.action);

            logs.push(ExecutionLog {
                step_id: step.id.clone(),
                step_name: step.name.clone(),
                success: result.success,
                output: result.output,
                duration_ms: result.duration_ms,
            });

            if !result.success {
                all_success = false;
                break;
            }
        }

        WorkflowResult {
            workflow_id: workflow.id.clone(),
            success: all_success,
            logs,
            total_duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::builder::WorkflowBuilder;
    use crate::workflow::actions::{Action, ActionType};

    fn make_action(id: &str) -> Action {
        let mut config = std::collections::HashMap::new();
        config.insert("command".to_string(), "echo test".to_string());
        Action {
            id: id.to_string(),
            action_type: ActionType::RunCommand,
            config,
            timeout_ms: 5000,
        }
    }

    #[test]
    fn test_execute_workflow() {
        let workflow = WorkflowBuilder::new("w1", "Test")
            .step("s1", "Step 1", make_action("a1"))
            .step("s2", "Step 2", make_action("a2"))
            .build();
        let engine = WorkflowEngine::new();
        let result = engine.execute(&workflow);
        assert!(result.success);
        assert_eq!(result.logs.len(), 2);
    }

    #[test]
    fn test_execute_empty_workflow() {
        let workflow = WorkflowBuilder::new("w1", "Empty").build();
        let engine = WorkflowEngine::new();
        let result = engine.execute(&workflow);
        assert!(result.success);
        assert!(result.logs.is_empty());
    }

    #[test]
    fn test_workflow_duration() {
        let workflow = WorkflowBuilder::new("w1", "Test")
            .step("s1", "Step", make_action("a1"))
            .build();
        let engine = WorkflowEngine::new();
        let result = engine.execute(&workflow);
        assert!(result.total_duration_ms < 1000);
    }
}