# 📋 Fase 2: Generación de Archivos

**Duración:** Semanas 5-7  
**Estado:** 🟡 En desarrollo  
**Rama:** `phase/2-file-generation`

---

## 🎯 Objetivo

Implementar un endpoint de generación de código que permita a los usuarios generar archivos completos desde prompts naturales, con preview, edición y descarga.

---

## 📦 Entregables

### 1. Backend Rust (`crates/api`)

#### Endpoint `POST /api/generate`
- **Request:**
  ```json
  {
    "prompt": "Crear una función Python que calcule el factorial",
    "language": "python",
    "context": {
      "project_type": "library",
      "existing_files": ["utils.py", "config.py"]
    }
  }
  ```

- **Response:**
  ```json
  {
    "file_name": "factorial.py",
    "content": "def factorial(n):\n    ...\n",
    "language": "python",
    "explanation": "Generado con patrón de función recursiva...",
    "suggestions": ["Agregar tests", "Añadir docstring"]
  }
  ```

#### Componentes:
- `GenerateRequest` y `GenerateResponse` types
- `CodeGenerator` service con prompt templates
- Context injection desde archivos existentes
- Validación básica de sintaxis (opcional)

---

### 2. Frontend (`webui`)

#### Página `/generate`
- **Componente `GeneratePage`:**
  - Editor de prompt (textarea con auto-completado)
  - Selector de lenguaje (Python, JS, Rust, etc.)
  - Upload de archivos de contexto (opcional)
  - Botón "Generar"

- **Componente `CodePreview`:**
  - Syntax highlighting (Prism.js o similar)
  - Botón "Copiar"
  - Botón "Descargar"
  - Botón "Editar" (abre editor inline)

- **Componente `GenerationHistory`:**
  - Lista de generaciones recientes
  - Click para re-abrir
  - Búsqueda en historial

---

### 3. PHP Server (`server`)

#### Proxy `/api/generate`
- Forward al backend Rust
- Session context injection
- Rate limiting por usuario

#### Gestión de sesiones
- Guardar historial de generaciones en DB
- Associar generaciones al usuario actual

---

## 📝 Tareas Detalladas

### Semana 5: Backend Core

#### Día 1-2: Tipos y Estructura
- [ ] Definir `GenerateRequest` y `GenerateResponse` en `crates/api/src/routes/generate.rs`
- [ ] Crear `CodeGenerator` struct en `crates/core/src/generator/mod.rs`
- [ ] Implementar prompt templates básicos

#### Día 3-4: LLM Integration
- [ ] Integrar con backend LLM existente (llama.cpp)
- [ ] Implementar `CodeGenerator::generate(prompt, language)`
- [ ] Añadir context injection desde archivos

#### Día 5: Endpoint API
- [ ] Implementar `POST /api/generate` handler
- [ ] Añadir validación de input
- [ ] Error handling y logging

### Semana 6: Frontend

#### Día 1-2: Layout Base
- [ ] Crear `GeneratePage.tsx` con layout básico
- [ ] Añadir `PromptEditor` component
- [ ] Selector de lenguaje

#### Día 3-4: Preview y Descarga
- [ ] Implementar `CodePreview` con syntax highlighting
- [ ] Añadir botones "Copiar" y "Descargar"
- [ ] Implementar `downloadFile(filename, content)`

#### Día 5: Historial
- [ ] Crear `GenerationHistory` component
- [ ] Persistir historial en localStorage
- [ ] Búsqueda en historial

### Semana 7: Integración y PHP

#### Día 1-2: PHP Integration
- [ ] Crear proxy `/api/generate` en PHP
- [ ] Añadir session context
- [ ] Rate limiting básico

#### Día 3-4: Testing
- [ ] Tests de integración para endpoint
- [ ] E2E tests para flujo completo
- [ ] Test con 10 prompts diferentes

#### Día 5: Documentación y Refactor
- [ ] Documentar API endpoint
- [ ] Añadir ejemplos de uso
- [ ] Code review y refactor

---

## 🧪 Criterios de Aceptación

### Funcional
- [ ] Genera archivo Python válido desde prompt
- [ ] Genera archivo JavaScript válido desde prompt
- [ ] Genera archivo Rust válido desde prompt
- [ ] Usuario puede descargar archivo generado
- [ ] Historial persiste por sesión (localStorage)
- [ ] Context injection funciona con archivos existentes

### Performance
- [ ] Generación completa en < 10 segundos
- [ ] Preview renderiza en < 100ms
- [ ] Descarga inmediata (sin delay)

### Testing
- [ ] 10/10 tests de integración pasan
- [ ] 5/5 E2E tests pasan
- [ ] Código pasa clippy sin warnings

---

## 🔗 Dependencias

- **Fase 1:** Chat básico con GraphRAG (en progreso)
- **Backend LLM:** llama.cpp configurado y funcionando
- **Frontend:** WebUI base con routing

---

## 📌 Notas Técnicas

### Prompt Templates

**Python:**
```
Eres un experto en Python. Genera código Python limpio y Pythonic.

Requisitos:
- Seguir PEP 8
- Incluir docstrings
- Usar type hints
- Manejar errores apropiadamente

Prompt: {prompt}

Contexto de archivos existentes:
{context}

Genera solo el código, sin explicaciones adicionales.
```

**JavaScript:**
```
Eres un experto en JavaScript/TypeScript. Genera código moderno y limpio.

Requisitos:
- Usar async/await en vez de callbacks
- Incluir JSDoc comments
- Validar inputs
- Usar ES6+ features

Prompt: {prompt}

Contexto:
{context}

Genera solo el código.
```

**Rust:**
```
Eres un experto en Rust. Genera código seguro e idiomatico.

Requisitos:
- Usar Result para errores
- Incluir doc comments
- Seguir Rust idioms
- Evitar unwrap()

Prompt: {prompt}

Contexto:
{context}

Genera solo el código.
```

---

## 🚧 Bloqueos Potenciales

| Bloqueo | Impacto | Mitigación |
|---------|---------|------------|
| LLM no genera código válido | Alto | Ajustar temperature, few-shot examples |
| Syntax highlighting lento | Bajo | Usar Prism.js en vez de highlight.js |
| Context injection complejidad | Medio | Limitar a 3 archivos máximo |

---

**Fecha de inicio:** 2026-07-16  
**Fecha estimada de fin:** 2026-08-05  
**Responsable:** ALEsys Development Team