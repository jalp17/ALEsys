//! Context Analyzer - Reads and understands project structure

use std::path::Path;

/// Analyzes project context
pub struct ContextAnalyzer;

impl ContextAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze project structure
    pub fn analyze_project(&self, root: &Path) -> ProjectContext {
        let mut context = ProjectContext::default();

        if root.exists() {
            self.walk_directory(root, &mut context, 0);
        }

        context
    }

    fn walk_directory(&self, dir: &Path, context: &mut ProjectContext, depth: usize) {
        if depth > 5 {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name == "node_modules" || name == "target" || name == ".git" {
                        continue;
                    }
                    self.walk_directory(&path, context, depth + 1);
                } else if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_string();
                    *context.file_types.entry(ext).or_insert(0) += 1;
                    context.total_files += 1;

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        context.total_lines += content.lines().count();
                    }
                }
            }
        }
    }
}

impl Default for ContextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Project context information
#[derive(Debug, Default)]
pub struct ProjectContext {
    pub total_files: usize,
    pub total_lines: usize,
    pub file_types: std::collections::HashMap<String, usize>,
}

impl ProjectContext {
    pub fn summary(&self) -> String {
        format!(
            "Project: {} files, {} lines, languages: {}",
            self.total_files,
            self.total_lines,
            self.file_types
                .iter()
                .map(|(k, v)| format!("{}({})", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_new() {
        let analyzer = ContextAnalyzer::new();
        let context = analyzer.analyze_project(Path::new("/nonexistent"));
        assert_eq!(context.total_files, 0);
    }
}
