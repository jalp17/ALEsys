//! Orchestrator - Coordinates multiple agents for complex tasks

use super::decomposer::{Subtask, TaskDecomposer};
use super::pool::AgentPool;
use super::scheduler::{CompletedTask, TaskScheduler};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Status of an orchestrator task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    PartiallyCompleted,
}

/// A complex task to be orchestrated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorTask {
    pub id: Uuid,
    pub description: String,
    pub status: TaskStatus,
    pub subtasks: Vec<Subtask>,
}

/// Result of an orchestrated task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorResult {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub subtask_results: Vec<CompletedTask>,
    pub summary: String,
}

/// Orchestrates multiple agents for complex tasks
pub struct Orchestrator {
    decomposer: TaskDecomposer,
    pool: AgentPool,
    scheduler: TaskScheduler,
    active_tasks: RwLock<Vec<OrchestratorTask>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            decomposer: TaskDecomposer::new(),
            pool: AgentPool::new(),
            scheduler: TaskScheduler::new(),
            active_tasks: RwLock::new(Vec::new()),
        }
    }

    /// Submit a complex task for orchestration
    pub async fn submit_task(&self, description: String) -> Uuid {
        let task_id = Uuid::new_v4();
        let subtasks = self.decomposer.decompose(task_id, &description);

        let task = OrchestratorTask {
            id: task_id,
            description,
            status: TaskStatus::Pending,
            subtasks: subtasks.clone(),
        };

        // Add to active tasks
        let mut active = self.active_tasks.write().await;
        active.push(task);

        // Enqueue subtasks
        for subtask in subtasks {
            self.scheduler.enqueue(subtask).await;
        }

        task_id
    }

    /// Execute the orchestrator loop
    pub async fn run(&self) -> Result<OrchestratorResult, String> {
        let mut completed_ids: Vec<Uuid> = Vec::new();
        let mut results: Vec<CompletedTask> = Vec::new();

        // Process tasks until queue is empty
        while !self.scheduler.is_empty().await {
            if let Some(subtask) = self.scheduler.dequeue().await {
                // Check dependencies
                if !self.decomposer.can_run(&subtask, &completed_ids) {
                    // Re-enqueue and try later
                    self.scheduler.enqueue(subtask).await;
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }

                // Find available agent
                let agent_id = self.pool.get_idle(&subtask.agent_type).await;
                if let Some(agent_id) = agent_id {
                    // Mark agent as busy
                    self.pool.mark_busy(&agent_id, &subtask.id.to_string()).await;

                    // Execute subtask
                    let start = std::time::Instant::now();
                    let (success, output, error) = self.execute_subtask(&subtask).await;
                    let duration = start.elapsed().as_millis() as u64;

                    // Record result
                    let completed = CompletedTask {
                        subtask_id: subtask.id,
                        success,
                        output: output.clone(),
                        error: error.clone(),
                        duration_ms: duration,
                    };
                    results.push(completed.clone());
                    self.scheduler.complete(subtask.id, success, output, error, duration).await;

                    if success {
                        completed_ids.push(subtask.id);
                    }

                    // Mark agent as idle
                    self.pool.mark_idle(&agent_id).await;
                } else {
                    // No agent available, re-enqueue
                    self.scheduler.enqueue(subtask).await;
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }

        let all_success = results.iter().all(|r| r.success);
        let status = if all_success {
            TaskStatus::Completed
        } else if results.iter().any(|r| r.success) {
            TaskStatus::PartiallyCompleted
        } else {
            TaskStatus::Failed
        };

        let summary = format!(
            "Task completed: {}/{} subtasks successful",
            results.iter().filter(|r| r.success).count(),
            results.len()
        );

        Ok(OrchestratorResult {
            task_id: Uuid::new_v4(),
            status,
            subtask_results: results,
            summary,
        })
    }

    /// Execute a single subtask (stub implementation)
    async fn execute_subtask(&self, subtask: &Subtask) -> (bool, Option<String>, Option<String>) {
        // In production, this would dispatch to the appropriate agent
        // For now, return a stub result
        let output = format!(
            "Executed {:?} task: {}",
            subtask.agent_type,
            subtask.command
        );
        (true, Some(output), None)
    }

    /// Get pool reference
    pub fn pool(&self) -> &AgentPool {
        &self.pool
    }

    /// Get scheduler reference
    pub fn scheduler(&self) -> &TaskScheduler {
        &self.scheduler
    }

    /// Get active tasks
    pub async fn active_tasks(&self) -> Vec<OrchestratorTask> {
        let active = self.active_tasks.read().await;
        active.clone()
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_task() {
        let orchestrator = Orchestrator::new();
        let task_id = orchestrator
            .submit_task("Implement a new feature".to_string())
            .await;

        let tasks = orchestrator.active_tasks().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task_id);
    }

    #[tokio::test]
    async fn test_pool_register() {
        let orchestrator = Orchestrator::new();
        orchestrator
            .pool()
            .register("agent-1".to_string(), super::super::decomposer::AgentType::Coder)
            .await;

        let stats = orchestrator.pool().stats().await;
        assert_eq!(stats.total, 1);
        assert_eq!(stats.idle, 1);
    }
}
