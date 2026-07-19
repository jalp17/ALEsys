use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub test_type: TestType,
    pub input: String,
    pub expected_output: Option<String>,
    pub description: String,
    pub assertions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TestType {
    Unit,
    Integration,
    EdgeCase,
    ErrorHandling,
    Performance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub complexity: ComplexityLevel,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub type_name: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
}

pub struct TestGenerator {
    language: String,
    framework: String,
}

impl TestGenerator {
    pub fn new(language: &str, framework: &str) -> Self {
        Self {
            language: language.to_string(),
            framework: framework.to_string(),
        }
    }

    pub fn generate_for_function(&self, function: &FunctionInfo) -> Vec<TestCase> {
        let mut tests = Vec::new();

        tests.extend(self.generate_basic_cases(function));
        tests.extend(self.generate_edge_cases(function));
        if !function.dependencies.is_empty() {
            tests.push(self.generate_integration_case(function));
        }

        tests
    }

    fn generate_basic_cases(&self, function: &FunctionInfo) -> Vec<TestCase> {
        let mut cases = Vec::new();
        let input = self.generate_sample_input(function);

        cases.push(TestCase {
            name: format!("test_{}_basic", function.name),
            test_type: TestType::Unit,
            input: input.clone(),
            expected_output: Some(self.generate_expected_output(function)),
            description: format!("Basic test for {}", function.name),
            assertions: vec![self.generate_assertion(function, &input)],
        });

        if function.parameters.iter().any(|p| p.is_optional) {
            cases.push(TestCase {
                name: format!("test_{}_with_defaults", function.name),
                test_type: TestType::Unit,
                input: self.generate_input_with_defaults(function),
                expected_output: Some(self.generate_expected_output(function)),
                description: format!("Test {} with default parameters", function.name),
                assertions: vec![self.generate_assertion(function, &self.generate_input_with_defaults(function))],
            });
        }

        cases
    }

    fn generate_edge_cases(&self, function: &FunctionInfo) -> Vec<TestCase> {
        let mut cases = Vec::new();

        for param in &function.parameters {
            if param.type_name == "String" || param.type_name == "str" {
                cases.push(TestCase {
                    name: format!("test_{}_empty_string_{}", function.name, param.name),
                    test_type: TestType::EdgeCase,
                    input: self.generate_empty_input(function, &param.name),
                    expected_output: None,
                    description: format!("Empty string for parameter {}", param.name),
                    assertions: vec![self.generate_edge_assertion(function)],
                });
            }
            if param.type_name.contains("int") || param.type_name.contains("i32") || param.type_name.contains("u32") {
                cases.push(TestCase {
                    name: format!("test_{}_zero_{}", function.name, param.name),
                    test_type: TestType::EdgeCase,
                    input: self.generate_zero_input(function, &param.name),
                    expected_output: None,
                    description: format!("Zero value for parameter {}", param.name),
                    assertions: vec![self.generate_edge_assertion(function)],
                });
            }
        }

        if function.parameters.len() > 1 {
            cases.push(TestCase {
                name: format!("test_{}_no_params", function.name),
                test_type: TestType::EdgeCase,
                input: "None".to_string(),
                expected_output: None,
                description: "No parameters provided".to_string(),
                assertions: vec![self.generate_edge_assertion(function)],
            });
        }

        cases
    }

    fn generate_integration_case(&self, function: &FunctionInfo) -> TestCase {
        TestCase {
            name: format!("test_{}_integration", function.name),
            test_type: TestType::Integration,
            input: format!("Mock dependencies: {:?}", function.dependencies),
            expected_output: None,
            description: format!("Integration test with mocked dependencies for {}", function.name),
            assertions: vec![format!("assert!(result.is_ok()); // Integration test")],
        }
    }

    fn generate_sample_input(&self, function: &FunctionInfo) -> String {
        let params: Vec<String> = function.parameters.iter().map(|p| {
            match p.type_name.as_str() {
                "String" | "str" => "\"test\"".to_string(),
                "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => "42".to_string(),
                "f32" | "f64" => "3.14".to_string(),
                "bool" => "true".to_string(),
                "Vec<String>" | "Vec<&str>" => "vec![\"a\".to_string(), \"b\".to_string()]".to_string(),
                "Option<String>" | "Option<&str>" => "Some(\"test\".to_string())".to_string(),
                _ => "Default::default()".to_string(),
            }
        }).collect();
        params.join(", ")
    }

    fn generate_expected_output(&self, function: &FunctionInfo) -> String {
        match function.return_type.as_deref() {
            Some("String") | Some("String>") => "\"expected\".to_string()".to_string(),
            Some("bool") => "true".to_string(),
            Some(t) if t.contains("Option") => "Some(result)".to_string(),
            Some(t) if t.contains("Result") => "Ok(result)".to_string(),
            Some("()") => "()".to_string(),
            _ => "assert!(result.is_ok());".to_string(),
        }
    }

    fn generate_input_with_defaults(&self, function: &FunctionInfo) -> String {
        let params: Vec<String> = function.parameters.iter().map(|p| {
            if p.is_optional {
                if let Some(ref default) = p.default_value {
                    default.clone()
                } else {
                    "None".to_string()
                }
            } else {
                match p.type_name.as_str() {
                    "String" | "str" => "\"test\"".to_string(),
                    "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => "42".to_string(),
                    _ => "Default::default()".to_string(),
                }
            }
        }).collect();
        params.join(", ")
    }

    fn generate_empty_input(&self, _function: &FunctionInfo, _param_name: &str) -> String {
        "\"\"".to_string()
    }

    fn generate_zero_input(&self, _function: &FunctionInfo, _param_name: &str) -> String {
        "0".to_string()
    }

    fn generate_assertion(&self, _function: &FunctionInfo, _input: &str) -> String {
        "assert!(result.is_ok(), \"Test should pass\");".to_string()
    }

    fn generate_edge_assertion(&self, _function: &FunctionInfo) -> String {
        "assert!(result.is_ok() || result.is_err(), \"Should handle edge case gracefully\");".to_string()
    }

    pub fn generate_test_file(&self, function: &FunctionInfo) -> String {
        let tests = self.generate_for_function(function);
        let test_cases: Vec<String> = tests.iter().map(|tc| {
            format!(
                "    #[test]\n    fn {}() {{\n        // {}\n        {}\n        {}\n    }}",
                tc.name,
                tc.description,
                format!("let input = {};", tc.input),
                tc.assertions.join("\n        ")
            )
        }).collect();

        format!(
            "#[cfg(test)]\nmod tests {{\n    use super::*;\n\n{}\n}}",
            test_cases.join("\n\n")
        )
    }
}

impl Default for TestGenerator {
    fn default() -> Self {
        Self::new("rust", "built-in")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_function() -> FunctionInfo {
        FunctionInfo {
            name: "add".to_string(),
            parameters: vec![
                ParameterInfo {
                    name: "a".to_string(),
                    type_name: "i32".to_string(),
                    is_optional: false,
                    default_value: None,
                },
                ParameterInfo {
                    name: "b".to_string(),
                    type_name: "i32".to_string(),
                    is_optional: false,
                    default_value: None,
                },
            ],
            return_type: Some("i32".to_string()),
            is_async: false,
            complexity: ComplexityLevel::Simple,
            dependencies: vec![],
        }
    }

    #[test]
    fn test_generate_basic_cases() {
        let generator = TestGenerator::new("rust", "built-in");
        let function = sample_function();
        let cases = generator.generate_for_function(&function);
        assert!(!cases.is_empty());
        assert!(cases.iter().any(|c| c.test_type == TestType::Unit));
    }

    #[test]
    fn test_generate_edge_cases() {
        let generator = TestGenerator::new("rust", "built-in");
        let function = sample_function();
        let cases = generator.generate_for_function(&function);
        assert!(cases.iter().any(|c| c.test_type == TestType::EdgeCase));
    }

    #[test]
    fn test_generate_test_file() {
        let generator = TestGenerator::new("rust", "built-in");
        let function = sample_function();
        let file = generator.generate_test_file(&function);
        assert!(file.contains("#[cfg(test)]"));
        assert!(file.contains("mod tests"));
    }

    #[test]
    fn test_integration_case() {
        let generator = TestGenerator::new("rust", "built-in");
        let function = FunctionInfo {
            name: "process".to_string(),
            parameters: vec![ParameterInfo {
                name: "data".to_string(),
                type_name: "Vec<String>".to_string(),
                is_optional: false,
                default_value: None,
            }],
            return_type: Some("Result<String>".to_string()),
            is_async: false,
            complexity: ComplexityLevel::Complex,
            dependencies: vec!["database".to_string(), "cache".to_string()],
        };
        let cases = generator.generate_for_function(&function);
        assert!(cases.iter().any(|c| c.test_type == TestType::Integration));
    }
}
