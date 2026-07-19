use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizeResult {
    pub cleaned: String,
    pub was_modified: bool,
    pub threats_removed: Vec<String>,
}

pub struct Sanitizer;

impl Sanitizer {
    pub fn sanitize_input(input: &str) -> SanitizeResult {
        let mut cleaned = input.to_string();
        let mut threats = Vec::new();

        if cleaned.contains("<script") {
            cleaned = cleaned.replace("<script", "&lt;script");
            threats.push("script_tag".to_string());
        }

        if cleaned.contains("javascript:") {
            cleaned = cleaned.replace("javascript:", "javascript_");
            threats.push("javascript_uri".to_string());
        }

        if cleaned.contains("'") {
            cleaned = cleaned.replace("'", "''");
            threats.push("single_quote".to_string());
        }

        if cleaned.contains("--") {
            cleaned = cleaned.replace("--", " ");
            threats.push("sql_comment".to_string());
        }

        let was_modified = cleaned != input;

        SanitizeResult { cleaned, was_modified, threats_removed: threats }
    }

    pub fn sanitize_filename(name: &str) -> String {
        let mut result = String::new();
        for c in name.chars() {
            match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => result.push('_'),
                _ => result.push(c),
            }
        }
        result
    }

    pub fn sanitize_sql_identifier(name: &str) -> Result<String, String> {
        if name.is_empty() {
            return Err("Empty identifier".to_string());
        }

        let mut result = String::new();
        for (i, c) in name.chars().enumerate() {
            if i == 0 {
                if c.is_ascii_alphabetic() || c == '_' {
                    result.push(c);
                } else {
                    return Err(format!("Invalid start character: {}", c));
                }
            } else {
                if c.is_ascii_alphanumeric() || c == '_' {
                    result.push(c);
                } else {
                    return Err(format!("Invalid character: {}", c));
                }
            }
        }

        Ok(format!("\"{}\"", result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_input() {
        let result = Sanitizer::sanitize_input("hello <script>alert('xss')</script>");
        assert!(result.was_modified);
        assert!(result.cleaned.contains("&lt;script"));
    }

    #[test]
    fn test_sanitize_clean_input() {
        let result = Sanitizer::sanitize_input("hello world");
        assert!(!result.was_modified);
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(Sanitizer::sanitize_filename("file:name.txt"), "file_name.txt");
        assert_eq!(Sanitizer::sanitize_filename("normal.txt"), "normal.txt");
    }

    #[test]
    fn test_sanitize_sql_identifier() {
        assert_eq!(Sanitizer::sanitize_sql_identifier("users"), Ok("\"users\"".to_string()));
        assert!(Sanitizer::sanitize_sql_identifier("123abc").is_err());
        assert!(Sanitizer::sanitize_sql_identifier("").is_err());
    }
}