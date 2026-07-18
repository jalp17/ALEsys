//! Code execution sandbox using Docker containers.
//!
//! Provides isolated code execution with resource limits (CPU, memory, time)
//! and no network access.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Docker error: {0}")]
    Docker(String),

    #[error("Execution timeout ({0}ms)")]
    Timeout(u64),

    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("Container setup failed: {0}")]
    SetupFailed(String),
}

impl Serialize for SandboxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Supported programming languages for sandbox execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    JavaScript,
    Rust,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Python => write!(f, "python"),
            Language::JavaScript => write!(f, "javascript"),
            Language::Rust => write!(f, "rust"),
        }
    }
}

impl std::str::FromStr for Language {
    type Err = SandboxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "python" | "py" => Ok(Language::Python),
            "javascript" | "js" | "node" => Ok(Language::JavaScript),
            "rust" | "rs" => Ok(Language::Rust),
            _ => Err(SandboxError::UnsupportedLanguage(s.to_string())),
        }
    }
}

/// Result of a sandboxed code execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// stdout output
    pub stdout: String,
    /// stderr output
    pub stderr: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether the execution was killed by timeout
    pub timed_out: bool,
    /// Language used
    pub language: Language,
}

/// Configuration for sandbox execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum execution time in milliseconds (default: 30000)
    pub timeout_ms: u64,
    /// Maximum memory in MB (default: 256)
    pub memory_limit_mb: u64,
    /// Maximum CPU shares (default: 1.0)
    pub cpu_shares: f64,
    /// Network access (default: false)
    pub network_access: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            memory_limit_mb: 256,
            cpu_shares: 1.0,
            network_access: false,
        }
    }
}

// =============================================================================
// Docker Executor
// =============================================================================

use bollard::container::{Config, CreateContainerOptions, LogOutput, RemoveContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::HostConfig;
use bollard::Docker;
use futures::StreamExt;
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

/// Docker image names for each language.
const PYTHON_IMAGE: &str = "python:3.11-slim";
const NODE_IMAGE: &str = "node:20-slim";
const RUST_IMAGE: &str = "rust:1.75-slim";

/// Execute code in a Docker container with resource limits.
pub async fn execute(
    code: &str,
    language: Language,
    config: &SandboxConfig,
) -> Result<ExecutionResult, SandboxError> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| SandboxError::Docker(e.to_string()))?;

    // Test Docker connection
    docker
        .ping()
        .await
        .map_err(|e| SandboxError::Docker(e.to_string()))?;

    let image = match language {
        Language::Python => PYTHON_IMAGE,
        Language::JavaScript => NODE_IMAGE,
        Language::Rust => RUST_IMAGE,
    };

    // Create container with resource limits
    let container_name = format!("alesys-sandbox-{}", uuid::Uuid::new_v4());
    let host_config = HostConfig {
        memory: Some(config.memory_limit_mb as i64 * 1024 * 1024),
        nano_cpus: Some((config.cpu_shares * 1_000_000_000.0) as i64),
        network_mode: Some(if config.network_access {
            "bridge".to_string()
        } else {
            "none".to_string()
        }),
        readonly_rootfs: Some(false),
        tmpfs: Some(HashMap::from([(
            "/tmp".to_string(),
            "size=100M".to_string(),
        )])),
        ..Default::default()
    };

    let create_options = CreateContainerOptions {
        name: container_name,
        ..Default::default()
    };

    let container_config = Config {
        image: Some(image.to_string()),
        host_config: Some(host_config),
        working_dir: Some("/tmp".to_string()),
        ..Default::default()
    };

    let container = docker
        .create_container(Some(create_options), container_config)
        .await
        .map_err(|e| SandboxError::SetupFailed(e.to_string()))?;

    // Start container
    docker
        .start_container::<String>(&container.id, None)
        .await
        .map_err(|e| SandboxError::Docker(e.to_string()))?;

    // Execute code
    let start_time = std::time::Instant::now();
    let exec_command: Vec<&str> = match language {
        Language::Python => vec!["python3", "-c", code],
        Language::JavaScript => vec!["node", "-e", code],
        Language::Rust => {
            let rust_cmd = format!(
                "echo '{}' > /tmp/code.rs && rustc /tmp/code.rs -o /tmp/code && /tmp/code",
                code.replace('\'', "'\\''")
            );
            // Leak is acceptable here — container lifetime is short
            let leaked = Box::leak(rust_cmd.into_boxed_str());
            vec!["sh", "-c", leaked]
        }
    };

    let exec_options = CreateExecOptions {
        cmd: Some(exec_command.iter().map(|s| s.to_string()).collect()),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        ..Default::default()
    };

    let exec = docker
        .create_exec(&container.id, exec_options)
        .await
        .map_err(|e| SandboxError::Docker(e.to_string()))?;

    // Start exec and collect output
    let start_options = StartExecOptions {
        detach: false,
        ..Default::default()
    };

    let exec_result = docker
        .start_exec(&exec.id, Some(start_options))
        .await
        .map_err(|e| SandboxError::Docker(e.to_string()))?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut timed_out = false;

    match exec_result {
        StartExecResults::Attached {
            mut output,
            input: _,
        } => {
            let collect_timeout = timeout(Duration::from_millis(config.timeout_ms), async {
                while let Some(result) = output.next().await {
                    match result {
                        Ok(frame) => {
                            let msg = format!("{}", frame);
                            match frame {
                                LogOutput::StdOut { .. } => stdout.push_str(&msg),
                                LogOutput::StdErr { .. } => stderr.push_str(&msg),
                                _ => stdout.push_str(&msg),
                            }
                        }
                        Err(e) => {
                            stderr.push_str(&format!("Error reading output: {}", e));
                        }
                    }
                }
            })
            .await;

            timed_out = collect_timeout.is_err();
        }
        StartExecResults::Detached => {
            stderr.push_str("Exec started in detached mode");
        }
    }

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    // Get exit code
    let exit_code = if timed_out {
        -1
    } else {
        let inspect = docker
            .inspect_exec(&exec.id)
            .await
            .map_err(|e| SandboxError::Docker(e.to_string()))?;
        inspect.exit_code.unwrap_or(-1) as i32
    };

    // Remove container
    let _ = docker
        .remove_container(
            &container.id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await;

    Ok(ExecutionResult {
        exit_code,
        stdout,
        stderr,
        execution_time_ms,
        timed_out,
        language,
    })
}

/// Main sandbox interface for executing code in isolated containers.
pub struct CodeSandbox {
    config: SandboxConfig,
}

impl CodeSandbox {
    /// Create a new CodeSandbox with default configuration.
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
        }
    }

    /// Create a new CodeSandbox with custom configuration.
    pub fn with_config(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Execute code in a sandboxed container.
    pub async fn execute(
        &self,
        code: &str,
        language: Language,
    ) -> Result<ExecutionResult, SandboxError> {
        execute(code, language, &self.config).await
    }

    /// Get the current configuration.
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }
}

impl Default for CodeSandbox {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!("python".parse::<Language>().unwrap(), Language::Python);
        assert_eq!("py".parse::<Language>().unwrap(), Language::Python);
        assert_eq!("javascript".parse::<Language>().unwrap(), Language::JavaScript);
        assert_eq!("js".parse::<Language>().unwrap(), Language::JavaScript);
        assert_eq!("rust".parse::<Language>().unwrap(), Language::Rust);
        assert_eq!("rs".parse::<Language>().unwrap(), Language::Rust);
        assert!("go".parse::<Language>().is_err());
    }

    #[test]
    fn test_sandbox_config_defaults() {
        let config = SandboxConfig::default();
        assert_eq!(config.timeout_ms, 30_000);
        assert_eq!(config.memory_limit_mb, 256);
        assert!(!config.network_access);
    }

    #[test]
    fn test_execution_result_serialization() {
        let result = ExecutionResult {
            exit_code: 0,
            stdout: "Hello".to_string(),
            stderr: String::new(),
            execution_time_ms: 100,
            timed_out: false,
            language: Language::Python,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ExecutionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.exit_code, 0);
        assert_eq!(deserialized.language, Language::Python);
    }

    #[test]
    fn test_python_image_name() {
        assert_eq!(PYTHON_IMAGE, "python:3.11-slim");
    }

    #[test]
    fn test_node_image_name() {
        assert_eq!(NODE_IMAGE, "node:20-slim");
    }

    #[test]
    fn test_rust_image_name() {
        assert_eq!(RUST_IMAGE, "rust:1.75-slim");
    }
}
