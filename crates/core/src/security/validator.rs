use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

pub struct InputValidator;

impl InputValidator {
    pub fn validate_required(value: &str, field: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            Err(ValidationError {
                field: field.to_string(),
                message: "Field is required".to_string(),
                code: "REQUIRED".to_string(),
            })
        } else {
            Ok(())
        }
    }

    pub fn validate_length(value: &str, field: &str, min: usize, max: usize) -> Result<(), ValidationError> {
        let len = value.len();
        if len < min || len > max {
            Err(ValidationError {
                field: field.to_string(),
                message: format!("Length must be between {} and {}", min, max),
                code: "INVALID_LENGTH".to_string(),
            })
        } else {
            Ok(())
        }
    }

    pub fn validate_range<T: PartialOrd + std::fmt::Debug>(value: T, field: &str, min: T, max: T) -> Result<(), ValidationError> {
        if value < min || value > max {
            Err(ValidationError {
                field: field.to_string(),
                message: format!("Value must be between {:?} and {:?}", min, max),
                code: "OUT_OF_RANGE".to_string(),
            })
        } else {
            Ok(())
        }
    }

    pub fn validate_email(value: &str) -> Result<(), ValidationError> {
        if value.contains('@') && value.contains('.') {
            Ok(())
        } else {
            Err(ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
                code: "INVALID_EMAIL".to_string(),
            })
        }
    }

    pub fn validate_no_sql_injection(value: &str) -> Result<(), ValidationError> {
        let dangerous = ["'", "\"", ";", "--", "/*", "*/", "UNION", "SELECT", "DROP", "INSERT", "DELETE", "UPDATE"];
        let upper = value.to_uppercase();
        for pattern in &dangerous {
            if upper.contains(&pattern.to_uppercase()) {
                return Err(ValidationError {
                    field: "input".to_string(),
                    message: "Potentially dangerous input detected".to_string(),
                    code: "SQL_INJECTION".to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn validate_no_xss(value: &str) -> Result<(), ValidationError> {
        let patterns = ["<script", "javascript:", "onerror=", "onclick=", "onload="];
        let lower = value.to_lowercase();
        for pattern in &patterns {
            if lower.contains(pattern) {
                return Err(ValidationError {
                    field: "input".to_string(),
                    message: "Potentially dangerous input detected".to_string(),
                    code: "XSS_DETECTED".to_string(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_required() {
        assert!(InputValidator::validate_required("hello", "name").is_ok());
        assert!(InputValidator::validate_required("", "name").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(InputValidator::validate_length("hi", "name", 1, 10).is_ok());
        assert!(InputValidator::validate_length("", "name", 1, 10).is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(InputValidator::validate_email("test@example.com").is_ok());
        assert!(InputValidator::validate_email("invalid").is_err());
    }

    #[test]
    fn test_validate_no_sql_injection() {
        assert!(InputValidator::validate_no_sql_injection("hello world").is_ok());
        assert!(InputValidator::validate_no_sql_injection("'; DROP TABLE users;--").is_err());
    }

    #[test]
    fn test_validate_no_xss() {
        assert!(InputValidator::validate_no_xss("hello world").is_ok());
        assert!(InputValidator::validate_no_xss("<script>alert('xss')</script>").is_err());
    }
}