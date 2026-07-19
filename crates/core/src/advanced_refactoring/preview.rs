use serde::{Deserialize, Serialize};
use super::transformer::{RefactoringResult, Change};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringPreview {
    pub original: String,
    pub transformed: String,
    pub diff: Vec<DiffLine>,
    pub changes_summary: String,
    pub warnings: Vec<String>,
    pub can_apply: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_number: usize,
    pub line_type: DiffLineType,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DiffLineType {
    Added,
    Removed,
    Context,
    Modified,
}

pub struct PreviewGenerator;

impl PreviewGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_preview(&self, result: &RefactoringResult) -> RefactoringPreview {
        let diff = self.compute_diff(&result.original_code, &result.transformed_code);
        let changes_summary = self.summarize_changes(&result.changes);
        let warnings = self.generate_warnings(result);
        let can_apply = result.success && warnings.is_empty();

        RefactoringPreview {
            original: result.original_code.clone(),
            transformed: result.transformed_code.clone(),
            diff,
            changes_summary,
            warnings,
            can_apply,
        }
    }

    fn compute_diff(&self, original: &str, transformed: &str) -> Vec<DiffLine> {
        let original_lines: Vec<&str> = original.lines().collect();
        let transformed_lines: Vec<&str> = transformed.lines().collect();
        let mut diff = Vec::new();

        let mut i = 0;
        let mut j = 0;

        while i < original_lines.len() || j < transformed_lines.len() {
            if i < original_lines.len() && j < transformed_lines.len() {
                if original_lines[i] == transformed_lines[j] {
                    diff.push(DiffLine {
                        line_number: i + 1,
                        line_type: DiffLineType::Context,
                        content: original_lines[i].to_string(),
                    });
                    i += 1;
                    j += 1;
                } else {
                    diff.push(DiffLine {
                        line_number: i + 1,
                        line_type: DiffLineType::Removed,
                        content: original_lines[i].to_string(),
                    });
                    diff.push(DiffLine {
                        line_number: j + 1,
                        line_type: DiffLineType::Added,
                        content: transformed_lines[j].to_string(),
                    });
                    i += 1;
                    j += 1;
                }
            } else if i < original_lines.len() {
                diff.push(DiffLine {
                    line_number: i + 1,
                    line_type: DiffLineType::Removed,
                    content: original_lines[i].to_string(),
                });
                i += 1;
            } else {
                diff.push(DiffLine {
                    line_number: j + 1,
                    line_type: DiffLineType::Added,
                    content: transformed_lines[j].to_string(),
                });
                j += 1;
            }
        }

        diff
    }

    fn summarize_changes(&self, changes: &[Change]) -> String {
        if changes.is_empty() {
            return "No changes to apply".to_string();
        }

        let mut summary = format!("{} change(s) to apply:\n", changes.len());
        for change in changes {
            summary.push_str(&format!(
                "- {:?}: {} (lines {}-{})\n",
                change.change_type, change.description, change.start_line, change.end_line
            ));
        }
        summary
    }

    fn generate_warnings(&self, result: &RefactoringResult) -> Vec<String> {
        let mut warnings = Vec::new();

        if !result.errors.is_empty() {
            warnings.extend(result.errors.iter().cloned());
        }

        if result.changes.len() > 5 {
            warnings.push("Large refactoring detected. Review changes carefully.".to_string());
        }

        if result.original_code.len() > 10000 {
            warnings.push("Code is very large. Consider breaking into smaller refactorings.".to_string());
        }

        warnings
    }

    pub fn format_diff_as_text(&self, preview: &RefactoringPreview) -> String {
        let mut output = String::new();
        output.push_str("=== Refactoring Preview ===\n\n");
        output.push_str(&format!("Changes: {}\n\n", preview.changes_summary));

        if !preview.warnings.is_empty() {
            output.push_str("⚠ Warnings:\n");
            for warning in &preview.warnings {
                output.push_str(&format!("  - {}\n", warning));
            }
            output.push('\n');
        }

        output.push_str("Diff:\n");
        for line in &preview.diff {
            let prefix = match line.line_type {
                DiffLineType::Added => "+",
                DiffLineType::Removed => "-",
                DiffLineType::Context => " ",
                DiffLineType::Modified => "~",
            };
            output.push_str(&format!("{} {}\n", prefix, line.content));
        }

        output.push_str(&format!(
            "\nCan apply: {}",
            if preview.can_apply { "Yes" } else { "No" }
        ));

        output
    }

    pub fn format_diff_as_html(&self, preview: &RefactoringPreview) -> String {
        let mut html = String::new();
        html.push_str("<div class='refactoring-preview'>");
        html.push_str("<h3>Refactoring Preview</h3>");
        html.push_str(&format!("<p>{}</p>", preview.changes_summary));

        if !preview.warnings.is_empty() {
            html.push_str("<div class='warnings'>");
            html.push_str("<h4>Warnings</h4><ul>");
            for warning in &preview.warnings {
                html.push_str(&format!("<li>{}</li>", warning));
            }
            html.push_str("</ul></div>");
        }

        html.push_str("<pre class='diff'>");
        for line in &preview.diff {
            let class = match line.line_type {
                DiffLineType::Added => "diff-added",
                DiffLineType::Removed => "diff-removed",
                DiffLineType::Context => "diff-context",
                DiffLineType::Modified => "diff-modified",
            };
            let prefix = match line.line_type {
                DiffLineType::Added => "+",
                DiffLineType::Removed => "-",
                DiffLineType::Context => " ",
                DiffLineType::Modified => "~",
            };
            html.push_str(&format!(
                "<span class='{}'>{} {}</span>\n",
                class, prefix, line.content
            ));
        }
        html.push_str("</pre>");

        html.push_str(&format!(
            "<p>Can apply: <strong>{}</strong></p>",
            if preview.can_apply { "Yes" } else { "No" }
        ));

        html.push_str("</div>");
        html
    }
}

impl Default for PreviewGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::transformer::{RefactoringResult, Change, ChangeType};

    #[test]
    fn test_generate_preview() {
        let generator = PreviewGenerator::new();
        let result = RefactoringResult {
            original_code: "fn old() {\n    println!(\"hello\");\n}".to_string(),
            transformed_code: "fn new() {\n    println!(\"hello\");\n}".to_string(),
            changes: vec![Change {
                change_type: ChangeType::Rename,
                description: "Renamed function".to_string(),
                start_line: 1,
                end_line: 1,
                original: "old".to_string(),
                replacement: "new".to_string(),
            }],
            success: true,
            errors: vec![],
        };
        let preview = generator.generate_preview(&result);
        assert!(preview.can_apply);
        assert!(!preview.diff.is_empty());
    }

    #[test]
    fn test_format_diff_text() {
        let generator = PreviewGenerator::new();
        let preview = RefactoringPreview {
            original: "old".to_string(),
            transformed: "new".to_string(),
            diff: vec![DiffLine {
                line_number: 1,
                line_type: DiffLineType::Added,
                content: "new".to_string(),
            }],
            changes_summary: "1 change".to_string(),
            warnings: vec![],
            can_apply: true,
        };
        let text = generator.format_diff_as_text(&preview);
        assert!(text.contains("Refactoring Preview"));
        assert!(text.contains("+ new"));
    }

    #[test]
    fn test_format_diff_html() {
        let generator = PreviewGenerator::new();
        let preview = RefactoringPreview {
            original: "old".to_string(),
            transformed: "new".to_string(),
            diff: vec![DiffLine {
                line_number: 1,
                line_type: DiffLineType::Added,
                content: "new".to_string(),
            }],
            changes_summary: "1 change".to_string(),
            warnings: vec![],
            can_apply: true,
        };
        let html = generator.format_diff_as_html(&preview);
        assert!(html.contains("refactoring-preview"));
        assert!(html.contains("diff-added"));
    }

    #[test]
    fn test_warnings_generated() {
        let generator = PreviewGenerator::new();
        let result = RefactoringResult {
            original_code: "test".to_string(),
            transformed_code: "test".to_string(),
            changes: vec![],
            success: false,
            errors: vec!["Something went wrong".to_string()],
        };
        let preview = generator.generate_preview(&result);
        assert!(!preview.warnings.is_empty());
        assert!(!preview.can_apply);
    }
}
