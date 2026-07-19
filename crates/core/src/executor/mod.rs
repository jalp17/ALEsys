use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    JavaScript,
    Rust,
    Shell,
    Custom(String),
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Python => write!(f, "python"),
            Language::JavaScript => write!(f, "javascript"),
            Language::Rust => write!(f, "rust"),
            Language::Shell => write!(f, "shell"),
            Language::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub timed_out: bool,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            max_output_bytes: 10_000_000,
        }
    }
}

pub async fn execute(
    command: &str,
    args: &[&str],
    workdir: Option<&str>,
    config: &ExecutorConfig,
) -> Result<ExecutionResult, String> {
    let start = std::time::Instant::now();

    let mut cmd = tokio::process::Command::new(command);
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if let Some(dir) = workdir {
        cmd.current_dir(dir);
    }

    let child = cmd.spawn().map_err(|e| format!("Failed to spawn process: {}", e))?;

    let result = timeout(Duration::from_millis(config.timeout_ms), child.wait_with_output())
        .await;

    let execution_time_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(output)) => Ok(ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time_ms,
            timed_out: false,
            language: String::new(),
        }),
        Ok(Err(e)) => Err(format!("Process error: {}", e)),
        Err(_) => Err(format!("Timeout after {}ms", config.timeout_ms)),
    }
}