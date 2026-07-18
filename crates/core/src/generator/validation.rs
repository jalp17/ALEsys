//! Validación básica de sintaxis para código generado

use anyhow::Result;

/// Validador de sintaxis para diferentes lenguajes
pub struct SyntaxValidator;

/// Verifica balance de delimitadores comunes (paréntesis, corchetes, llaves).
/// Retorna errores descriptivos para cada tipo desbalanceado.
fn check_balanced_delimiters(code: &str) -> Vec<String> {
    let mut errors = Vec::new();

    let paren_count = code.matches('(').count() as isize - code.matches(')').count() as isize;
    if paren_count != 0 {
        errors.push(format!(
            "Paréntesis no balanceados: faltan {} ')' ",
            paren_count.abs()
        ));
    }

    let bracket_count = code.matches('[').count() as isize - code.matches(']').count() as isize;
    if bracket_count != 0 {
        errors.push(format!(
            "Corchetes no balanceados: faltan {} ']' ",
            bracket_count.abs()
        ));
    }

    let brace_count = code.matches('{').count() as isize - code.matches('}').count() as isize;
    if brace_count != 0 {
        errors.push(format!(
            "Llaves no balanceadas: faltan {} '}}' ",
            brace_count.abs()
        ));
    }

    errors
}

impl SyntaxValidator {
    /// Valida sintaxis básica de código Python
    pub fn validate_python(code: &str) -> Result<bool> {
        let mut errors = check_balanced_delimiters(code);

        let single_quotes = code.matches('\'').count();
        let double_quotes = code.matches('"').count();
        if !single_quotes.is_multiple_of(2) {
            errors.push("Strings con single quotes no balanceadas".to_string());
        }
        if !double_quotes.is_multiple_of(2) {
            errors.push("Strings con double quotes no balanceadas".to_string());
        }

        if errors.is_empty() {
            Ok(true)
        } else {
            Err(anyhow::anyhow!(
                "Errores de sintaxis:\n{}",
                errors.join("\n")
            ))
        }
    }

    /// Valida sintaxis básica de JavaScript/TypeScript
    pub fn validate_javascript(code: &str) -> Result<bool> {
        let mut errors = check_balanced_delimiters(code);

        let single_quotes = code.matches('\'').count();
        let double_quotes = code.matches('"').count();
        let backticks = code.matches('`').count();

        if !single_quotes.is_multiple_of(2) {
            errors.push("Strings con single quotes no balanceadas".to_string());
        }
        if !double_quotes.is_multiple_of(2) {
            errors.push("Strings con double quotes no balanceadas".to_string());
        }
        if !backticks.is_multiple_of(2) {
            errors.push("Template literals no balanceados".to_string());
        }

        if errors.is_empty() {
            Ok(true)
        } else {
            Err(anyhow::anyhow!(
                "Errores de sintaxis:\n{}",
                errors.join("\n")
            ))
        }
    }

    /// Valida sintaxis básica de Rust
    pub fn validate_rust(code: &str) -> Result<bool> {
        let mut errors = check_balanced_delimiters(code);

        let double_quotes = code.matches('"').count();
        if !double_quotes.is_multiple_of(2) {
            errors.push("Strings no balanceadas".to_string());
        }

        let raw_strings = code.matches("r#\"").count();
        let raw_ends = code.matches("#\"").count();
        if raw_strings != raw_ends {
            errors.push("Raw strings no balanceadas".to_string());
        }

        if errors.is_empty() {
            Ok(true)
        } else {
            Err(anyhow::anyhow!(
                "Errores de sintaxis:\n{}",
                errors.join("\n")
            ))
        }
    }

    /// Valida sintaxis según el lenguaje
    pub fn validate(code: &str, language: &str) -> Result<bool> {
        match language.to_lowercase().as_str() {
            "python" | "py" => Self::validate_python(code),
            "javascript" | "js" | "typescript" | "ts" => Self::validate_javascript(code),
            "rust" | "rs" => Self::validate_rust(code),
            _ => Ok(true), // Skip validation for unknown languages
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_python_valid() {
        let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)
"#;
        assert!(SyntaxValidator::validate_python(code).is_ok());
    }

    #[test]
    fn test_validate_python_unbalanced_parens() {
        let code = r#"
def foo(
    print("hello"
"#;
        assert!(SyntaxValidator::validate_python(code).is_err());
    }

    #[test]
    fn test_validate_javascript_valid() {
        let code = r#"
function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
"#;
        assert!(SyntaxValidator::validate_javascript(code).is_ok());
    }

    #[test]
    fn test_validate_javascript_unbalanced_braces() {
        let code = r#"
function foo() {
    console.log("hello";
}
"#;
        assert!(SyntaxValidator::validate_javascript(code).is_err());
    }

    #[test]
    fn test_validate_rust_valid() {
        let code = r#"
fn factorial(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
"#;
        assert!(SyntaxValidator::validate_rust(code).is_ok());
    }

    #[test]
    fn test_validate_rust_unbalanced_braces() {
        let code = r#"
fn foo() {
    println!("hello";
}
"#;
        assert!(SyntaxValidator::validate_rust(code).is_err());
    }

    #[test]
    fn test_validate_generic() {
        assert!(SyntaxValidator::validate("print('hello')", "python").is_ok());
        assert!(SyntaxValidator::validate("console.log('hi')", "javascript").is_ok());
        assert!(SyntaxValidator::validate("fn main() {}", "rust").is_ok());
        assert!(SyntaxValidator::validate("unknown code", "unknown").is_ok());
    }
}
