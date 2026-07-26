# Contribuir a ALEsys

¡Gracias por tu interés en contribuir a ALEsys! Este documento explica cómo empezar.

## Guías Rápidas

### Para Bug Reports

1. Busca issues existentes
2. Si no existe, crea una nueva usando el template
3. Incluye pasos para reproducir, comportamiento esperado y actual

### Para Features

1. Abre un issue discutiendo la feature
2. Espera aprobación antes de empezar a programar
3. Sigue el flujo de trabajo de ramas (ver BRANCH_STRATEGY.md)

### Para Pull Requests

1. Fork el repositorio
2. Crea una rama desde `main` o `phase-*`
3. Haz cambios con commits atómicos
4. Asegura que tests pasen
5. Abre un PR describiendo cambios

## Estrategia de Ramas

Ver `BRANCH_STRATEGY.md` para detalles completos.

### Resumen

```
main (estable)
├── phase-1-chat (integración Fase 1)
│   ├── feature/1-core-hybrid-search
│   ├── feature/1-api-chat-endpoint
│   └── feature/1-webui-chat-ui
└── phase-2-generate (integración Fase 2)
```

### Flujo

1. Crear feature desde phase
2. Desarrollar con commits atómicos
3. Merge feature → phase
4. Merge phase → main (cuando completa)

## Convenciones de Código

### Rust

- Sigue `rustfmt` (configurado en rustfmt.toml)
- Usa `clippy` para linting
- Escribe tests para código nuevo
- Documenta funciones públicas

### TypeScript/React

- Sigue `eslint` y `prettier`
- Usa componentes funcionales con hooks
- Escribe tests con Vitest/React Testing Library
- Usa TypeScript estricto

### PHP

- Sigue PSR-12
- Usa PHPDoc para documentación
- Escribe tests con PHPUnit

### Git

- Commits atómicos
- Mensajes descriptivos en inglés
- Referencia issues: `fix #123`, `closes #456`

## Estructura del Proyecto

```
ALEsys/
├── crates/           # Backend Rust
│   ├── core/        # Lógica de negocio
│   ├── api/         # API REST + WebSocket
│   └── cli/         # CLI
├── webui/           # Frontend React
├── server/          # PHP backend
├── docker/          # Docker configs
└── scripts/         # Scripts de utilidad
```

## Requisitos de Desarrollo

### Herramientas

- Rust 1.80+
- Node.js 20+
- pnpm 9+
- Docker + Docker Compose
- PostgreSQL 16

### Setup

```bash
# Clonar
git clone https://github.com/tu-usuario/ALEsys
cd ALEsys

# Instalar dependencias
./scripts/setup-dev.sh

# Iniciar desarrollo
pnpm dev
```

## Tests
### Metodología

| Tipo | Comando | Cuándo |
|------|---------|--------|
| Unitarios | `cargo test -p alesys-core --lib` | Cada cambio en lógica |
| Integración | `cargo test -p alesys-api --test <suite>` | Backend + DB |
| E2E | `python3 tests/e2e/ingestion_test.py` | Pipeline completo |
| Benchmarks | `cargo bench -p alesys-core` | Regresión performance |

### Correr Todos los Tests

```bash
# Unit tests (core)
cargo test -p alesys-core --lib

# API tests
cargo test -p alesys-api

# E2E offline (sin backend)
python3 tests/e2e/ingestion_test.py --offline

# E2E live (con backend)
python3 tests/e2e/ingestion_test.py

# Benchmarks
cargo bench -p alesys-core ingestion_bench
```

### Cobertura

```bash
# Rust (instalar cargo-tarpaulin)
cargo tarpaulin --workspace

# WebUI
cd webui && pnpm test:coverage
```

## Metodología de Tickets y Flujo de Trabajo

### Formato de Tickets

Cada fase utiliza archivos de issues en `.github/`:

- **Formato:** `TICKET-{FASE}.{NUM}` (ej: `TICKET-29.1`, `TICKET-29.2`)
- **Tracking:** `.github/FaseXX-ISSUES.md`
- **Workflow:** feature branch → fase branch → main

### Flujo por Fase

1. **Planificar:** Crear `FaseXX-ISSUES.md` con 8-10 tickets
2. **Implementar:** Branches `feature/{N}-{area}-{desc}`
3. **Verificar:** Tests unitarios, integración, e2e, benchmarks
4. **Cerrar:** Marcar tickets completados en `FaseXX-ISSUES.md`
5. **Integrar:** Merge fase a `main` + tag semántico

### Comandos Útiles

```bash
# Ver tickets de una fase
cat .github/Fase29-ISSUES.md

# Crear issue desde ticket
gh issue create --title "TICKET-29.1: ..." --body-file .github/ISSUE_TEMPLATES/TICKET-29.1.md
```

### Referencias

- [Estrategia de Ramas](BRANCH_STRATEGY.md)
- [CHANGELOG.md](CHANGELOG.md)
- [Issues Fase 29](.github/Fase29-ISSUES.md)

## Documentación

- Mantén `README.md` actualizado
- Documenta APIs públicas
- Actualiza `AGENT.md` con progreso
- Usa comentarios para código complejo

## Ingesta de PDFs (Fase 29)
### Desarrollo

```bash
# Tests unitarios del pipeline de ingesta
cargo test -p alesys-core --lib ingestion::

# Integración (API + DB)
cargo test -p alesys-api --test ingestion_db_test

# E2E tests (requiere servidor corriendo)
python3 tests/e2e/ingestion_test.py

# E2E offline (sin backend)
python3 tests/e2e/ingestion_test.py --offline

# Benchmarks
cargo bench -p alesys-core ingestion_bench

# Setup MinerU
./scripts/setup-mineru.sh
```

### Tickets

Ver `.github/Fase29-ISSUES.md` para los 10 tickets completados:
- TICKET-29.1: Plugin Skeleton + Config
- TICKET-29.2: MinerUWrapper
- TICKET-29.3: PyMuPDFFallback
- TICKET-29.4: Organizer
- TICKET-29.5: PDFProcessor Orchestrator
- TICKET-29.6: API Endpoints + WebSocket
- TICKET-29.7: Frontend Ingestion Panel
- TICKET-29.8: GraphRAG Integration Hook
- TICKET-29.9: Tests E2E + Benchmarks
- TICKET-29.10: Docs + Scripts

### Estructura del código

```
crates/core/src/ingestion/
├── mod.rs              # Module root, IngestionConfig
├── models.rs           # IngestionJob, IngestionResult, Chapter
├── plugin.rs           # IngestionPlugin system
├── mineru_wrapper.rs   # MinerU GPU/CPU wrapper
├── pymupdf_fallback.rs # PyMuPDF fallback extractor
├── organizer.rs        # Output cleanup & structure
├── pdf_processor.rs    # Orchestrator + GraphRAG hook
├── progress.rs         # ProgressTracker wrapper
└── tests/              # Unit tests

crates/core/src/graphrag/
├── ingestion_hook.rs   # Post-ingestion GraphRAG indexing

crates/api/
├── src/handlers_ingestion.rs  # REST + WS handlers
├── src/state.rs               # Auto-run migrations
└── tests/ingestion_db_test.rs # PostgreSQL integration

tests/e2e/
└── ingestion_test.py          # 7 escenarios E2E

benches/
└── ingestion_bench.rs         # Benchmarks
```

### Convenciones

- Chunking: max 512 tokens, overlap 200 tokens
- Embeddings: ONNX Runtime batch 32
- Metadatos: topic, session_id, source_pdf, chapter_title
- Auth: JWT + RBAC (`ingestion:write`, `ingestion:read`)
- Tracking: DB-backed `ingestion_jobs` para persistencia

## Pull Requests

### Antes de Abrir un PR

- [ ] Tests pasando locally
- [ ] Código formateado (`cargo fmt`, `pnpm format`)
- [ ] Sin warnings (`cargo clippy`, `pnpm lint`)
- [ ] Documentación actualizada
- [ ] Commits limpios (squash si necesario)

### Formato del PR

```markdown
## Descripción
[Descripción breve de cambios]

## Tipo de Cambio
- [ ] Bug fix
- [ ] Nueva feature
- [ ] Breaking change
- [ ] Documentación

## Testing
- [ ] Tests unitarios
- [ ] Tests de integración
- [ ] Manual testing

## Checklist
- [ ] Código compila sin errores
- [ ] Tests pasan
- [ ] Documentación actualizada
- [ ] No hay secrets hardcodeados
```

## Code Review

### Como Reviewer

1. Revisa cambios por completitud
2. Verifica que tests estén incluidos
3. Busca problemas de seguridad
4. Asegura consistencia con el código existente
5. Aprueba o solicita cambios

### Como Author

1. Resuelve todos los comentarios
2. Pide re-review después de cambios
3. No haces force push después de review

## Issues

### Labels

| Label | Descripción |
|-------|-------------|
| `bug` | Bug report |
| `feature` | Feature request |
| `docs` | Documentación |
| `enhancement` | Mejora existente |
| `help wanted` | Necesita ayuda |
| `good first issue` | Bueno para principiantes |
| `priority: high` | Alta prioridad |
| `priority: low` | Baja prioridad |

### Milestones

| Milestone | Descripción |
|-----------|-------------|
| Phase 1 | Chat con GraphRAG |
| Phase 2 | Generación de archivos |
| Phase 3 | Sesiones multi-usuario |
| ... | ... |

## Comunidad

### Comunicación

- **GitHub Issues**: Discusiones técnicas
- **Discord**: Chat en tiempo real (si se crea)
- **Twitter**: Actualizaciones (@alesys_dev)

### Comportamiento

- Sé respetuoso y profesional
- Enfócate en el código, no en la persona
- Acepta feedback constructivo
- Ayuda a otros cuando puedas

## Licencia

Al contribuir, aceptas que tu código sea licenciado bajo AGPL v3.0.

## Agradecimientos

¡Gracias por contribuir a ALEsys! Tu ayuda es muy apreciada.

---

**Tags:** #contributing #development #alesys