//! Tests de integracion para el servicio de generacion de codigo
//!
//! Estos tests validan el pipeline completo del generador:
//! - Templates por lenguaje
//! - Renderizado de prompts con contexto
//! - Validacion de sintaxis post-generacion
//! - Sugerencia de nombres de archivo
//! - Inyeccion de contexto de archivos existentes

use alesys_core::generator::{
    BuildContext, FileInfo, GenerateRequest, GenerationResult, PromptTemplate, SyntaxValidator,
};

// === Templates ===

#[test]
fn test_template_python_renderiza_prompt() {
    let template = PromptTemplate::python();
    let prompt = template.render("Crear API REST", None);
    assert!(prompt.contains("Python"));
    assert!(prompt.contains("PEP 8"));
    assert!(prompt.contains("Crear API REST"));
}

#[test]
fn test_template_javascript_renderiza_prompt() {
    let template = PromptTemplate::javascript();
    let prompt = template.render("Crear componente React", None);
    assert!(prompt.contains("JavaScript"));
    assert!(prompt.contains("async/await"));
    assert!(prompt.contains("Crear componente React"));
}

#[test]
fn test_template_rust_renderiza_prompt() {
    let template = PromptTemplate::rust();
    let prompt = template.render("Implementar trait", None);
    assert!(prompt.contains("Rust"));
    assert!(prompt.contains("Result"));
    assert!(prompt.contains("Implementar trait"));
}

#[test]
fn test_template_generic_para_lenguaje_desconocido() {
    let template = PromptTemplate::generic();
    let prompt = template.render("Escribir logica", None);
    assert!(prompt.contains("programador"));
    assert!(prompt.contains("Escribir logica"));
}

// === Context Injection ===

#[test]
fn test_context_injection_archivos_existentes() {
    let template = PromptTemplate::python();
    let context = BuildContext {
        project_type: Some("library".to_string()),
        existing_files: vec![
            FileInfo {
                name: "utils.py".to_string(),
                content: "def helper(): pass".to_string(),
            },
            FileInfo {
                name: "models.py".to_string(),
                content: "class User: pass".to_string(),
            },
        ],
        dependencies: vec!["requests".to_string(), "sqlalchemy".to_string()],
    };

    let prompt = template.render("Crear endpoint /users", Some(&context));

    // Verificar que los archivos existentes aparecen en el prompt
    assert!(prompt.contains("utils.py"));
    assert!(prompt.contains("def helper(): pass"));
    assert!(prompt.contains("models.py"));
    assert!(prompt.contains("class User: pass"));

    // Verificar que las dependencias aparecen
    assert!(prompt.contains("requests"));
    assert!(prompt.contains("sqlalchemy"));

    // Verificar que el tipo de proyecto aparece
    assert!(prompt.contains("library"));
}

#[test]
fn test_context_vacio_no_agrega_nada() {
    let template = PromptTemplate::python();
    let context = BuildContext::new();

    let prompt_with = template.render("test", Some(&context));
    let prompt_without = template.render("test", None);

    // Context vacio no deberia agregar contenido extra significativo
    assert_eq!(prompt_with.len(), prompt_without.len());
}

// === Syntax Validation ===

#[test]
fn test_validate_python_codigo_valido() {
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

class Calculator:
    def add(self, a, b):
        return a + b
"#;
    assert!(SyntaxValidator::validate_python(code).is_ok());
}

#[test]
fn test_validate_python_parentesis_desbalanceados() {
    let code = "def foo(\n    print('hello')\n";
    assert!(SyntaxValidator::validate_python(code).is_err());
}

#[test]
fn test_validate_javascript_codigo_valido() {
    let code = r#"
function factorial(n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

const greet = (name) => {
    return `Hello ${name}`;
};
"#;
    assert!(SyntaxValidator::validate_javascript(code).is_ok());
}

#[test]
fn test_validate_javascript_template_literal_desbalanceado() {
    let code = "const x = `hello;";
    assert!(SyntaxValidator::validate_javascript(code).is_err());
}

#[test]
fn test_validate_rust_codigo_valido() {
    let code = r#"
fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

struct Point {
    x: f64,
    y: f64,
}
"#;
    assert!(SyntaxValidator::validate_rust(code).is_ok());
}

#[test]
fn test_validate_rust_llaves_desbalanceadas() {
    let code = "fn main() {\n    println!(\"hello\");\n";
    assert!(SyntaxValidator::validate_rust(code).is_err());
}

#[test]
fn test_validate_dispatch_por_lenguaje() {
    assert!(SyntaxValidator::validate("def f(): pass", "python").is_ok());
    assert!(SyntaxValidator::validate("function f() {}", "javascript").is_ok());
    assert!(SyntaxValidator::validate("fn f() {}", "rust").is_ok());
    assert!(SyntaxValidator::validate("anything", "unknown").is_ok());
}

// === GenerateRequest ===

#[test]
fn test_generate_request_builder() {
    let req = GenerateRequest::new("Crear clase", "python").with_max_tokens(4096);
    assert_eq!(req.prompt, "Crear clase");
    assert_eq!(req.language, "python");
    assert_eq!(req.max_tokens, 4096);
    assert!(req.context.is_none());
}

#[test]
fn test_generate_request_con_contexto() {
    let ctx = BuildContext::new()
        .add_file("main.py", "print('hello')")
        .add_dependency("numpy");

    let req = GenerateRequest::new("Agregar funcionalidad", "python").with_context(ctx);

    assert!(req.context.is_some());
    let ctx = req.context.unwrap();
    assert_eq!(ctx.existing_files.len(), 1);
    assert_eq!(ctx.existing_files[0].name, "main.py");
    assert_eq!(ctx.dependencies.len(), 1);
    assert_eq!(ctx.dependencies[0], "numpy");
}

// === GenerationResult ===

#[test]
fn test_generation_result_serialization() {
    let result = GenerationResult {
        file_name: "test.py".to_string(),
        content: "def test(): pass".to_string(),
        language: "python".to_string(),
        explanation: "Generado 1 linea de Python".to_string(),
        suggestions: vec!["Agregar tests".to_string()],
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("test.py"));
    assert!(json.contains("Python"));
    assert!(json.contains("Agregar tests"));
}
