//! Operational Transform Engine

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpAction {
    Insert,
    Delete,
    Retain,
}

/// A single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub user_id: String,
    pub position: usize,
    pub action: OpAction,
    pub content: Option<String>,
    pub length: Option<usize>,
}

/// OT Engine for conflict resolution
pub struct OTEngine {
    history: Vec<Operation>,
}

impl OTEngine {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Apply an operation
    pub fn apply(&mut self, op: Operation) -> Result<(), String> {
        // Validate operation
        if op.position > self.get_document_length() {
            return Err("Position out of bounds".to_string());
        }

        self.history.push(op);
        Ok(())
    }

    /// Transform two concurrent operations
    pub fn transform(&self, op1: &Operation, op2: &Operation) -> (Operation, Operation) {
        // Simple OT implementation
        // In production, use a proper OT library
        let mut transformed_op1 = op1.clone();
        let mut transformed_op2 = op2.clone();

        if op1.position < op2.position {
            // op1 is before op2, no transformation needed
        } else if op1.position == op2.position {
            // Same position - use user_id for deterministic ordering
            if op1.user_id > op2.user_id {
                match op1.action {
                    OpAction::Insert => {
                        transformed_op2.position += op1.content.as_ref().map_or(0, |s| s.len());
                    }
                    OpAction::Delete => {
                        transformed_op2.position = transformed_op2.position.saturating_sub(1);
                    }
                    _ => {}
                }
            } else {
                match op2.action {
                    OpAction::Insert => {
                        transformed_op1.position += op2.content.as_ref().map_or(0, |s| s.len());
                    }
                    OpAction::Delete => {
                        transformed_op1.position = transformed_op1.position.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        } else {
            // op1 is after op2
            match op2.action {
                OpAction::Insert => {
                    transformed_op1.position += op2.content.as_ref().map_or(0, |s| s.len());
                }
                OpAction::Delete => {
                    transformed_op1.position = transformed_op1.position.saturating_sub(
                        op2.length.unwrap_or(1)
                    );
                }
                _ => {}
            }
        }

        (transformed_op1, transformed_op2)
    }

    /// Get current document length
    fn get_document_length(&self) -> usize {
        let mut length = 0;
        for op in &self.history {
            match op.action {
                OpAction::Insert => {
                    length += op.content.as_ref().map_or(0, |s| s.len());
                }
                OpAction::Delete => {
                    length = length.saturating_sub(op.length.unwrap_or(1));
                }
                OpAction::Retain => {}
            }
        }
        length
    }

    /// Get operation history
    pub fn history(&self) -> &[Operation] {
        &self.history
    }
}

impl Default for OTEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_insert() {
        let mut engine = OTEngine::new();
        let op = Operation {
            id: Uuid::new_v4(),
            user_id: "user1".to_string(),
            position: 0,
            action: OpAction::Insert,
            content: Some("Hello".to_string()),
            length: None,
        };
        assert!(engine.apply(op).is_ok());
        assert_eq!(engine.history().len(), 1);
    }

    #[test]
    fn test_apply_out_of_bounds() {
        let mut engine = OTEngine::new();
        let op = Operation {
            id: Uuid::new_v4(),
            user_id: "user1".to_string(),
            position: 100,
            action: OpAction::Insert,
            content: Some("Hello".to_string()),
            length: None,
        };
        assert!(engine.apply(op).is_err());
    }

    #[test]
    fn test_transform_concurrent_inserts() {
        let engine = OTEngine::new();
        let op1 = Operation {
            id: Uuid::new_v4(),
            user_id: "user1".to_string(),
            position: 0,
            action: OpAction::Insert,
            content: Some("A".to_string()),
            length: None,
        };
        let op2 = Operation {
            id: Uuid::new_v4(),
            user_id: "user2".to_string(),
            position: 0,
            action: OpAction::Insert,
            content: Some("B".to_string()),
            length: None,
        };

        let (t1, t2) = engine.transform(&op1, &op2);
        // User with higher user_id gets transformed
        assert_eq!(t1.position, 1); // user2 > user1, so op1 gets shifted
        assert_eq!(t2.position, 0);
    }
}
