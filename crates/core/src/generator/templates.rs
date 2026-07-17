//! Templates de prompts optimizados para generación de código

use super::BuildContext;

/// Template de prompt para generación de código
pub struct PromptTemplate {
    system_prompt: String,
    requirements: Vec<String>,
    output_format: String,
}

impl PromptTemplate {
    /// Crea un nuevo template
    pub fn new(
        system_prompt: impl Into<String>,
        requirements: Vec<String>,
        output_format: impl Into<String>,
    ) -> Self {
        Self {
            system_prompt: system_prompt.into(),
            requirements,
            output_format: output_format.into(),
        }
    }

    /// Renderiza el prompt completo
    pub fn render(&self, user_prompt: &str, context: Option<&BuildContext>) -> String {
        let mut prompt = String::new();

        // System prompt
        prompt.push_str(&self.system_prompt);
        prompt.push('\n');

        // Requirements
        if !self.requirements.is_empty() {
            prompt.push_str("\nRequisitos:\n");
            for req in &self.requirements {
                prompt.push_str(&format!("- {}\n", req));
            }
        }

        // Context
        if let Some(ctx) = context {
            prompt.push_str("\nContexto de archivos existentes:\n");
            for file in &ctx.existing_files {
                prompt.push_str(&format!("\n--- {} ---\n{}\n", file.name, file.content));
            }

            if !ctx.dependencies.is_empty() {
                prompt.push_str("\nDependencias disponibles:\n");
                for dep in &ctx.dependencies {
                    prompt.push_str(&format!("- {}\n", dep));
                }
            }
        }

        // User prompt
        prompt.push('\n');
        prompt.push_str(&self.output_format);
        prompt.push_str("\n\n");
        prompt.push_str(user_prompt);

        prompt
    }

    /// Template para Python
    pub fn python() -> Self {
        Self::new(
            "Eres un experto en Python. Genera código Python limpio, eficiente y Pythonic.",
            vec![
                "Seguir PEP 8 style guide".to_string(),
                "Incluir docstrings para funciones y clases".to_string(),
                "Usar type hints para parámetros y retornos".to_string(),
                "Manejar errores apropiadamente con try/except".to_string(),
                "Usar f-strings para string formatting".to_string(),
                "Preferir list comprehensions sobre loops".to_string(),
            ],
            "Genera solo el código Python, sin explicaciones adicionales antes o después.",
        )
    }

    /// Template para JavaScript/TypeScript
    pub fn javascript() -> Self {
        Self::new(
            "Eres un experto en JavaScript/TypeScript moderno. Genera código limpio y mantenible.",
            vec![
                "Usar async/await en vez de callbacks".to_string(),
                "Incluir JSDoc comments para funciones".to_string(),
                "Validar inputs al inicio de funciones".to_string(),
                "Usar ES6+ features (const, let, arrow functions)".to_string(),
                "Preferir immutable data".to_string(),
                "Usar optional chaining (?.) para safe access".to_string(),
            ],
            "Genera solo el código JavaScript/TypeScript, sin explicaciones.",
        )
    }

    /// Template para Rust
    pub fn rust() -> Self {
        Self::new(
            "Eres un experto en Rust. Genera código seguro, eficiente e idiomatico.",
            vec![
                "Usar Result<T, E> para manejo de errores".to_string(),
                "Incluir doc comments (///) para items públicos".to_string(),
                "Seguir Rust idioms y best practices".to_string(),
                "Evitar unwrap() y expect(), usar proper error handling".to_string(),
                "Usar pattern matching apropiadamente".to_string(),
                "Preferir iteradores sobre loops".to_string(),
                "Mencionar lifetime annotations si son necesarias".to_string(),
            ],
            "Genera solo el código Rust, sin explicaciones adicionales.",
        )
    }

    /// Template genérico para otros lenguajes
    pub fn generic() -> Self {
        Self::new(
            "Eres un experto programador. Genera código limpio y bien estructurado.",
            vec![
                "Seguir convenciones del lenguaje".to_string(),
                "Incluir comentarios apropiados".to_string(),
                "Manejar errores correctamente".to_string(),
                "Escribir código legible y mantenible".to_string(),
            ],
            "Genera solo el código, sin explicaciones adicionales.",
        )
    }
}

/// Obtiene el template apropiado para un lenguaje
pub fn get_template(language: &str) -> Option<PromptTemplate> {
    match language.to_lowercase().as_str() {
        "python" | "py" => Some(PromptTemplate::python()),
        "javascript" | "js" | "typescript" | "ts" => Some(PromptTemplate::javascript()),
        "rust" | "rs" => Some(PromptTemplate::rust()),
        _ => Some(PromptTemplate::generic()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_template() {
        let template = PromptTemplate::python();
        let prompt = template.render("Crear función factorial", None);
        
        assert!(prompt.contains("Eres un experto en Python"));
        assert!(prompt.contains("Seguir PEP 8"));
        assert!(prompt.contains("Crear función factorial"));
    }

    #[test]
    fn test_javascript_template() {
        let template = PromptTemplate::javascript();
        let prompt = template.render("Implementar clase User", None);
        
        assert!(prompt.contains("Eres un experto en JavaScript"));
        assert!(prompt.contains("async/await"));
        assert!(prompt.contains("Implementar clase User"));
    }

    #[test]
    fn test_rust_template() {
        let template = PromptTemplate::rust();
        let prompt = template.render("Escribir función main", None);
        
        assert!(prompt.contains("Eres un experto en Rust"));
        assert!(prompt.contains("Result<T, E>"));
        assert!(prompt.contains("Escribir función main"));
    }

    #[test]
    fn test_template_with_context() {
        let template = PromptTemplate::python();
        let context = BuildContext::new()
            .add_file("utils.py", "def helper(): pass")
            .add_dependency("requests");

        let prompt = template.render("Crear función principal", Some(&context));
        
        assert!(prompt.contains("utils.py"));
        assert!(prompt.contains("def helper(): pass"));
        assert!(prompt.contains("requests"));
    }

    #[test]
    fn test_get_template() {
        assert!(get_template("python").is_some());
        assert!(get_template("py").is_some());
        assert!(get_template("javascript").is_some());
        assert!(get_template("js").is_some());
        assert!(get_template("rust").is_some());
        assert!(get_template("unknown").is_some()); // Returns generic
    }
}