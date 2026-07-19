use serde::{Deserialize, Serialize};
use super::analyzer::{CodeBlock, RefactoringOpportunity, OpportunityType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringResult {
    pub original_code: String,
    pub transformed_code: String,
    pub changes: Vec<Change>,
    pub success: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub change_type: ChangeType,
    pub description: String,
    pub start_line: usize,
    pub end_line: usize,
    pub original: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChangeType {
    ExtractFunction,
    Rename,
    Inline,
    Simplify,
    Remove,
    Reorder,
}

pub struct Transformer;

impl Transformer {
    pub fn new() -> Self {
        Self
    }

    pub fn apply_refactoring(
        &self,
        code: &str,
        opportunity: &RefactoringOpportunity,
        blocks: &[CodeBlock],
    ) -> RefactoringResult {
        match opportunity.opportunity_type {
            OpportunityType::ExtractFunction => self.extract_function(code, opportunity, blocks),
            OpportunityType::RenameSymbol => self.rename_symbol(code, opportunity, blocks),
            OpportunityType::InlineFunction => self.inline_function(code, opportunity, blocks),
            OpportunityType::SimplifyConditional => self.simplify_conditional(code, opportunity),
            OpportunityType::RemoveDeadCode => self.remove_dead_code(code, opportunity),
            OpportunityType::DeduplicateCode => self.deduplicate_code(code, opportunity, blocks),
            _ => RefactoringResult {
                original_code: code.to_string(),
                transformed_code: code.to_string(),
                changes: vec![],
                success: false,
                errors: vec!["Refactoring type not implemented".to_string()],
            },
        }
    }

    fn extract_function(
        &self,
        code: &str,
        opportunity: &RefactoringOpportunity,
        blocks: &[CodeBlock],
    ) -> RefactoringResult {
        let mut lines: Vec<String> = code.lines().map(|l| l.to_string()).collect();
        let mut changes = Vec::new();
        let mut errors = Vec::new();

        if let Some(block_id) = opportunity.affected_blocks.first() {
            if let Some(block) = blocks.iter().find(|b| &b.id == block_id) {
                let start = block.start_line.saturating_sub(1);
                let end = block.end_line.min(lines.len());

                if start < lines.len() && end <= lines.len() {
                    let extracted_code: String = lines[start..end].join("\n");
                    let new_fn_name = format!("extracted_{}", block.name);

                    let function_def = format!(
                        "fn {}() {{\n{}\n}}\n",
                        new_fn_name,
                        extracted_code
                    );

                    let call_line = format!("{}();", new_fn_name);
                    let original = extracted_code.clone();

                    lines.drain(start..end);
                    lines.insert(start, call_line);

                    changes.push(Change {
                        change_type: ChangeType::ExtractFunction,
                        description: format!("Extracted function '{}' from '{}'", new_fn_name, block.name),
                        start_line: start + 1,
                        end_line: end,
                        original,
                        replacement: function_def,
                    });
                } else {
                    errors.push("Block out of bounds".to_string());
                }
            }
        }

        let transformed = if errors.is_empty() {
            lines.join("\n")
        } else {
            code.to_string()
        };

        RefactoringResult {
            original_code: code.to_string(),
            transformed_code: transformed,
            changes,
            success: errors.is_empty(),
            errors,
        }
    }

    fn rename_symbol(
        &self,
        code: &str,
        opportunity: &RefactoringOpportunity,
        blocks: &[CodeBlock],
    ) -> RefactoringResult {
        let mut transformed = code.to_string();
        let mut changes = Vec::new();

        if let Some(block_id) = opportunity.affected_blocks.first() {
            if let Some(block) = blocks.iter().find(|b| &b.id == block_id) {
                let new_name = format!("{}_renamed", block.name);
                let count = transformed.matches(&block.name).count();

                transformed = transformed.replace(&block.name, &new_name);

                if count > 0 {
                    changes.push(Change {
                        change_type: ChangeType::Rename,
                        description: format!("Renamed '{}' to '{}' ({} occurrences)", block.name, new_name, count),
                        start_line: block.start_line,
                        end_line: block.end_line,
                        original: block.name.clone(),
                        replacement: new_name,
                    });
                }
            }
        }

        let has_changes = !changes.is_empty();
        RefactoringResult {
            original_code: code.to_string(),
            transformed_code: transformed,
            changes,
            success: has_changes,
            errors: vec![],
        }
    }

    fn inline_function(
        &self,
        code: &str,
        opportunity: &RefactoringOpportunity,
        blocks: &[CodeBlock],
    ) -> RefactoringResult {
        let mut lines: Vec<String> = code.lines().map(|l| l.to_string()).collect();
        let mut changes = Vec::new();

        if let Some(block_id) = opportunity.affected_blocks.first() {
            if let Some(block) = blocks.iter().find(|b| &b.id == block_id) {
                let start = block.start_line.saturating_sub(1);
                let end = block.end_line.min(lines.len());

                if start < lines.len() && end <= lines.len() {
                    let inlined_body = lines[start + 1..end - 1].join("\n");
                    let call_pattern = format!("{}();", block.name);

                    for line in lines.iter_mut() {
                        if line.trim() == call_pattern.trim() {
                            *line = inlined_body.clone();
                        }
                    }

                    changes.push(Change {
                        change_type: ChangeType::Inline,
                        description: format!("Inlined function '{}'", block.name),
                        start_line: start + 1,
                        end_line: end,
                        original: call_pattern,
                        replacement: inlined_body,
                    });
                }
            }
        }

        let has_changes = !changes.is_empty();
        RefactoringResult {
            original_code: code.to_string(),
            transformed_code: lines.join("\n"),
            changes,
            success: has_changes,
            errors: vec![],
        }
    }

    fn simplify_conditional(
        &self,
        code: &str,
        _opportunity: &RefactoringOpportunity,
    ) -> RefactoringResult {
        let mut transformed = code.to_string();
        let mut changes = Vec::new();

        let patterns = vec![
            "if true {",
            "if false {",
            "if 1 == 1 {",
            "if 0 == 0 {",
        ];

        for pattern in patterns {
            if transformed.contains(pattern) {
                transformed = transformed.replace(pattern, "// Simplified: ");
                changes.push(Change {
                    change_type: ChangeType::Simplify,
                    description: format!("Simplified redundant conditional: {}", pattern),
                    start_line: 0,
                    end_line: 0,
                    original: pattern.to_string(),
                    replacement: "// Simplified: ".to_string(),
                });
            }
        }

        let has_changes = !changes.is_empty();
        RefactoringResult {
            original_code: code.to_string(),
            transformed_code: transformed,
            changes,
            success: has_changes,
            errors: vec![],
        }
    }

    fn remove_dead_code(
        &self,
        code: &str,
        _opportunity: &RefactoringOpportunity,
    ) -> RefactoringResult {
        let mut transformed = code.to_string();
        let mut changes = Vec::new();

        let dead_patterns = vec![
            "// TODO:",
            "// FIXME:",
            "// HACK:",
            "// XXX:",
        ];

        for pattern in dead_patterns {
            if transformed.contains(pattern) {
                let filtered: Vec<&str> = transformed.lines()
                    .filter(|l| !l.trim().starts_with(pattern))
                    .collect();
                transformed = filtered.join("\n");
                changes.push(Change {
                    change_type: ChangeType::Remove,
                    description: format!("Removed dead code with pattern: {}", pattern),
                    start_line: 0,
                    end_line: 0,
                    original: pattern.to_string(),
                    replacement: String::new(),
                });
            }
        }

        let has_changes = !changes.is_empty();
        RefactoringResult {
            original_code: code.to_string(),
            transformed_code: transformed,
            changes,
            success: has_changes,
            errors: vec![],
        }
    }

    fn deduplicate_code(
        &self,
        code: &str,
        _opportunity: &RefactoringOpportunity,
        blocks: &[CodeBlock],
    ) -> RefactoringResult {
        let mut transformed = code.to_string();
        let mut changes = Vec::new();

        if blocks.len() >= 2 {
            let mut seen = std::collections::HashSet::new();
            for block in blocks {
                if !seen.insert(block.content.clone()) && block.content.len() > 50 {
                    let placeholder = format!("// Deduplicated: {}", block.name);
                    transformed = transformed.replace(&block.content, &placeholder);
                    changes.push(Change {
                        change_type: ChangeType::Remove,
                        description: format!("Deduplicated code block '{}'", block.name),
                        start_line: block.start_line,
                        end_line: block.end_line,
                        original: block.content.clone(),
                        replacement: placeholder,
                    });
                }
            }
        }

        let has_changes = !changes.is_empty();
        RefactoringResult {
            original_code: code.to_string(),
            transformed_code: transformed,
            changes,
            success: has_changes,
            errors: vec![],
        }
    }
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::analyzer::{OpportunityType, ImpactLevel, BlockType};

    #[test]
    fn test_rename_symbol() {
        let transformer = Transformer::new();
        let code = "fn old_name() {\n    println!(\"hello\");\n}";
        let blocks = vec![CodeBlock {
            id: "fn_0".to_string(),
            block_type: BlockType::Function,
            name: "old_name".to_string(),
            start_line: 1,
            end_line: 3,
            content: code.to_string(),
            complexity: 1,
            dependencies: vec![],
            children: vec![],
        }];
        let opportunity = RefactoringOpportunity {
            opportunity_type: OpportunityType::RenameSymbol,
            description: "Rename function".to_string(),
            confidence: 0.9,
            affected_blocks: vec!["fn_0".to_string()],
            estimated_impact: ImpactLevel::Low,
        };
        let result = transformer.apply_refactoring(code, &opportunity, &blocks);
        assert!(result.success);
        assert!(result.transformed_code.contains("old_name_renamed"));
    }

    #[test]
    fn test_simplify_conditional() {
        let transformer = Transformer::new();
        let code = "if true {\n    do_something();\n}";
        let opportunity = RefactoringOpportunity {
            opportunity_type: OpportunityType::SimplifyConditional,
            description: "Simplify".to_string(),
            confidence: 1.0,
            affected_blocks: vec![],
            estimated_impact: ImpactLevel::Low,
        };
        let result = transformer.apply_refactoring(code, &opportunity, &[]);
        assert!(result.success);
        assert!(result.transformed_code.contains("// Simplified:"));
    }

    #[test]
    fn test_remove_dead_code() {
        let transformer = Transformer::new();
        let code = "// TODO: fix this\nfn main() {\n    // FIXME: broken\n    println!(\"hello\");\n}";
        let opportunity = RefactoringOpportunity {
            opportunity_type: OpportunityType::RemoveDeadCode,
            description: "Remove dead code".to_string(),
            confidence: 0.9,
            affected_blocks: vec![],
            estimated_impact: ImpactLevel::Medium,
        };
        let result = transformer.apply_refactoring(code, &opportunity, &[]);
        assert!(result.success);
        assert!(!result.transformed_code.contains("TODO"));
        assert!(!result.transformed_code.contains("FIXME"));
    }

    #[test]
    fn test_unimplemented_returns_error() {
        let transformer = Transformer::new();
        let code = "test code";
        let opportunity = RefactoringOpportunity {
            opportunity_type: OpportunityType::ImproveNaming,
            description: "Improve naming".to_string(),
            confidence: 0.5,
            affected_blocks: vec![],
            estimated_impact: ImpactLevel::Low,
        };
        let result = transformer.apply_refactoring(code, &opportunity, &[]);
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }
}
