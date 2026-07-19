use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Review,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assigned_agents: Vec<String>,
    pub dependencies: Vec<String>,
    pub result: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct TaskBoard {
    tasks: Vec<Task>,
}

impl TaskBoard {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }

    pub fn create_task(&mut self, id: &str, title: &str, description: &str, priority: TaskPriority) -> Task {
        let task = Task {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            priority,
            assigned_agents: vec![],
            dependencies: vec![],
            result: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        self.tasks.push(task.clone());
        task
    }

    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn update_status(&mut self, task_id: &str, status: TaskStatus) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = status;
            task.updated_at = chrono::Utc::now().to_rfc3339();
            true
        } else {
            false
        }
    }

    pub fn assign_agent(&mut self, task_id: &str, agent_id: &str) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if !task.assigned_agents.contains(&agent_id.to_string()) {
                task.assigned_agents.push(agent_id.to_string());
                task.updated_at = chrono::Utc::now().to_rfc3339();
            }
            true
        } else {
            false
        }
    }

    pub fn add_dependency(&mut self, task_id: &str, dependency_id: &str) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if !task.dependencies.contains(&dependency_id.to_string()) {
                task.dependencies.push(dependency_id.to_string());
            }
            true
        } else {
            false
        }
    }

    pub fn set_result(&mut self, task_id: &str, result: &str) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.result = Some(result.to_string());
            task.status = TaskStatus::Done;
            task.updated_at = chrono::Utc::now().to_rfc3339();
            true
        } else {
            false
        }
    }

    pub fn list_tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn get_tasks_by_status(&self, status: &TaskStatus) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.status == *status).collect()
    }

    pub fn get_tasks_by_priority(&self, priority: &TaskPriority) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.priority == *priority).collect()
    }

    pub fn get_ready_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|t| {
            t.status == TaskStatus::Pending && t.dependencies.iter().all(|dep_id| {
                self.tasks.iter().any(|d| d.id == *dep_id && d.status == TaskStatus::Done)
            })
        }).collect()
    }

    pub fn get_stats(&self) -> BoardStats {
        BoardStats {
            total: self.tasks.len(),
            pending: self.get_tasks_by_status(&TaskStatus::Pending).len(),
            in_progress: self.get_tasks_by_status(&TaskStatus::InProgress).len(),
            done: self.get_tasks_by_status(&TaskStatus::Done).len(),
            failed: self.get_tasks_by_status(&TaskStatus::Failed).len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardStats {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub done: usize,
    pub failed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let mut board = TaskBoard::new();
        let task = board.create_task("t1", "Test Task", "Description", TaskPriority::High);
        assert_eq!(task.id, "t1");
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_update_status() {
        let mut board = TaskBoard::new();
        board.create_task("t1", "Test", "Desc", TaskPriority::Medium);
        assert!(board.update_status("t1", TaskStatus::InProgress));
        assert_eq!(board.get_task("t1").unwrap().status, TaskStatus::InProgress);
    }

    #[test]
    fn test_assign_agent() {
        let mut board = TaskBoard::new();
        board.create_task("t1", "Test", "Desc", TaskPriority::Medium);
        assert!(board.assign_agent("t1", "agent-1"));
        assert!(board.get_task("t1").unwrap().assigned_agents.contains(&"agent-1".to_string()));
    }

    #[test]
    fn test_dependencies() {
        let mut board = TaskBoard::new();
        board.create_task("t1", "Dep 1", "Desc", TaskPriority::High);
        board.create_task("t2", "Main Task", "Desc", TaskPriority::High);
        board.add_dependency("t2", "t1");
        assert_eq!(board.get_ready_tasks().len(), 1);
        board.update_status("t1", TaskStatus::Done);
        assert_eq!(board.get_ready_tasks().len(), 1);
    }

    #[test]
    fn test_stats() {
        let mut board = TaskBoard::new();
        board.create_task("t1", "Task 1", "Desc", TaskPriority::Low);
        board.create_task("t2", "Task 2", "Desc", TaskPriority::High);
        board.update_status("t1", TaskStatus::Done);
        let stats = board.get_stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.done, 1);
    }
}