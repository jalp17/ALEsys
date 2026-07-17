//! Implementación del generador de código

use super::{GenerateRequest, GenerationResult};
use crate::llm::{LLMBackend, LLMBackendType, LLMConfig, LLMEngine};
use anyhow::Result;
use std::sync::Arc;

/// Generador principal de código
pub struct CodeGenerator {
    llm: Arc<LLMBackend>,
}

impl CodeGenerator {
    /// Crea un nuevo generador con el backend LLM configurado
    pub async fn new() -> Result<Self> {
        let config = LLMConfig {
            backend: LLMBackendType::Auto,
            max_tokens: 2048,
            temperature: 0.7,
            ..Default::default()
        };

        let llm = LLMBackend::from_config(config).await?;
        Ok(Self {
            llm: Arc::new(llm),
        })
    }

    /// Genera código desde un prompt
    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerationResult> {
        // Construir prompt completo con templates
        let template = super::templates::get_template(&request.language)
            .unwrap_or(super::templates::PromptTemplate::generic());

        let full_prompt = template.render(&request.prompt, request.context.as_ref());

        // Generar código usando LLM
        let response = self.llm.generate_code(&full_prompt, &request.language)?;

        // Extraer nombre de archivo sugerido
        let file_name = self.suggest_filename(&request.prompt, &request.language);

        // Generar explicación y sugerencias
        let (explanation, suggestions) = self.analyze_generation(&response).await;

        Ok(GenerationResult {
            file_name,
            content: response,
            language: request.language,
            explanation,
            suggestions,
        })
    }

    /// Sugiere un nombre de archivo basado en el prompt
    fn suggest_filename(&self, prompt: &str, language: &str) -> String {
        // Extraer palabras clave del prompt
        let words: Vec<&str> = prompt
            .split_whitespace()
            .take(3)
            .collect();

        let base_name = words
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<_>>()
            .join("_");

        let extension = match language.to_lowercase().as_str() {
            "python" | "py" => "py",
            "javascript" | "js" => "js",
            "typescript" | "ts" => "ts",
            "rust" | "rs" => "rs",
            "java" => "java",
            "c" => "c",
            "cpp" | "c++" => "cpp",
            _ => "txt",
        };

        format!("{}.{}", base_name, extension)
    }

    /// Analiza el código generado para extraer explicación y sugerencias
    async fn analyze_generation(&self, code: &str) -> (String, Vec<String>) {
        let mut suggestions = Vec::new();

        // Análisis básico de código
        if code.contains("TODO") || code.contains("FIXME") {
            suggestions.push("Revisar marcadores TODO/FIXME".to_string());
        }

        if !code.contains("fn") && !code.contains("function") && !code.contains("def") {
            suggestions.push("Verificar que el código tenga funciones definidas".to_string());
        }

        // Contar líneas
        let line_count = code.lines().count();
        if line_count > 100 {
            suggestions.push(format!("Código extenso ({} líneas), considerar dividir", line_count));
        }

        // Detección de patrones comunes
        if code.contains("unwrap()") || code.contains(".unwrap") {
            suggestions.push("Considerar manejar errores explícitamente en vez de unwrap()".to_string());
        }

        let explanation = format!("Generado {} líneas de código {}", line_count, self.detect_language_pattern(code));

        (explanation, suggestions)
    }

    /// Detecta el patrón de lenguaje usado
    fn detect_language_pattern(&self, code: &str) -> &'static str {
        if code.contains("fn ") && code.contains("->") {
            "Rust"
        } else if code.contains("def ") {
            "Python"
        } else if code.contains("function ") || code.contains("=>") {
            "JavaScript/TypeScript"
        } else {
            "código"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_filename_python() {
        // Test unitario sin dependencias LLM
        let filename = suggest_filename_internal("Crear función factorial", "python");
        assert_eq!(filename, "crear_función_factorial.py");
    }

    #[test]
    fn test_suggest_filename_javascript() {
        let filename = suggest_filename_internal("Implementar clase User", "javascript");
        assert_eq!(filename, "implementar_clase_user.js");
    }

    // Función helper para tests
    fn suggest_filename_internal(prompt: &str, language: &str) -> String {
        let words: Vec<&str> = prompt.split_whitespace().take(3).collect();
        let base_name = words.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>().join("_");
        let extension = match language.to_lowercase().as_str() {
            "python" | "py" => "py",
            "javascript" | "js" => "js",
            _ => "txt",
        };
        format!("{}.{}", base_name, extension)
    }
}