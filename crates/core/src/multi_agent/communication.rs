use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    TaskAssignment,
    TaskUpdate,
    Request,
    Response,
    Broadcast,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub message_type: MessageType,
    pub content: String,
    pub timestamp: String,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct AgentMessageBus {
    messages: Vec<Message>,
}

impl AgentMessageBus {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    pub fn send(&mut self, from: &str, to: &str, msg_type: MessageType, content: &str) -> Message {
        let msg = Message {
            id: format!("msg-{}", self.messages.len()),
            from: from.to_string(),
            to: to.to_string(),
            message_type: msg_type,
            content: content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: std::collections::HashMap::new(),
        };
        self.messages.push(msg.clone());
        msg
    }

    pub fn get_messages_for(&self, agent_id: &str) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.to == agent_id || m.to == "*").collect()
    }

    pub fn get_messages_from(&self, agent_id: &str) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.from == agent_id).collect()
    }

    pub fn get_conversation(&self, agent_a: &str, agent_b: &str) -> Vec<&Message> {
        self.messages.iter().filter(|m| {
            (m.from == agent_a && m.to == agent_b) || (m.from == agent_b && m.to == agent_a)
        }).collect()
    }

    pub fn broadcast(&mut self, from: &str, content: &str) -> Vec<Message> {
        let mut msgs = vec![];
        let msg = self.send(from, "*", MessageType::Broadcast, content);
        msgs.push(msg);
        msgs
    }

    pub fn get_stats(&self) -> MessageStats {
        let mut by_type = std::collections::HashMap::new();
        for msg in &self.messages {
            *by_type.entry(format!("{:?}", msg.message_type)).or_insert(0) += 1;
        }
        MessageStats {
            total_messages: self.messages.len(),
            by_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    pub total_messages: usize,
    pub by_type: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_message() {
        let mut bus = AgentMessageBus::new();
        let msg = bus.send("agent-1", "agent-2", MessageType::Request, "Do something");
        assert_eq!(msg.from, "agent-1");
        assert_eq!(msg.to, "agent-2");
    }

    #[test]
    fn test_get_messages_for() {
        let mut bus = AgentMessageBus::new();
        bus.send("a1", "a2", MessageType::Request, "msg1");
        bus.send("a1", "a3", MessageType::Request, "msg2");
        let msgs = bus.get_messages_for("a2");
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_conversation() {
        let mut bus = AgentMessageBus::new();
        bus.send("a1", "a2", MessageType::Request, "hello");
        bus.send("a2", "a1", MessageType::Response, "hi");
        bus.send("a1", "a3", MessageType::Request, "other");
        let conv = bus.get_conversation("a1", "a2");
        assert_eq!(conv.len(), 2);
    }

    #[test]
    fn test_broadcast() {
        let mut bus = AgentMessageBus::new();
        let msgs = bus.broadcast("a1", "urgent update");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].to, "*");
    }

    #[test]
    fn test_stats() {
        let mut bus = AgentMessageBus::new();
        bus.send("a1", "a2", MessageType::Request, "msg");
        bus.send("a1", "a2", MessageType::Response, "reply");
        let stats = bus.get_stats();
        assert_eq!(stats.total_messages, 2);
    }
}