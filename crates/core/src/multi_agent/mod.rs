pub mod coordinator;
pub mod task_board;
pub mod communication;
pub mod consensus;

pub use coordinator::{AgentCoordinator, CoordinationResult};
pub use task_board::{TaskBoard, Task, TaskStatus, TaskPriority};
pub use communication::{AgentMessageBus, Message, MessageType};
pub use consensus::{ConsensusEngine, ConsensusResult, Vote};
