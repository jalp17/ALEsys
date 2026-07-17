//! Generación de código y archivos desde prompts naturales
//!
//! Este módulo proporciona funcionalidad para:
//! - Generar código en múltiples lenguajes (Python, JavaScript, Rust)
//! - Inyectar contexto desde archivos existentes
//! - Validación básica de sintaxis
//! - Templates de prompts optimizados para cada lenguaje

mod engine;
mod templates;
mod validation;

pub use engine::CodeGenerator;
pub use templates::PromptTemplate;
pub use validation::SyntaxValidator;

/// Request para generación de código
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerateRequest {
    /// Prompt natural del usuario
    pub prompt: String,
    
    /// Lenguaje objetivo (python, javascript, rust, etc.)
    pub language: String,
    
    /// Contexto opcional de archivos existentes
    #[serde(default)]
    pub context: Option<BuildContext>,
    
    /// Número máximo de tokens a generar
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

fn default_max_tokens() -> usize {
    2048
}

impl GenerateRequest {
    /// Crea un nuevo request de generación
    pub fn new(prompt: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            language: language.into(),
            context: None,
            max_tokens: default_max_tokens(),
        }
    }

    /// Añade contexto de archivos existentes
    pub fn with_context(mut self, context: BuildContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Establece máximo de tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}

/// Contexto de archivos existentes para inyección
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildContext {
    /// Tipo de proyecto (library, application, etc.)
    pub project_type: Option<String>,
    
    /// Lista de archivos existentes con su contenido
    #[serde(default)]
    pub existing_files: Vec<FileInfo>,
    
    /// Dependencias del proyecto
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl BuildContext {
    /// Crea contexto vacío
    pub fn new() -> Self {
        Self {
            project_type: None,
            existing_files: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Añade un archivo al contexto
    pub fn add_file(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.existing_files.push(FileInfo {
            name: name.into(),
            content: content.into(),
        });
        self
    }

    /// Añade una dependencia
    pub fn add_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }
}

impl Default for BuildContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Información de un archivo existente
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileInfo {
    /// Nombre/ruta del archivo
    pub name: String,
    
    /// Contenido del archivo
    pub content: String,
}

/// Resultado de la generación
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationResult {
    /// Nombre sugerido para el archivo
    pub file_name: String,
    
    /// Código generado
    pub content: String,
    
    /// Lenguaje del código
    pub language: String,
    
    /// Explicación breve de lo generado
    pub explanation: String,
    
    /// Sugerencias adicionales
    #[serde(default)]
    pub suggestions: Vec<String>,
}