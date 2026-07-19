pub mod protocol;
pub mod manager;

pub use protocol::{AgentCommand, AgentResponse, AgentInfo, AgentStatus, FileEntry};
pub use manager::AgentManager;
