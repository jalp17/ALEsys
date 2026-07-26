# Estrategia de Ramas - ALEsys

## Visión General

ALEsys utiliza una estrategia de **ramas por fase + features por área** para garantizar estabilidad y un desarrollo organizado.

## Estructura de Ramas

```
main (estable, solo código probado)
│
├── phase-1-chat (integración Fase 1: Semanas 1-4)
│   │
│   ├── feature/1-core-hybrid-search
│   ├── feature/1-core-mistralrs-integration
│   ├── feature/1-core-onnx-embeddings
│   ├── feature/1-api-chat-endpoint
│   ├── feature/1-api-websocket-streaming
│   └── feature/1-webui-chat-ui
│
├── phase-2-generate (integración Fase 2: Semanas 5-7)
│   │
│   ├── feature/2-core-generation-engine
│   ├── feature/2-api-generate-endpoint
│   └── feature/2-webui-generate-ui
│
└── phase-3-sessions (integración Fase 3: Semanas 8-9)
    │
    ├── feature/3-core-session-manager
    ├── feature/3-php-auth-system
    └── feature/3-webui-session-ui
```

## Reglas de Naming

### Ramas de Fase

```
phase-{N}-{nombre-corto}
```

Ejemplos:
- `phase-1-chat`
- `phase-2-generate`
- `phase-3-sessions`
- `phase-4-optimization`

### Ramas de Feature

```
feature/{N}-{area}-{descripcion}
```

Ejemplos:
- `feature/1-core-hybrid-search`
- `feature/1-api-chat-endpoint`
- `feature/1-webui-chat-ui`
- `feature/3-php-auth-system`

### Áreas de Desarrollo

| Prefix | Área | Descripción |
|--------|------|-------------|
| `core` | Rust Core | Lógica de negocio, LLM, embeddings, grafos |
| `api` | API REST | Endpoints, WebSocket, middleware |
| `webui` | Frontend React | Componentes, páginas, layouts |
| `php` | PHP Backend | Auth, proxy, sesiones multi-usuario |
| `infra` | Infraestructura | Docker, CI/CD, configs, scripts |
| `tauri` | Desktop Wrapper | Tauri, system tray, filesystem |

## Flujo de Trabajo

### 1. Crear Feature desde Fase

```bash
# Desde rama de fase
git checkout phase-1-chat
git checkout -b feature/1-core-hybrid-search

# O usando el script
./scripts/new-feature.sh 1 core hybrid-search
```

### 2. Desarrollar Feature

```bash
# Trabajar en la feature
# ...
# Commits atómicos
git commit -m "feat(core): implement hybrid search with pgvector"
git commit -m "test(core): add unit tests for hybrid search"
```

### 3. Merge Feature → Fase

```bash
# Cuando la feature esté completa y testeada
git checkout phase-1-chat
git merge feature/1-core-hybrid-search
git push origin phase-1-chat
```

### 4. Merge Fase → Main

```bash
# Cuando TODAS las features de la fase estén completas
git checkout main
git merge phase-1-chat
git tag -a v1.0.0 -m "Release v1.0.0: Fase 1 - Chat con GraphRAG"
git push origin main --tags
```

## Tickets por Fase

Cada fase se rastrea en `.github/FaseXX-ISSUES.md`:

- **Formato:** `TICKET-{FASE}.{NUM}` (ej: `TICKET-29.1`)
- **Workflow:** Crear fase → feature branches → merge fase → main
- **Cierre:** Todos los tickets marcados + merge a main + tag

Ver `.github/Fase29-ISSUES.md` como ejemplo de metodología.

## Checklist por Feature

### Pre-merge (feature → phase)

- [ ] Código compilando sin warnings
- [ ] Tests unitarios pasando
- [ ] Tests de integración pasando
- [ ] Documentación actualizada
- [ ] AGENT.md actualizado con progreso
- [ ] Tickets actualizados en FaseXX-ISSUES.md
- [ ] Variables de entorno documentadas
- [ ] Migraciones de DB (si aplica)

### Pre-release (phase → main)

- [ ] Todas las features de la fase completas
- [ ] Tests E2E pasando
- [ ] Performance benchmarks (si aplica)
- [ ] Documentación de usuario actualizada
- [ ] Tag semántico (v1.0.0, v1.1.0, etc.)
- [ ] CHANGELOG.md actualizado

## Convenciones de Commits

### Formato

```
{type}({scope}): {description}

[optional body]

[optional footer]
```

### Tipos

| Type | Descripción |
|------|-------------|
| `feat` | Nueva feature |
| `fix` | Bug fix |
| `docs` | Documentación |
| `style` | Formato (no afecta lógica) |
| `refactor` | Refactorización |
| `test` | Tests |
| `chore` | Tareas de mantenimiento |
| `perf` | Mejora de rendimiento |
| `ci` | CI/CD |

### Scopes

| Scope | Descripción |
|-------|-------------|
| `core` | Rust core logic |
| `api` | API REST/WebSocket |
| `webui` | Frontend React |
| `php` | PHP backend |
| `infra` | Infraestructura |
| `tauri` | Desktop wrapper |

### Ejemplos

```
feat(core): implement hybrid search with pgvector
fix(api): handle WebSocket disconnection gracefully
docs(readme): update installation instructions
test(core): add unit tests for session manager
refactor(webui): extract chat component into separate module
perf(core): optimize embedding cache lookup
chore(infra): update Docker Compose for production
```

## Tags Semánticos

### Formato

```
v{MAJOR}.{MINOR}.{PATCH}
```

### Reglas

- **MAJOR**: Cambios incompatibles con versiones anteriores
- **MINOR**: Nuevas features compatibles hacia atrás
- **PATCH**: Bug fixes compatibles hacia atrás

### Ejemplos

- `v1.0.0` - Fase 1 completa (primera release estable)
- `v1.1.0` - Fase 1 + mejoras menores
- `v2.0.0` - Fase 2 completa (nuevas features significativas)

## Protección de Ramas

### main

- ✅ Requiere aprobación de 1 reviewer
- ✅ Requiere tests pasando
- ✅ No permite force push
- ✅ No permite deletion
- ✅ Requiere branch actualizado

### phase-*

- ✅ Requiere tests pasando
- ✅ No permite force push
- ✅ Permite deletion después de merge a main

### feature/*

- ✅ No requiere reviews (desarrollo solo)
- ✅ Requiere tests pasando
- ✅ Permite force push
- ✅ Se elimina después de merge a phase

## Comandos Útiles

```bash
# Listar ramas de fase
git branch -a | grep phase

# Listar ramas de feature activas
git branch -a | grep feature

# Ver progreso de una fase
git log phase-1-chat --oneline --graph

# Ver features completadas
git log main --oneline --grep="Merge branch 'feature/"

# Eliminar feature mergeada
git branch -d feature/1-core-hybrid-search
git push origin --delete feature/1-core-hybrid-search
```

---

**Tags:** #git #branches #strategy #alesys