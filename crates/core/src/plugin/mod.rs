//! Plugin System for ALEsys
//!
//! Provides a sandboxed, extensible plugin architecture with:
//! - Dynamic loading of plugin binaries
//! - Granular permissions (filesystem, network, execute, database)
//! - Lifecycle hooks (init, execute, shutdown)
//! - Plugin registry with versioning
//! - Security sandbox with timeout

pub mod api;
pub mod manager;
pub mod registry;
pub mod security;

#[cfg(test)]
mod tests;

pub use api::{Plugin, PluginContext, PluginMetadata, PluginPermission, PluginResult};
pub use manager::PluginManager;
pub use registry::PluginRegistry;
pub use security::SecuritySandbox;
