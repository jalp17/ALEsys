use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub environment: Environment,
    pub port: u16,
    pub host: String,
    pub log_level: LogLevel,
    pub max_connections: usize,
    pub cors_origins: Vec<String>,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
}

impl Default for DeployConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            port: 3000,
            host: "127.0.0.1".to_string(),
            log_level: LogLevel::Info,
            max_connections: 100,
            cors_origins: vec!["http://localhost:3000".to_string()],
            enable_metrics: true,
            enable_tracing: true,
        }
    }
}

impl DeployConfig {
    pub fn production() -> Self {
        Self {
            environment: Environment::Production,
            port: 443,
            host: "0.0.0.0".to_string(),
            log_level: LogLevel::Warn,
            max_connections: 1000,
            cors_origins: vec![],
            enable_metrics: true,
            enable_tracing: true,
        }
    }

    pub fn staging() -> Self {
        Self {
            environment: Environment::Staging,
            port: 8443,
            host: "0.0.0.0".to_string(),
            log_level: LogLevel::Info,
            max_connections: 500,
            cors_origins: vec!["https://staging.example.com".to_string()],
            enable_metrics: true,
            enable_tracing: true,
        }
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_config_default() {
        let config = DeployConfig::default();
        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_deploy_config_production() {
        let config = DeployConfig::production();
        assert!(config.is_production());
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_environment_from_str() {
        assert_eq!(Environment::from_str("production"), Environment::Production);
        assert_eq!(Environment::from_str("dev"), Environment::Development);
    }
}