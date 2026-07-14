//! Sandbox de ejecución de código
//!
//! ⚠️  FASE AVANZADA - NO IMPLEMENTAR HASTA FASE 7
//!
//! Características planificadas:
//! - Ejecución aislada con Docker/firecracker
//! - Límites de CPU/memoria/tiempo
//! - Soporte para Python, Rust, JavaScript, Bash
//! - Streaming de output en tiempo real

use crate::Result;
use std::str::FromStr;
use std::time::Duration;

/// Configuración del sandbox
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub timeout: Duration,
    pub max_memory_mb: u64,
    pub allowed_languages: Vec<String>,
    pub network_access: bool,          // ⚠️ default: false
    pub filesystem_paths: Vec<String>, // paths permitidos (default: solo /tmp)
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_memory_mb: 256,
            allowed_languages: vec!["python".to_string()],
            network_access: false,
            filesystem_paths: vec!["/tmp".to_string()],
        }
    }
}

/// Resultado de ejecución
#[derive(Debug)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

/// Sandbox para ejecución de código
pub struct CodeSandbox {
    _config: SandboxConfig,
}

impl CodeSandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self { _config: config }
    }

    /// Ejecutar código
    pub async fn execute(&self, _code: &str, _language: &str) -> Result<ExecutionResult> {
        // ⚠️  TODO: Implementar en Fase 7
        todo!("Implementar sandbox de ejecución - FASE 7")
    }

    /// Ejecutar con streaming de output
    pub async fn execute_streaming(
        &self,
        _code: &str,
        _language: &str,
        mut _stdout_callback: impl FnMut(String),
        mut _stderr_callback: impl FnMut(String),
    ) -> Result<ExecutionResult> {
        // ⚠️  TODO: Implementar en Fase 7
        todo!("Implementar ejecución con streaming - FASE 7")
    }
}

/// Lenguajes soportados
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupportedLanguage {
    Python,
    Rust,
    JavaScript,
    Bash,
}

impl FromStr for SupportedLanguage {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "python" | "py" => Ok(Self::Python),
            "rust" | "rs" => Ok(Self::Rust),
            "javascript" | "js" | "node" => Ok(Self::JavaScript),
            "bash" | "sh" => Ok(Self::Bash),
            _ => Err(()),
        }
    }
}

impl SupportedLanguage {
    pub fn from_str_compat(s: &str) -> Option<Self> {
        Self::from_str(s).ok()
    }
}

// ⚠️  ADVERTENCIA DE SEGURIDAD
//
// La ejecución de código de usuarios es PELIGROSA. Implementar:
//
// 1. Aislamiento estricto (Docker/firecracker)
// 2. Límites de recursos (CPU, memoria, tiempo)
// 3. Sin acceso a red por defecto
// 4. Filesystem read-only except /tmp
// 5. Nunca ejecutar como root
// 6. Logs de auditoría de toda ejecución
// 7. Rate limiting por usuario
// 8. Validación de código antes de ejecutar (opcional)
