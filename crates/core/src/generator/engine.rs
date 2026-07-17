//! Implementación del generador de código

use super::{templates, GenerateRequest, GenerationResult};
use crate::llm::{LLMBackend, LLMEngine};
use crate::generator::validation::SyntaxValidator;
use anyhow::Result;
use std::sync::Arc;

/// Generador principal de código
///
/// Reutiliza un `LLMBackend` compartido (inyectado via `AppState`).
/// No crea instancias propias del backend LLM.
pub struct CodeGenerator {
    llm: Arc<LLMBackend>,
}

impl CodeGenerator {
    /// Crea un generador reutilizando el backend LLM existente
    pub fn new(llm: Arc<LLMBackend>) -> Self {
        Self { llm }
    }

    /// Genera código desde un prompt
    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerationResult> {
        tracing::info!(
            "Generación solicitada: lang={}, prompt_len={}",
            request.language,
            request.prompt.len()
        );

        // 1. Seleccionar template según lenguaje
        let template = templates::get_template(&request.language)
            .unwrap_or(templates::PromptTemplate::generic());
        tracing::debug!("Template seleccionado: {}", request.language);

        // 2. Renderizar prompt completo (system + requirements + context + user)
        let full_prompt = template.render(&request.prompt, request.context.as_ref());

        // 3. Generar código usando LLM compartido
        let response = self.llm.generate_code(&full_prompt, &request.language).await?;

        // 4. Validar sintaxis del código generado
        let validation_result = SyntaxValidator::validate(&response, &request.language);
        let validation_warnings = match validation_result {
            Ok(true) => {
                tracing::debug!("Validación de sintaxis: OK");
                Vec::new()
            }
            Ok(false) => Vec::new(),
            Err(e) => {
                tracing::warn!("Validación de sintaxis falló: {}", e);
                vec![format!("Advertencia de sintaxis: {}", e)]
            }
        };

        // 5. Extraer nombre de archivo sugerido
        let file_name = self.suggest_filename(&request.prompt, &request.language);

        // 6. Análisis estático → explicación + sugerencias
        let (explanation, mut suggestions) = self.analyze_generation(&response);
        suggestions.extend(validation_warnings);

        tracing::info!(
            "Generación completada: file={}, lines={}",
            file_name,
            response.lines().count()
        );

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
        let words: Vec<&str> = prompt.split_whitespace().take(3).collect();

        let base_name: String = words
            .iter()
            .map(|s| {
                s.to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                    .take(50)
                    .collect::<String>()
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("_");

        let base_name = if base_name.is_empty() {
            "untitled".to_string()
        } else {
            base_name
        };

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

    /// Análisis estático básico del código generado
    fn analyze_generation(&self, code: &str) -> (String, Vec<String>) {
        let mut suggestions = Vec::new();

        if code.contains("TODO") || code.contains("FIXME") {
            suggestions.push("Revisar marcadores TODO/FIXME".to_string());
        }

        if !code.contains("fn") && !code.contains("function") && !code.contains("def") {
            suggestions.push("Verificar que el código tenga funciones definidas".to_string());
        }

        let line_count = code.lines().count();
        if line_count > 100 {
            suggestions.push(format!("Código extenso ({} líneas), considerar dividir", line_count));
        }

        if code.contains("unwrap()") || code.contains(".unwrap") {
            suggestions.push("Considerar manejar errores explícitamente en vez de unwrap()".to_string());
        }

        let explanation = format!(
            "Generado {} líneas de código {}",
            line_count,
            self.detect_language_pattern(code)
        );

        (explanation, suggestions)
    }

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

    #[test]
    fn test_suggest_filename_python() {
        let filename = suggest_filename_internal("Crear función factorial", "python");
        assert_eq!(filename, "crear_función_factorial.py");
    }

    #[test]
    fn test_suggest_filename_javascript() {
        let filename = suggest_filename_internal("Implementar clase User", "javascript");
        assert_eq!(filename, "implementar_clase_user.js");
    }

    #[test]
    fn test_suggest_filename_sanitizes_path_traversal() {
        let filename = suggest_filename_internal("foo/../../../etc/passwd", "python");
        assert_eq!(filename, "fooetcpasswd.py");
    }

    #[test]
    fn test_suggest_filename_sanitizes_special_chars() {
        let filename = suggest_filename_internal("file; rm -rf /", "python");
        assert_eq!(filename, "file_rm_-rf.py");
    }

    #[test]
    fn test_suggest_filename_empty_prompt() {
        let filename = suggest_filename_internal("", "python");
        assert_eq!(filename, "untitled.py");
    }

    fn suggest_filename_internal(prompt: &str, language: &str) -> String {
        let words: Vec<&str> = prompt.split_whitespace().take(3).collect();
        let base_name: String = words
            .iter()
            .map(|s| {
                s.to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                    .take(50)
                    .collect::<String>()
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("_");
        let base_name = if base_name.is_empty() {
            "untitled".to_string()
        } else {
            base_name
        };
        let extension = match language.to_lowercase().as_str() {
            "python" | "py" => "py",
            "javascript" | "js" => "js",
            _ => "txt",
        };
        format!("{}.{}", base_name, extension)
    }
}
