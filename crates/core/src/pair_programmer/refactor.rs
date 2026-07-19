//! Auto-Refactor - Applies safe refactoring transformations

/// Auto-refactoring engine
pub struct AutoRefactor;

impl AutoRefactor {
    pub fn new() -> Self {
        Self
    }

    /// Remove trailing whitespace
    pub fn remove_trailing_whitespace(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Remove empty lines (consecutive)
    pub fn remove_empty_lines(&self, code: &str) -> String {
        let mut result = Vec::new();
        let mut prev_empty = false;

        for line in code.lines() {
            let is_empty = line.trim().is_empty();
            if !is_empty || !prev_empty {
                result.push(line);
            }
            prev_empty = is_empty;
        }

        result.join("\n")
    }

    /// Sort imports
    pub fn sort_imports(&self, code: &str) -> String {
        let mut imports = Vec::new();
        let mut non_imports = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") || trimmed.starts_with("import ") {
                imports.push(line);
            } else {
                non_imports.push(line);
            }
        }

        imports.sort();
        imports.extend(non_imports);
        imports.join("\n")
    }

    /// Extract function from code block
    pub fn extract_function(
        &self,
        code: &str,
        start_line: usize,
        end_line: usize,
        function_name: &str,
    ) -> (String, String) {
        let lines: Vec<&str> = code.lines().collect();
        let mut extracted = Vec::new();
        let mut new_code = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if i >= start_line && i <= end_line {
                extracted.push(*line);
            } else {
                new_code.push(*line);
            }
        }

        let function_body = format!(
            "fn {}() {{\n{}\n}}",
            function_name,
            extracted.iter().map(|l| format!("    {}", l)).collect::<Vec<_>>().join("\n")
        );

        let remaining = new_code.join("\n");
        (function_body, remaining)
    }
}

impl Default for AutoRefactor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_trailing_whitespace() {
        let refactor = AutoRefactor::new();
        let code = "fn main() {   \n    let x = 42;   \n}";
        let result = refactor.remove_trailing_whitespace(code);
        assert_eq!(result, "fn main() {\n    let x = 42;\n}");
    }

    #[test]
    fn test_remove_empty_lines() {
        let refactor = AutoRefactor::new();
        let code = "fn main() {\n\n\n\n    let x = 42;\n}";
        let result = refactor.remove_empty_lines(code);
        assert_eq!(result, "fn main() {\n\n    let x = 42;\n}");
    }

    #[test]
    fn test_sort_imports() {
        let refactor = AutoRefactor::new();
        let code = "use z::foo;\nuse a::bar;\nfn main() {}";
        let result = refactor.sort_imports(code);
        assert!(result.contains("use a::bar;"));
        assert!(result.contains("use z::foo;"));
    }
}
