//! Code sandbox for safe execution of untrusted code (Fase 7 / I+D)
//!
//! Provides a trait `CodeSandbox` with implementations:
//! - `DockerSandbox`: local Docker + gVisor (dev/CI)
//! - (future) `E2BSandbox`: cloud Firecracker microVMs
//! - (future) `FirecrackerSandbox`: on-prem microVMs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub timed_out: bool,
    pub sandbox_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    None,
    Restricted,
    Full,
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub image: String,
    pub cpu_limit: f32,
    pub memory_limit: String,
    pub timeout_ms: u64,
    pub network: NetworkMode,
    pub readonly_rootfs: bool,
    pub max_output_bytes: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: "python:3.11-slim".to_string(),
            cpu_limit: 1.0,
            memory_limit: "512m".to_string(),
            timeout_ms: 30_000,
            network: NetworkMode::None,
            readonly_rootfs: true,
            max_output_bytes: 10_000_000,
        }
    }
}

#[async_trait::async_trait]
pub trait CodeSandbox: Send + Sync {
    async fn execute(&self, code: &str, files: Vec<SandboxFile>) -> Result<SandboxResult, String>;
    fn name(&self) -> &str;
}

/// Local Docker sandbox with gVisor-friendly flags.
pub struct DockerSandbox {
    pub config: SandboxConfig,
}

impl DockerSandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    fn build_command(&self, workdir: &PathBuf, code: &str) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new("docker");
        cmd.arg("run")
            .arg("--rm")
            .arg(format!("--cpus={}", self.config.cpu_limit))
            .arg(format!("--memory={}", self.config.memory_limit))
            .arg("--network=none")
            .arg("--read-only")
            .arg("-v")
            .arg(format!("{}:/workspace:ro", workdir.display()))
            .arg("-w")
            .arg("/workspace")
            .arg(&self.config.image)
            .arg("sh")
            .arg("-c")
            .arg(code);
        cmd
    }
}

#[async_trait::async_trait]
impl CodeSandbox for DockerSandbox {
    async fn execute(&self, code: &str, files: Vec<SandboxFile>) -> Result<SandboxResult, String> {
        let sandbox_id = uuid::Uuid::new_v4().to_string();
        let workdir = std::env::temp_dir().join(format!("alesys-sandbox-{}", sandbox_id));
        std::fs::create_dir_all(&workdir).map_err(|e| format!("Failed to create workdir: {}", e))?;

        // Write files
        for f in &files {
            let path = workdir.join(&f.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent: {}", e))?;
            }
            std::fs::write(&path, &f.content).map_err(|e| format!("Failed to write file: {}", e))?;
        }

        let start = Instant::now();
        let mut cmd = self.build_command(&workdir, code);
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn docker: {}", e))?;

        let result = timeout(Duration::from_millis(self.config.timeout_ms), child.wait_with_output()).await;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Cleanup
        let _ = std::fs::remove_dir_all(&workdir);

        match result {
            Ok(Ok(output)) => Ok(SandboxResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                execution_time_ms,
                timed_out: false,
                sandbox_id,
            }),
            Ok(Err(e)) => Err(format!("Process error: {}", e)),
            Err(_) => Ok(SandboxResult {
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Timeout after {}ms", self.config.timeout_ms),
                execution_time_ms,
                timed_out: true,
                sandbox_id,
            }),
        }
    }

    fn name(&self) -> &str {
        "docker"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let cfg = SandboxConfig::default();
        assert_eq!(cfg.image, "python:3.11-slim");
        assert_eq!(cfg.network, NetworkMode::None);
        assert!(cfg.readonly_rootfs);
    }

    #[tokio::test]
    async fn test_docker_sandbox_echo() {
        // Skip if docker not available
        if std::process::Command::new("docker").arg("info").output().is_err() {
            return;
        }
        let sandbox = DockerSandbox::new(SandboxConfig::default());
        let res = sandbox.execute("echo hello", vec![]).await.unwrap();
        assert!(res.stdout.contains("hello"));
        assert_eq!(res.exit_code, 0);
    }
}
