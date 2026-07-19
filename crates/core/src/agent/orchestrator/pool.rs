//! Agent Pool - Manages available agents with health checks

use super::decomposer::AgentType;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Agent instance in the pool
#[derive(Debug, Clone)]
pub struct PooledAgent {
    pub id: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub tasks_completed: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Idle,
    Busy,
    Offline,
    Error,
}

/// Pool of available agents
pub struct AgentPool {
    agents: RwLock<HashMap<String, PooledAgent>>,
}

impl AgentPool {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
        }
    }

    /// Register an agent in the pool
    pub async fn register(&self, id: String, agent_type: AgentType) {
        let agent = PooledAgent {
            id: id.clone(),
            agent_type,
            status: AgentStatus::Idle,
            current_task: None,
            tasks_completed: 0,
        };
        let mut agents = self.agents.write().await;
        agents.insert(id, agent);
    }

    /// Unregister an agent
    pub async fn unregister(&self, id: &str) {
        let mut agents = self.agents.write().await;
        agents.remove(id);
    }

    /// Get an idle agent of the specified type
    pub async fn get_idle(&self, agent_type: &AgentType) -> Option<String> {
        let agents = self.agents.read().await;
        agents
            .iter()
            .find(|(_, a)| a.agent_type == *agent_type && a.status == AgentStatus::Idle)
            .map(|(id, _)| id.clone())
    }

    /// Mark agent as busy
    pub async fn mark_busy(&self, id: &str, task_id: &str) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(id) {
            agent.status = AgentStatus::Busy;
            agent.current_task = Some(task_id.to_string());
        }
    }

    /// Mark agent as idle
    pub async fn mark_idle(&self, id: &str) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(id) {
            agent.status = AgentStatus::Idle;
            agent.current_task = None;
            agent.tasks_completed += 1;
        }
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let agents = self.agents.read().await;
        let total = agents.len();
        let idle = agents.values().filter(|a| a.status == AgentStatus::Idle).count();
        let busy = agents.values().filter(|a| a.status == AgentStatus::Busy).count();
        let total_completed: u64 = agents.values().map(|a| a.tasks_completed).sum();

        PoolStats {
            total,
            idle,
            busy,
            total_completed,
        }
    }

    /// List all agents
    pub async fn list(&self) -> Vec<PooledAgent> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }
}

impl Default for AgentPool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolStats {
    pub total: usize,
    pub idle: usize,
    pub busy: usize,
    pub total_completed: u64,
}
