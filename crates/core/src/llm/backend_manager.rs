//! Gestor de backends LLM
//!
//! Descarga, compila y gestiona los backends disponibles.
//! Lee la configuración desde `crates/core/resources/models.toml`.

use super::config::{
    BackendBuildConfig, BuildMode, GpuType, ModelArch, ModelInfo, ModelRegistry, QuantType,
};
use crate::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Artefacto resultante de la build de un backend
#[derive(Debug, Clone)]
pub struct BackendArtifact {
    pub backend_type: String,
    pub path: PathBuf,
    pub version: String,
    pub is_python: bool,
    pub gpu_support: Vec<GpuType>,
}

/// Gestor principal de backends
pub struct BackendManager {
    registry: ModelRegistry,
    cache_dir: PathBuf,
    built: HashMap<String, BackendArtifact>,
}

impl BackendManager {
    /// Crea un nuevo BackendManager
    pub fn new() -> Result<Self> {
        let registry = Self::load_registry()?;
        let cache_dir = std::env::temp_dir().join("alesys/backends");
        std::fs::create_dir_all(&cache_dir)?;
        Ok(Self {
            registry,
            cache_dir,
            built: HashMap::new(),
        })
    }

    /// Carga el registry desde models.toml
    fn load_registry() -> Result<ModelRegistry> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/models.toml");

        if !path.exists() {
            return Err(crate::AlesysError::LLM(format!(
                "Model registry no encontrado: {}",
                path.display()
            )));
        }

        let content = std::fs::read_to_string(&path)?;
        let registry: ModelRegistry = toml::from_str(&content)
            .map_err(|e| crate::AlesysError::LLM(format!("Error parseando models.toml: {}", e)))?;
        Ok(registry)
    }

    /// Asegura que un backend esté disponible (lo descarga/compila si es necesario)
    pub async fn ensure_backend(&mut self, backend_name: &str) -> Result<BackendArtifact> {
        if let Some(artifact) = self.built.get(backend_name) {
            return Ok(artifact.clone());
        }

        let config = self.registry.backends.get(backend_name).ok_or_else(|| {
            crate::AlesysError::LLM(format!("Backend no registrado: {}", backend_name))
        })?;

        if !config.enabled {
            return Err(crate::AlesysError::LLM(format!(
                "Backend deshabilitado: {}. Habilitarlo en models.toml",
                backend_name
            )));
        }

        tracing::info!("Preparando backend: {} ({})", config.name, backend_name);

        let artifact = match &config.build_mode {
            BuildMode::CiPrebuilt => self.download_prebuilt(backend_name, config).await?,
            BuildMode::LocalCompile => self.compile_local(backend_name, config).await?,
            BuildMode::PythonInstall => self.install_python(backend_name, config).await?,
        };

        tracing::info!(
            "Backend listo: {} en {}",
            backend_name,
            artifact.path.display()
        );
        self.built
            .insert(backend_name.to_string(), artifact.clone());
        Ok(artifact)
    }

    /// Descarga binario precompilado de CI
    async fn download_prebuilt(
        &self,
        name: &str,
        config: &BackendBuildConfig,
    ) -> Result<BackendArtifact> {
        let backend_dir = self.cache_dir.join(name);
        std::fs::create_dir_all(&backend_dir)?;

        tracing::info!("Descargando binario precompilado para {}...", name);

        let binary = backend_dir.join(format!("alesys-{}", name));
        if binary.exists() {
            return Ok(BackendArtifact {
                backend_type: name.to_string(),
                path: binary,
                version: "local".to_string(),
                is_python: false,
                gpu_support: self.parse_gpu_backends(&config.gpu_backends),
            });
        }

        Err(crate::AlesysError::LLM(
            format!("Binario precompilado no encontrado para {}. Compilar localmente con: cargo build --features {}", name, config.features.join(","))
        ))
    }

    /// Compila backend localmente con Cargo
    async fn compile_local(
        &self,
        name: &str,
        config: &BackendBuildConfig,
    ) -> Result<BackendArtifact> {
        tracing::info!("Compilando {} localmente...", name);

        let features_str = config.features.join(",");

        tracing::info!(
            "Para compilar {}, ejecuta:\n  cargo build --release --features {}",
            name,
            features_str
        );

        Err(crate::AlesysError::LLM(format!(
            "Build local no implementado aún. Usar: cargo build --features {}",
            features_str
        )))
    }

    /// Instala dependencias Python y configura servidor
    async fn install_python(
        &self,
        name: &str,
        config: &BackendBuildConfig,
    ) -> Result<BackendArtifact> {
        let python_config = config.python.as_ref().ok_or_else(|| {
            crate::AlesysError::LLM(format!(
                "Backend {} requiere configuración Python en models.toml",
                name
            ))
        })?;

        tracing::info!("Instalando dependencias Python para {}...", name);

        let venv_dir = self.cache_dir.join(format!("{}_venv", name));
        let python_path = self.find_python(&python_config.version)?;

        if !venv_dir.join("bin/python").exists() {
            tracing::info!("Creando virtualenv en {}", venv_dir.display());
            let venv_str = venv_dir
                .to_str()
                .ok_or_else(|| crate::AlesysError::LLM("Ruta de venv contiene caracteres no UTF-8".to_string()))?;
            let status = tokio::process::Command::new(&python_path)
                .args(["-m", "venv", venv_str])
                .status()
                .await
                .map_err(|e| crate::AlesysError::LLM(format!("Error creando venv: {}", e)))?;

            if !status.success() {
                return Err(crate::AlesysError::LLM(
                    "Error creando virtualenv".to_string(),
                ));
            }
        }

        let pip_bin = venv_dir.join("bin/pip");
        let mut pip_args: Vec<String> = python_config
            .packages
            .iter()
            .map(|p| p.to_string())
            .collect();

        if let Some(ref index_url) = python_config.index_url {
            pip_args.insert(0, format!("--index-url={}", index_url));
        }

        tracing::info!("Instalando paquetes: {:?}", python_config.packages);

        let status = tokio::process::Command::new(&pip_bin)
            .args(&pip_args)
            .status()
            .await
            .map_err(|e| crate::AlesysError::LLM(format!("Error ejecutando pip: {}", e)))?;

        if !status.success() {
            return Err(crate::AlesysError::LLM(
                "Error instalando paquetes Python".to_string(),
            ));
        }

        let python_bin = venv_dir.join("bin/python");
        Ok(BackendArtifact {
            backend_type: name.to_string(),
            path: python_bin,
            version: python_config.version.clone(),
            is_python: true,
            gpu_support: self.parse_gpu_backends(&config.gpu_backends),
        })
    }

    /// Busca ejecutable Python compatible
    fn find_python(&self, _version_req: &str) -> Result<String> {
        let candidates = vec!["python3".to_string(), "python".to_string()];

        for candidate in &candidates {
            if let Ok(output) = std::process::Command::new(candidate)
                .arg("--version")
                .output()
            {
                let version = String::from_utf8_lossy(&output.stdout);
                tracing::info!("Python encontrado: {} ({})", candidate, version.trim());
                return Ok(candidate.clone());
            }
        }

        Err(crate::AlesysError::LLM("Python no encontrado".to_string()))
    }

    /// Parsea strings de GPU backends a enum GpuType
    fn parse_gpu_backends(&self, backends: &[String]) -> Vec<GpuType> {
        backends
            .iter()
            .map(|b| match b.to_lowercase().as_str() {
                "cuda" => GpuType::Cuda,
                "rocm" => GpuType::Rocm,
                "metal" => GpuType::Metal,
                "vulkan" => GpuType::Vulkan,
                _ => GpuType::None,
            })
            .collect()
    }

    /// Lista backends disponibles
    pub fn list_available(&self) -> Vec<&BackendBuildConfig> {
        self.registry
            .backends
            .values()
            .filter(|b| b.enabled)
            .collect()
    }

    /// Lista todos los backends registrados
    pub fn list_all(&self) -> &HashMap<String, BackendBuildConfig> {
        &self.registry.backends
    }

    /// Obtiene configuración de un backend
    pub fn get_backend_config(&self, name: &str) -> Option<&BackendBuildConfig> {
        self.registry.backends.get(name)
    }

    /// Detecta información de un modelo desde su path
    pub fn detect_model_info(model_path: &str) -> Result<ModelInfo> {
        let path = Path::new(model_path);

        if !path.exists() {
            return Err(crate::AlesysError::LLM(format!(
                "Modelo no encontrado: {}",
                model_path
            )));
        }

        if model_path.ends_with(".gguf") {
            return Self::detect_from_gguf_filename(path);
        }

        if path.is_dir() {
            return Self::detect_from_hf_config(path);
        }

        Ok(ModelInfo {
            arch: ModelArch::Unknown("unknown".to_string()),
            quant: QuantType::Unknown("unknown".to_string()),
            is_moe: false,
            parameter_count: None,
        })
    }

    /// Detecta info desde nombre de archivo GGUF
    fn detect_from_gguf_filename(path: &Path) -> Result<ModelInfo> {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let arch = if filename.contains("qwen3moe") || filename.contains("Qwen3-MoE") {
            ModelArch::Qwen3MoE
        } else if filename.contains("qwen3") || filename.contains("Qwen3") {
            ModelArch::Qwen3
        } else if filename.contains("llama") || filename.contains("Llama") {
            ModelArch::Llama
        } else if filename.contains("mistral") || filename.contains("Mistral") {
            ModelArch::Mistral
        } else {
            ModelArch::Unknown(filename.to_string())
        };

        let quant = if filename.contains("Q4_K") || filename.contains("q4_k") {
            QuantType::Q4K
        } else if filename.contains("Q5_K") || filename.contains("q5_k") {
            QuantType::Q5K
        } else if filename.contains("Q6_K") || filename.contains("q6_k") {
            QuantType::Q6K
        } else if filename.contains("Q8") || filename.contains("q8") {
            QuantType::Q8_0
        } else if filename.contains("F16") || filename.contains("f16") {
            QuantType::F16
        } else {
            QuantType::Unknown(filename.to_string())
        };

        let is_moe = filename.contains("MoE")
            || filename.contains("moe")
            || filename.contains("mixtral")
            || filename.contains("Mixtral");

        Ok(ModelInfo {
            arch,
            quant,
            is_moe,
            parameter_count: None,
        })
    }

    /// Detecta info desde config.json de HuggingFace
    fn detect_from_hf_config(path: &Path) -> Result<ModelInfo> {
        let config_path = path.join("config.json");
        if !config_path.exists() {
            return Err(crate::AlesysError::LLM(format!(
                "config.json no encontrado en {}",
                path.display()
            )));
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;

        let arch_str = config
            .get("architectures")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|a| a.as_str())
            .unwrap_or("unknown");

        let arch = match arch_str {
            "LlamaForCausalLM" | "LlamaModel" => ModelArch::Llama,
            "MistralForCausalLM" => ModelArch::Mistral,
            "MixtralForCausalLM" => ModelArch::Mixtral,
            "Qwen2ForCausalLM" => ModelArch::Qwen2,
            "Qwen3ForCausalLM" => ModelArch::Qwen3,
            "Qwen3MoeForCausalLM" => ModelArch::Qwen3MoE,
            "PhiForCausalLM" | "Phi3ForCausalLM" => ModelArch::Phi3,
            "GemmaForCausalLM" | "Gemma2ForCausalLM" => ModelArch::Gemma,
            _ => ModelArch::Unknown(arch_str.to_string()),
        };

        let is_moe = arch_str.contains("Moe")
            || arch_str.contains("Mixtral")
            || config
                .get("num_experts")
                .map(|e| e.as_u64().unwrap_or(0) > 1)
                .unwrap_or(false);

        Ok(ModelInfo {
            arch,
            quant: QuantType::F16,
            is_moe,
            parameter_count: config
                .get("num_parameters")
                .and_then(|p| p.as_f64())
                .map(|p| p / 1e9),
        })
    }
}
