# Fase 2: Generacion de Archivos — Servicio Backend

**Duracion:** Semanas 5-7
**Estado:** En desarrollo
**Rama:** `phase/2-file-generation`

---

## Objetivo

Implementar el **servicio backend** de generacion de codigo: un endpoint que recibe un prompt en lenguaje natural y retorna un archivo de codigo valido. El frontend actual es un MVP funcional para probar el servicio, sera reemplazado en Fase 7 por Monaco editor.

---

## Responsabilidades: Fase 2 vs Fase 7

### Fase 2 (ESTA fase) — Servicio de generacion

| Capa | Componente | Estado |
|------|-----------|--------|
| Core | `CodeGenerator` — genera codigo via LLM compartido | Hecho |
| Core | `PromptTemplate` — templates por lenguaje (Python/JS/Rust/generic) | Hecho |
| Core | `SyntaxValidator` — validacion basica post-generacion | Hecho |
| Core | `GenerateRequest`, `GenerationResult`, `BuildContext`, `FileInfo` | Hecho |
| API | `POST /api/generate` — endpoint funcional | Hecho |
| Frontend | `Generate.tsx` — MVP: form + preview + download | Hecho |

**Falta completar:**
- [ ] Context injection real (upload de archivos existentes al servidor)
- [ ] Historial de generaciones en localStorage
- [ ] Tests de integracion para el endpoint

### Fase 7 (OTRA fase) — IDE completo

| Capa | Componente | Estado |
|------|-----------|--------|
| Frontend | Monaco editor (reemplaza Generate.tsx) | Pendiente |
| Frontend | Terminal embebida (xterm.js) | Pendiente |
| Frontend | Tree view de archivos | Pendiente |
| Frontend | Diff viewer | Pendiente |
| Backend | `CodeSandbox::execute()` — Docker sandbox | Pendiente |
| Backend | `POST /api/modify` — editar archivos existentes | Pendiente |
| Backend | Streaming stdout/stderr | Pendiente |
| Seguridad | Auditoria, rate limiting, approval workflow | Pendiente |

**Clave:** El servicio `CodeGenerator` de Fase 2 se reutiliza tal cual en Fase 7. No se borra nada, se construye encima.

---

## Arquitectura del Servicio

```
Usuario (prompt + lenguaje)
    │
    ▼
POST /api/generate
    │
    ├─ generate_handler()          [crates/api/src/handlers.rs]
    │   └─ CodeGenerator::new(llm) [reutiliza LLM de AppState]
    │
    ├─ CodeGenerator.generate()    [crates/core/src/generator/engine.rs]
    │   ├─ get_template(lang)      [templates.rs]
    │   ├─ template.render()       [compila prompt con context]
    │   ├─ llm.generate_code()     [LLMBackend compartido]
    │   ├─ SyntaxValidator::validate() [validation.rs]
    │   ├─ suggest_filename()      [heuristica 3 palabras]
    │   └─ analyze_generation()    [analisis estatico basico]
    │
    └─ Retorna GenerationResult
        {file_name, content, language, explanation, suggestions}
```

### Puntos criticos del diseno

1. **LLM compartido:** `CodeGenerator` recibe `Arc<LLMBackend>` de `AppState`. No crea instancias nuevas por request. Reutiliza el modelo ya cargado.

2. **Validacion integrada:** `SyntaxValidator` se ejecuta despues de generar. Los warnings se agregan a `suggestions` del resultado.

3. **Templates separados del generador:** `PromptTemplate` es un modulo independiente. Agregar un nuevo lenguaje es solo crear un template.

4. **Context injection (pendiente):** `BuildContext` esta definido pero no se consume desde el endpoint. El frontend actual no envia archivos existentes.

---

## Tareas Pendientes (Semanas 6-7)

### Semana 6: Completar servicio

- [ ] Context injection: recibir archivos existentes en el request, inyectar en el template
- [ ] Historial: persistir generaciones en localStorage, mostrar en UI
- [ ] Tests: 5 tests de integracion para POST /api/generate

### Semana 7: Validacion y documentacion

- [ ] Integrar SyntaxValidator como campo en la respuesta (no solo suggestions)
- [ ] Documentar API endpoint en docs/api.md
- [ ] Code review y refactor

---

## Archivos

| Archivo | Responsabilidad |
|---------|----------------|
| `crates/core/src/generator/mod.rs` | Tipos: GenerateRequest, GenerationResult, BuildContext, FileInfo |
| `crates/core/src/generator/engine.rs` | CodeGenerator: logica principal de generacion |
| `crates/core/src/generator/templates.rs` | PromptTemplate: templates por lenguaje |
| `crates/core/src/generator/validation.rs` | SyntaxValidator: validacion basica de sintaxis |
| `crates/api/src/handlers.rs` | generate_handler: endpoint HTTP |
| `webui/src/pages/Generate.tsx` | MVP frontend (reemplazado en Fase 7) |

---

## Criterios de Aceptacion (Fase 2)

### Completados
- [x] Genera codigo Python/JS/Rust desde prompt
- [x] LLM reutilizado via AppState (no crea instancias nuevas)
- [x] Validacion de sintaxis integrada
- [x] Frontend funcional con preview, copy, download
- [x] 28 tests unitarios pasando
- [x] Clippy sin warnings

### Pendientes
- [ ] Context injection funcional
- [ ] Historial persistente
- [ ] 5+ tests de integracion

---

**Inicio:** 2026-07-16
**Fin estimado:** 2026-08-05
