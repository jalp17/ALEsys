//! AI Pair Programmer Module
//!
//! Provides:
//! - Context analysis of the project
//! - Proactive suggestions for improvements
//! - Auto-refactoring capabilities
//! - Debug assistance
//! - Test generation

pub mod analyzer;
pub mod suggestions;
pub mod refactor;

pub use analyzer::ContextAnalyzer;
pub use suggestions::{SuggestionEngine, Suggestion, SuggestionType};
pub use refactor::AutoRefactor;
