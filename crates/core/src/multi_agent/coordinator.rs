use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub current_task: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationResult {
    pub task_id: String,
    pub assigned_agents: Vec<String>,
    pub strategy: String,
    pub estimated_duration_ms: u64,
    pub success: bool,
    pub warnings: Vec<String>,
}

pub struct AgentCoordinator {
    agents: HashMap<String, AgentInfo>,
}

impl AgentCoordinator {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register_agent(&mut self, agent: AgentInfo) {
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn get_agent(&self, id: &str) -> Option<&AgentInfo> {
        self.agents.get(id)
    }

    pub fn list_agents(&self) -> Vec<&AgentInfo> {
        self.agents.values().collect()
    }

    pub fn find_available_agents(&self, capability: &str) -> Vec<&AgentInfo> {
        self.agents
            .values()
            .filter(|a| a.status == "idle" && a.capabilities.iter().any(|c| c == capability))
            .collect()
    }

    pub fn coordinate_task(&self, task_id: &str, required_capabilities: &[String]) -> CoordinationResult {
        let mut assigned = Vec::new();
        let mut warnings = Vec::new();

        for cap in required_capabilities {
            let available: Vec<&AgentInfo> = self.agents
                .values()
                .filter(|a| a.status == "idle" && a.capabilities.iter().any(|c| c == cap.as_str()))
                .collect();

            if available.is_empty() {
                warnings.push(format!("No available agent for capability: {}", cap));
            } else {
                assigned.push(available[0].id.clone());
            }
        }

        assigned.sort();
        assigned.dedup();

        CoordinationResult {
            task_id: task_id.to_string(),
            assigned_agents: assigned,
            strategy: "capability-matching".to_string(),
            estimated_duration_ms: 5000,
            success: warnings.is_empty(),
            warnings,
        }
    }

    pub fn get_stats(&self) -> CoordinationStats {
        let total = self.agents.len();
        let idle = self.agents.values().filter(|a| a.status == "idle").count();
        let busy = self.agents.values().filter(|a| a.status == "busy").count();

        CoordinationStats {
            total_agents: total,
            idle_agents: idle,
            busy_agents: busy,
            capabilities: self.agents.values().flat_map(|a| a.capabilities.clone()).collect::<std::collections::HashSet<_>>().len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationStats {
    pub total_agents: usize,
    pub idle_agents: usize,
    pub busy_agents: usize,
    pub capabilities: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(id: &str, caps: Vec<&str>) -> AgentInfo {
        AgentInfo {
            id: id.to_string(),
            name: format!("Agent {}", id),
            capabilities: caps.into_iter().map(String::from).collect(),
            status: "idle".to_string(),
            current_task: None,
        }
    }

    #[test]
    fn test_register_and_list() {
        let mut coord = AgentCoordinator::new();
        coord.register_agent(make_agent("a1", vec!["code", "test"]));
        coord.register_agent(make_agent("a2", vec!["review"]));
        assert_eq!(coord.list_agents().len(), 2);
    }

    #[test]
    fn test_find_available() {
        let mut coord = AgentCoordinator::new();
        coord.register_agent(make_agent("a1", vec!["code"]));
        let available = coord.find_available_agents("code");
        assert_eq!(available.len(), 1);
    }

    #[test]
    fn test_coordinate_success() {
        let mut coord = AgentCoordinator::new();
        coord.register_agent(make_agent("a1", vec!["code"]));
        coord.register_agent(make_agent("a2", vec!["test"]));
        let result = coord.coordinate_task("t1", &["code".to_string(), "test".to_string()]);
        assert!(result.success);
        assert_eq!(result.assigned_agents.len(), 2);
    }

    #[test]
    fn test_coordinate_missing() {
        let coord = AgentCoordinator::new();
        let result = coord.coordinate_task("t1", &["code".to_string()]);
        assert!(!result.success);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_stats() {
        let mut coord = AgentCoordinator::new();
        coord.register_agent(make_agent("a1", vec!["code"]));
        coord.register_agent(make_agent("a2", vec!["test"]));
        let stats = coord.get_stats();
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.idle_agents, 2);
    }
}