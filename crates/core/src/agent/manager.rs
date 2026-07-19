use super::protocol::{AgentInfo, AgentStatus};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

pub struct AgentManager {
    agents: Arc<RwLock<HashMap<String, AgentConnection>>>,
    user_agents: Arc<RwLock<HashMap<i32, String>>>,
}

pub struct AgentConnection {
    pub info: AgentInfo,
    pub sender: mpsc::Sender<Vec<u8>>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            user_agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_agent(&self, info: AgentInfo, sender: mpsc::Sender<Vec<u8>>) {
        let mut agents = self.agents.write().await;
        agents.insert(
            info.id.clone(),
            AgentConnection { info, sender },
        );
    }

    pub async fn unregister_agent(&self, agent_id: &str) {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id);

        let mut user_agents = self.user_agents.write().await;
        user_agents.retain(|_, id| id != agent_id);
    }

    pub async fn assign_agent_to_user(&self, user_id: i32, agent_id: &str) -> bool {
        let agents = self.agents.read().await;
        if !agents.contains_key(agent_id) {
            return false;
        }
        let mut user_agents = self.user_agents.write().await;
        user_agents.insert(user_id, agent_id.to_string());
        true
    }

    pub async fn get_agent_for_user(&self, user_id: i32) -> Option<AgentInfo> {
        let user_agents = self.user_agents.read().await;
        let agent_id = user_agents.get(&user_id)?;
        let agents = self.agents.read().await;
        agents.get(agent_id).map(|c| c.info.clone())
    }

    pub async fn list_agents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        agents.values().map(|c| c.info.clone()).collect()
    }

    pub async fn get_agent(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        agents.get(agent_id).map(|c| c.info.clone())
    }

    pub async fn get_agent_count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }

    pub async fn get_connected_count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.values().filter(|c| c.info.status == AgentStatus::Connected).count()
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_agent(id: &str) -> AgentInfo {
        AgentInfo {
            id: id.to_string(),
            name: format!("agent-{}", id),
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            status: AgentStatus::Connected,
            connected_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_register_and_list() {
        let manager = AgentManager::new();
        let (tx, _rx) = mpsc::channel(10);

        let agent = create_test_agent("agent-1");
        manager.register_agent(agent, tx).await;

        let agents = manager.list_agents().await;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "agent-1");
    }

    #[tokio::test]
    async fn test_unregister() {
        let manager = AgentManager::new();
        let (tx, _rx) = mpsc::channel(10);

        let agent = create_test_agent("agent-1");
        manager.register_agent(agent, tx).await;
        manager.unregister_agent("agent-1").await;

        let agents = manager.list_agents().await;
        assert_eq!(agents.len(), 0);
    }

    #[tokio::test]
    async fn test_assign_to_user() {
        let manager = AgentManager::new();
        let (tx, _rx) = mpsc::channel(10);

        let agent = create_test_agent("agent-1");
        manager.register_agent(agent, tx).await;

        assert!(manager.assign_agent_to_user(1, "agent-1").await);
        assert!(!manager.assign_agent_to_user(1, "nonexistent").await);

        let assigned = manager.get_agent_for_user(1).await;
        assert!(assigned.is_some());
        assert_eq!(assigned.unwrap().id, "agent-1");
    }
}
