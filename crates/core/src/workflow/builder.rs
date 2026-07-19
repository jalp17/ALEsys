use serde::{Deserialize, Serialize};
use super::actions::Action;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub action: Action,
    pub depends_on: Vec<String>,
}

pub struct WorkflowBuilder {
    workflow: Workflow,
}

impl WorkflowBuilder {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            workflow: Workflow {
                id: id.to_string(),
                name: name.to_string(),
                description: String::new(),
                steps: vec![],
                enabled: true,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.workflow.description = desc.to_string();
        self
    }

    pub fn step(mut self, id: &str, name: &str, action: Action) -> Self {
        self.workflow.steps.push(WorkflowStep {
            id: id.to_string(),
            name: name.to_string(),
            action,
            depends_on: vec![],
        });
        self
    }

    pub fn step_with_deps(mut self, id: &str, name: &str, action: Action, deps: Vec<String>) -> Self {
        self.workflow.steps.push(WorkflowStep {
            id: id.to_string(),
            name: name.to_string(),
            action,
            depends_on: deps,
        });
        self
    }

    pub fn build(self) -> Workflow {
        self.workflow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_build_workflow() {
        let workflow = WorkflowBuilder::new("w1", "Test Workflow")
            .description("A test workflow")
            .step("s1", "Step 1", make_action("a1"))
            .step("s2", "Step 2", make_action("a2"))
            .build();
        assert_eq!(workflow.steps.len(), 2);
        assert!(workflow.enabled);
    }

    #[test]
    fn test_step_with_deps() {
        let workflow = WorkflowBuilder::new("w1", "Test")
            .step("s1", "First", make_action("a1"))
            .step_with_deps("s2", "Second", make_action("a2"), vec!["s1".to_string()])
            .build();
        assert_eq!(workflow.steps[1].depends_on, vec!["s1"]);
    }

    #[test]
    fn test_empty_workflow() {
        let workflow = WorkflowBuilder::new("w1", "Empty").build();
        assert!(workflow.steps.is_empty());
    }
}