//! Multi-Agent Orchestration System
//!
//! Coordinates multiple specialized agents to execute complex tasks in parallel.

pub mod orchestrator;
pub mod decomposer;
pub mod pool;
pub mod scheduler;

pub use orchestrator::{Orchestrator, OrchestratorTask, OrchestratorResult, TaskStatus};
pub use decomposer::{TaskDecomposer, Subtask, AgentType};
pub use pool::AgentPool;
pub use scheduler::TaskScheduler;
