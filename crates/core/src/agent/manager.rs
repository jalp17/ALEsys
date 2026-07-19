use super::protocol::{AgentCommand, AgentInfo, AgentResponse, AgentStatus};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, RwLock};
use std::sync::Arc;

pub struct AgentManager {
    agents: Arc<RwLock<HashMap<String, AgentConnection>>>,
    user_agents: Arc<RwLock<HashMap<i32, String>>>,
    pending: Arc<RwLock<HashMap<String, oneshot::Sender<AgentResponse>>>>,
    rate_limits: Arc<RwLock<HashMap<String, RateWindow>>>,
}

struct RateWindow {
    count: usize,
    window_start: Instant,
}

pub struct AgentConnection {
    pub info: AgentInfo,
    pub sender: mpsc::Sender<Vec<u8>>,
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
const RATE_LIMIT_MAX: usize = 100;

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            user_agents: Arc::new(RwLock::new(HashMap::new())),
            pending: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if an agent is within rate limits. Returns true if allowed.
    async fn check_rate_limit(&self, agent_id: &str) -> bool {
        let mut limits = self.rate_limits.write().await;
        let now = Instant::now();
        let entry = limits.entry(agent_id.to_string()).or_insert(RateWindow {
            count: 0,
            window_start: now,
        });

        if now.duration_since(entry.window_start) > RATE_LIMIT_WINDOW {
            entry.count = 1;
            entry.window_start = now;
            true
        } else if entry.count < RATE_LIMIT_MAX {
            entry.count += 1;
            true
        } else {
            false
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

        let mut pending = self.pending.write().await;
        pending.retain(|id, _| !id.starts_with(agent_id));

        let mut rate_limits = self.rate_limits.write().await;
        rate_limits.remove(agent_id);
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

    pub async fn send_command(
        &self,
        agent_id: &str,
        command: AgentCommand,
        timeout: Option<Duration>,
    ) -> Result<AgentResponse, String> {
        // Check rate limit before sending
        if !self.check_rate_limit(agent_id).await {
            return Err("Rate limit exceeded for agent".to_string());
        }

        let sender = {
            let agents = self.agents.read().await;
            agents.get(agent_id)
                .ok_or_else(|| format!("Agent '{}' not found", agent_id))?
                .sender.clone()
        };

        let msg_id = match &command {
            AgentCommand::Execute { id, .. } => id.clone(),
            AgentCommand::ReadFile { id, .. } => id.clone(),
            AgentCommand::WriteFile { id, .. } => id.clone(),
            AgentCommand::ListDirectory { id, .. } => id.clone(),
            AgentCommand::Ping => "ping".to_string(),
        };

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.write().await;
            pending.insert(msg_id.clone(), tx);
        }

        let data = serde_json::to_vec(&command)
            .map_err(|e| format!("Serialize error: {}", e))?;
        sender.send(data).await
            .map_err(|e| format!("Send error: {}", e))?;

        let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                let mut pending = self.pending.write().await;
                pending.remove(&msg_id);
                Err("Agent disconnected".to_string())
            }
            Err(_) => {
                let mut pending = self.pending.write().await;
                pending.remove(&msg_id);
                Err(format!("Timeout after {:?}", timeout))
            }
        }
    }

    pub async fn handle_response(&self, response: AgentResponse) {
        let id = match &response {
            AgentResponse::ExecuteResult { id, .. } => id.clone(),
            AgentResponse::FileContent { id, .. } => id.clone(),
            AgentResponse::DirectoryList { id, .. } => id.clone(),
            AgentResponse::Error { id, .. } => id.clone(),
            AgentResponse::Pong => return,
        };

        let mut pending = self.pending.write().await;
        if let Some(sender) = pending.remove(&id) {
            let _ = sender.send(response);
        }
    }

    /// Remove stale pending requests older than the given duration.
    /// Should be called periodically to prevent memory leaks from timed-out requests.
    pub async fn cleanup_stale_pending(&self, max_age: Duration) {
        // Since we can't track creation time in the current design,
        // we rely on the timeout in send_command. This is a safety net
        // for any orphaned entries.
        let mut pending = self.pending.write().await;
        let before = pending.len();
        // Clear all if there are orphaned entries (safety measure)
        if before > 1000 {
            pending.clear();
            tracing::warn!(
                "Cleared {} stale pending requests (safety threshold)",
                before
            );
        }
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

    #[tokio::test]
    async fn test_send_command_timeout() {
        let manager = AgentManager::new();
        let (tx, _rx) = mpsc::channel(10);

        let agent = create_test_agent("agent-1");
        manager.register_agent(agent, tx).await;

        let cmd = AgentCommand::Ping;
        let result = manager.send_command("agent-1", cmd, Some(Duration::from_millis(50))).await;
        assert!(result.is_err());
    }
}
