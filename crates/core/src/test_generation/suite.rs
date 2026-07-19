use serde::{Deserialize, Serialize};
use super::generator::{TestCase, TestType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<TestCase>,
    pub setup_code: Option<String>,
    pub teardown_code: Option<String>,
    pub metadata: SuiteMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteMetadata {
    pub generated_at: String,
    pub source_file: Option<String>,
    pub language: String,
    pub framework: String,
    pub total_tests: usize,
    pub by_type: std::collections::HashMap<String, usize>,
}

impl TestSuite {
    pub fn new(name: &str, language: &str, framework: &str) -> Self {
        Self {
            name: name.to_string(),
            tests: Vec::new(),
            setup_code: None,
            teardown_code: None,
            metadata: SuiteMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                source_file: None,
                language: language.to_string(),
                framework: framework.to_string(),
                total_tests: 0,
                by_type: std::collections::HashMap::new(),
            },
        }
    }

    pub fn add_test(&mut self, test: TestCase) {
        self.tests.push(test);
        self.update_metadata();
    }

    pub fn add_tests(&mut self, tests: Vec<TestCase>) {
        self.tests.extend(tests);
        self.update_metadata();
    }

    pub fn get_tests_by_type(&self, test_type: &TestType) -> Vec<&TestCase> {
        self.tests.iter().filter(|t| &t.test_type == test_type).collect()
    }

    pub fn get_test_count(&self) -> usize {
        self.tests.len()
    }

    pub fn get_summary(&self) -> String {
        let unit = self.get_tests_by_type(&TestType::Unit).len();
        let integration = self.get_tests_by_type(&TestType::Integration).len();
        let edge = self.get_tests_by_type(&TestType::EdgeCase).len();
        let error = self.get_tests_by_type(&TestType::ErrorHandling).len();
        let perf = self.get_tests_by_type(&TestType::Performance).len();

        format!(
            "Test Suite '{}': {} tests total ({} unit, {} integration, {} edge, {} error, {} performance)",
            self.name, self.tests.len(), unit, integration, edge, error, perf
        )
    }

    pub fn export_to_file(&self) -> String {
        let mut output = String::new();

        match self.metadata.language.as_str() {
            "rust" => {
                output.push_str("#[cfg(test)]\n");
                output.push_str(&format!("mod {} {{\n", self.name.replace('-', "_")));
                output.push_str("    use super::*;\n\n");

                if let Some(ref setup) = self.setup_code {
                    output.push_str(&format!("    fn setup() {{\n{}\n    }}\n\n", setup));
                }

                for test in &self.tests {
                    output.push_str(&format!("    #[test]\n"));
                    output.push_str(&format!("    fn {}() {{\n", test.name));
                    output.push_str(&format!("        // {}\n", test.description));
                    output.push_str(&format!("        let input = {};\n", test.input));
                    for assertion in &test.assertions {
                        output.push_str(&format!("        {}\n", assertion));
                    }
                    output.push_str("    }\n\n");
                }

                output.push_str("}\n");
            }
            "python" => {
                output.push_str(&format!("import pytest\n\n"));
                output.push_str(&format!("class {}:\n", self.name.replace('-', "_")));

                if let Some(ref setup) = self.setup_code {
                    output.push_str(&format!("    def setup_method(self):\n{}\n\n", setup));
                }

                for test in &self.tests {
                    output.push_str(&format!("    def {}(self):\n", test.name));
                    output.push_str(&format!("        \"\"\"{}\"\"\"\n", test.description));
                    output.push_str(&format!("        input_data = {}\n", test.input));
                    for assertion in &test.assertions {
                        output.push_str(&format!("        {}\n", assertion));
                    }
                    output.push_str("\n");
                }
            }
            "typescript" | "javascript" => {
                output.push_str(&format!("describe('{}', () => {{\n", self.name));

                if let Some(ref setup) = self.setup_code {
                    output.push_str(&format!("  beforeEach(() => {{\n{}\n  }});\n\n", setup));
                }

                for test in &self.tests {
                    output.push_str(&format!("  it('{}', () => {{\n", test.name));
                    output.push_str(&format!("    // {}\n", test.description));
                    output.push_str(&format!("    const input = {};\n", test.input));
                    for assertion in &test.assertions {
                        output.push_str(&format!("    {}\n", assertion));
                    }
                    output.push_str("  });\n\n");
                }

                output.push_str("});\n");
            }
            _ => {
                for test in &self.tests {
                    output.push_str(&format!("Test: {}\n", test.name));
                    output.push_str(&format!("  Description: {}\n", test.description));
                    output.push_str(&format!("  Input: {}\n", test.input));
                    for assertion in &test.assertions {
                        output.push_str(&format!("  Assertion: {}\n", assertion));
                    }
                    output.push_str("\n");
                }
            }
        }

        output
    }

    fn update_metadata(&mut self) {
        self.metadata.total_tests = self.tests.len();
        self.metadata.by_type.clear();
        for test in &self.tests {
            let type_name = format!("{:?}", test.test_type);
            *self.metadata.by_type.entry(type_name).or_insert(0) += 1;
        }
    }
}

impl Default for TestSuite {
    fn default() -> Self {
        Self::new("test_suite", "rust", "built-in")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::generator::{FunctionInfo, TestGenerator};

    #[test]
    fn test_suite_creation() {
        let suite = TestSuite::new("my_tests", "rust", "built-in");
        assert_eq!(suite.name, "my_tests");
        assert_eq!(suite.get_test_count(), 0);
    }

    #[test]
    fn test_add_tests() {
        let mut suite = TestSuite::new("my_tests", "rust", "built-in");
        let generator = TestGenerator::new("rust", "built-in");
        let function = FunctionInfo {
            name: "add".to_string(),
            parameters: vec![],
            return_type: Some("i32".to_string()),
            is_async: false,
            complexity: super::super::generator::ComplexityLevel::Simple,
            dependencies: vec![],
        };
        let tests = generator.generate_for_function(&function);
        suite.add_tests(tests);
        assert!(suite.get_test_count() > 0);
    }

    #[test]
    fn test_get_summary() {
        let mut suite = TestSuite::new("summary_test", "rust", "built-in");
        let generator = TestGenerator::new("rust", "built-in");
        let function = FunctionInfo {
            name: "process".to_string(),
            parameters: vec![],
            return_type: Some("String".to_string()),
            is_async: false,
            complexity: super::super::generator::ComplexityLevel::Moderate,
            dependencies: vec!["dep1".to_string()],
        };
        let tests = generator.generate_for_function(&function);
        suite.add_tests(tests);
        let summary = suite.get_summary();
        assert!(summary.contains("summary_test"));
        assert!(summary.contains("tests total"));
    }

    #[test]
    fn test_export_rust() {
        let mut suite = TestSuite::new("export_test", "rust", "built-in");
        suite.add_test(super::super::generator::TestCase {
            name: "test_add".to_string(),
            test_type: super::super::generator::TestType::Unit,
            input: "1, 2".to_string(),
            expected_output: Some("3".to_string()),
            description: "Test addition".to_string(),
            assertions: vec!["assert_eq!(result, 3);".to_string()],
        });
        let output = suite.export_to_file();
        assert!(output.contains("#[cfg(test)]"));
        assert!(output.contains("fn test_add"));
    }
}
