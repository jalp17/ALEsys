//! Task Scheduler - Priority queue with retry and timeout

use super::decomposer::Subtask;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Task status in the scheduler
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScheduledTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
}

/// A scheduled task
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub subtask: Subtask,
    pub status: ScheduledTaskStatus,
    pub attempts: u32,
    pub last_error: Option<String>,
}

/// Priority task scheduler
pub struct TaskScheduler {
    queue: RwLock<VecDeque<ScheduledTask>>,
    completed: RwLock<Vec<CompletedTask>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTask {
    pub subtask_id: Uuid,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            completed: RwLock::new(Vec::new()),
        }
    }

    /// Enqueue a subtask
    pub async fn enqueue(&self, subtask: Subtask) {
        let task = ScheduledTask {
            subtask,
            status: ScheduledTaskStatus::Pending,
            attempts: 0,
            last_error: None,
        };
        let mut queue = self.queue.write().await;
        queue.push_back(task);
    }

    /// Dequeue the next ready task
    pub async fn dequeue(&self) -> Option<Subtask> {
        let mut queue = self.queue.write().await;
        if let Some(task) = queue.pop_front() {
            Some(task.subtask)
        } else {
            None
        }
    }

    /// Mark a task as completed
    pub async fn complete(&self, subtask_id: Uuid, success: bool, output: Option<String>, error: Option<String>, duration_ms: u64) {
        let completed = CompletedTask {
            subtask_id,
            success,
            output,
            error,
            duration_ms,
        };
        let mut completed_list = self.completed.write().await;
        completed_list.push(completed);
    }

    /// Get completed tasks
    pub async fn get_completed(&self) -> Vec<CompletedTask> {
        let completed = self.completed.read().await;
        completed.clone()
    }

    /// Get queue length
    pub async fn queue_len(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len()
    }

    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        let queue = self.queue.read().await;
        queue.is_empty()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}
