//! Task Decomposer - Analyzes complex tasks and generates dependency graphs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of specialized agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType {
    Coder,
    Reviewer,
    Tester,
    Debugger,
}

impl AgentType {
    pub fn name(&self) -> &str {
        match self {
            AgentType::Coder => "coder",
            AgentType::Reviewer => "reviewer",
            AgentType::Tester => "tester",
            AgentType::Debugger => "debugger",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            AgentType::Coder => "Implements code based on specifications",
            AgentType::Reviewer => "Reviews code for bugs and improvements",
            AgentType::Tester => "Runs tests and analyzes coverage",
            AgentType::Debugger => "Analyzes errors and proposes fixes",
        }
    }
}

/// A subtask to be executed by a specialized agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: Uuid,
    pub agent_type: AgentType,
    pub command: String,
    pub args: Vec<String>,
    pub timeout_secs: u64,
    pub retries: u32,
    pub depends_on: Vec<Uuid>,
}

/// Decomposes complex tasks into subtask graphs
pub struct TaskDecomposer;

impl TaskDecomposer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a task description and decompose into subtasks
    pub fn decompose(&self, task_id: Uuid, description: &str) -> Vec<Subtask> {
        let mut subtasks = Vec::new();

        // Simple heuristic decomposition based on keywords
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("implement") || desc_lower.contains("create") || desc_lower.contains("build") {
            subtasks.push(Subtask {
                id: Uuid::new_v4(),
                agent_type: AgentType::Coder,
                command: "implement".to_string(),
                args: vec![description.to_string()],
                timeout_secs: 300,
                retries: 2,
                depends_on: vec![],
            });
        }

        if desc_lower.contains("review") || desc_lower.contains("audit") || desc_lower.contains("check") {
            let depends_on = subtasks.iter().map(|s| s.id).collect();
            subtasks.push(Subtask {
                id: Uuid::new_v4(),
                agent_type: AgentType::Reviewer,
                command: "review".to_string(),
                args: vec![description.to_string()],
                timeout_secs: 120,
                retries: 1,
                depends_on,
            });
        }

        if desc_lower.contains("test") || desc_lower.contains("verify") {
            let depends_on = subtasks.iter().map(|s| s.id).collect();
            subtasks.push(Subtask {
                id: Uuid::new_v4(),
                agent_type: AgentType::Tester,
                command: "test".to_string(),
                args: vec![description.to_string()],
                timeout_secs: 180,
                retries: 1,
                depends_on,
            });
        }

        if desc_lower.contains("debug") || desc_lower.contains("fix") || desc_lower.contains("error") {
            subtasks.push(Subtask {
                id: Uuid::new_v4(),
                agent_type: AgentType::Debugger,
                command: "debug".to_string(),
                args: vec![description.to_string()],
                timeout_secs: 240,
                retries: 3,
                depends_on: vec![],
            });
        }

        // If no specific keywords found, default to coder
        if subtasks.is_empty() {
            subtasks.push(Subtask {
                id: Uuid::new_v4(),
                agent_type: AgentType::Coder,
                command: "process".to_string(),
                args: vec![description.to_string()],
                timeout_secs: 300,
                retries: 2,
                depends_on: vec![],
            });
        }

        subtasks
    }

    /// Check if a subtask can run (all dependencies satisfied)
    pub fn can_run(&self, subtask: &Subtask, completed: &[Uuid]) -> bool {
        subtask.depends_on.iter().all(|dep| completed.contains(dep))
    }
}

impl Default for TaskDecomposer {
    fn default() -> Self {
        Self::new()
    }
}
