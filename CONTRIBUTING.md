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

### Correr Todos los Tests

```bash
# Rust
cargo test --workspace

# WebUI
cd webui && pnpm test

# PHP
cd server && composer test
```

### Cobertura

```bash
# Rust (instalar cargo-tarpaulin)
cargo tarpaulin --workspace

# WebUI
cd webui && pnpm test:coverage
```

## Documentación

- Mantén `README.md` actualizado
- Documenta APIs públicas
- Actualiza `AGENT.md` con progreso
- Usa comentarios para código complejo

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