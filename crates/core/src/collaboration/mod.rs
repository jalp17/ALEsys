//! Real-Time Collaboration Module
//!
//! Provides:
//! - WebSocket rooms for collaborative editing
//! - Operational Transform (OT) for conflict resolution
//! - Presence system with cursor sync
//! - Shared terminal

pub mod ot;
pub mod presence;
pub mod room;

pub use ot::{Operation, OpAction, OTEngine};
pub use presence::{Presence, UserStatus, PresenceManager};
pub use room::{CollabRoom, CollabMessage, CollabPayload};
