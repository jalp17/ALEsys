---
fecha: 2026-07-19
tipo: plan
proyecto: alesys
fase: 15
tags: [alesys, ai-pair-programmer, code-analysis, phase15]
status: en-progreso
---

# Phase 15: AI Pair Programmer Avanzado

## Objetivo
IA que programa activamente, no solo genera código.

## Arquitectura

### Componentes

1. **Context Analyzer** (`crates/core/src/pair_programmer/analyzer.rs`)
   - Lee TODO el proyecto
   - Entiende estructura y dependencias
   - Identifica patrones y convenciones
   - Detecta código duplicado

2. **Suggestion Engine** (`crates/core/src/pair_programmer/suggestions.rs`)
   - Sugiere mejoras proactivas
   - Detecta código duplicado
   - Identifica módulos sin tests
   - Sugiere refactorizaciones

3. **Auto-Refactor** (`crates/core/src/pair_programmer/refactor.rs`)
   - Aplica refactorizaciones seguras
   - Extrae funciones/métodos
   - Renombra variables
   - Optimiza imports

4. **Debug Assistant** (`crates/core/src/pair_programmer/debugger.rs`)
   - Analiza logs de error
   - Identifica root cause
   - Propone fixes
   - Aplica patches

5. **Test Generator** (`crates/core/src/pair_programmer/test_gen.rs`)
   - Genera tests unitarios
   - Genera tests de integración
   - Analiza cobertura

### Flujo de Uso

```
Usuario escribe código
    ↓
Context Analyzer lee proyecto
    ↓
Suggestion Engine identifica mejoras
    ↓
Auto-Refactor aplica cambios seguros
    ↓
Test Generator crea tests
    ↓
Debug Assistant analiza errores
```

## Fases

### 15.1 Analysis + Suggestions (v1.24.0)
- [ ] Context Analyzer: lee proyecto, entiende estructura
- [ ] Suggestion Engine: detecta mejoras
- [ ] Code smell detection
- [ ] Tests unitarios

### 15.2 Refactor + Debug + UI (v1.25.0)
- [ ] Auto-Refactor: aplica cambios seguros
- [ ] Debug Assistant: analiza errores
- [ ] Test Generator: crea tests
- [ ] UI: suggestions panel en editor
- [ ] Tests de integración

## Criterios de Éxito

- [ ] Sugiere 3+ mejoras relevantes por sesión
- [ ] Auto-refactor no rompe tests existentes
- [ ] Debug assistant fija 80% de bugs comunes
- [ ] Tests generados tienen > 80% coverage

---

**Tags:** #alesys #plan #ai-pair-programmer #phase15
