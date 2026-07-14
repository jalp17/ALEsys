# Protección de Ramas - ALEsys

Este archivo documenta las reglas de protección de ramas configuradas en GitHub.

## Configuración en GitHub

### Rama `main`

**Ubicación:** Settings → Branches → Branch protection rules → Add rule

```
Branch name pattern: main

☑ Require a pull request before merging
  ☑ Require approvals: 1
  ☑ Dismiss stale pull request approvals when new commits are pushed
  ☑ Require review from Code Owners

☑ Require status checks to pass before merging
  ☑ Require branches to be up to date before merging
  Required status checks:
    - Lint & Format
    - Rust Tests
    - WebUI Tests
    - Security Scan
    - Docker Build

☑ Require conversation resolution before merging

☑ Require linear history
  ☑ Require squash merging

☑ Do not allow bypassing the above settings

☑ Restrict who can push to matching branches
  Restrictions: @alesys-dev (solo para hotfixes)

☑ Force push: ❌ Not allowed
☑ Deletions: ❌ Not allowed
```

### Ramas `phase-*`

**Ubicación:** Settings → Branches → Branch protection rules → Add rule

```
Branch name pattern: phase-*

☑ Require status checks to pass before merging
  ☑ Require branches to be up to date before merging
  Required status checks:
    - Lint & Format
    - Rust Tests
    - WebUI Tests

☑ Do not allow bypassing the above settings

☑ Force push: ❌ Not allowed
☑ Deletions: ✅ Allowed (después de merge a main)
```

### Ramas `feature/*`

**Configuración por defecto:** Sin protección (desarrollo libre)

```
Branch name pattern: feature/*

☑ Force push: ✅ Allowed
☑ Deletions: ✅ Allowed (después de merge a phase)
```

## Reglas de Merge

### Feature → Phase

**Pre-requisitos:**
1. ✅ Código compilando sin warnings
2. ✅ Tests unitarios pasando
3. ✅ Tests de integración pasando
4. ✅ Documentación actualizada
5. ✅ Variables de entorno documentadas

**Proceso:**
```bash
# 1. Checkout a la fase
git checkout phase-1-chat

# 2. Merge la feature
git merge feature/1-core-hybrid-search

# 3. Push
git push origin phase-1-chat

# 4. Limpiar feature local
git branch -d feature/1-core-hybrid-search

# 5. Limpiar feature remota
git push origin --delete feature/1-core-hybrid-search
```

### Phase → Main

**Pre-requisitos:**
1. ✅ TODAS las features de la fase completas
2. ✅ Tests E2E pasando
3. ✅ Performance benchmarks (si aplica)
4. ✅ Documentación de usuario actualizada
5. ✅ Tag semántico listo

**Proceso:**
```bash
# 1. Checkout main
git checkout main
git pull origin main

# 2. Merge phase
git merge phase-1-chat

# 3. Tag release
git tag -a v1.0.0 -m "Release v1.0.0: Fase 1 - Chat con GraphRAG"

# 4. Push
git push origin main --tags

# 5. Limpiar phase local
git branch -d phase-1-chat

# 6. Limpiar phase remota
git push origin --delete phase-1-chat
```

## CI/CD Checks

### Checks Requeridos para Phase → Main

| Check | Descripción | Timeout |
|-------|-------------|---------|
| Lint & Format | Clippy, ESLint, formateo | 10 min |
| Rust Tests | Tests unitarios Rust | 20 min |
| WebUI Tests | Tests React/TypeScript | 15 min |
| Security Scan | Trivy, Gitleaks, Semgrep | 15 min |
| Docker Build | Build de imágenes | 30 min |

### Checks Requeridos para Feature → Phase

| Check | Descripción | Timeout |
|-------|-------------|---------|
| Lint & Format | Clippy, ESLint, formateo | 10 min |
| Rust Tests | Tests unitarios Rust | 20 min |
| WebUI Tests | Tests React/TypeScript | 15 min |

## Excepciones

### Hotfixes

Para hotfixes críticos en producción:

1. Crear rama `hotfix/` desde `main`
2. Hacer el fix
3. Crear PR a `main`
4. Merge directo (requiere 1 approval)
5. Tag `v1.0.1`, `v1.0.2`, etc.

### Features Urgentes

Para features que necesitan merge rápido:

1. Crear rama `feature/` normalmente
2. Desarrollar y testear
3. Crear PR a `phase-*`
4. Después de merge, crear PR `phase-*` → `main`
5. Merge con 1 approval (por ser urgente)

## Monitoreo

### Dashboard de Branches

```
main (estable)
├── phase-1-chat (3/6 features completas) 🟡
│   ├── feature/1-core-hybrid-search ✅
│   ├── feature/1-core-mistralrs ✅
│   ├── feature/1-api-chat-endpoint 🟡
│   ├── feature/1-api-websocket ⏳
│   ├── feature/1-webui-chat-ui ⏳
│   └── feature/1-core-onnx-embeddings ⏳
└── phase-2-generate (no iniciada) ⚪
```

### Métricas

- **Feature duration:** < 4 días (30-40 horas)
- **Phase duration:** 2-4 semanas
- **Merge frequency:** 2-4 features por semana
- **CI/CD success rate:** > 95%

## Troubleshooting

### CI/CD Falla

1. Revisa logs en GitHub Actions
2. Corrige localmente
3. Push de nuevo
4. Si persiste, revisa dependencias

### Merge Conflict

1. Rebase desde la rama base
2. Resuelve conflictos
3. Push de nuevo
4. Si es complejo, pide ayuda

### Feature Tarda Mucho

1. Divide en sub-features más pequeñas
2. Crea nueva rama desde phase
3. Merge incremental
4. Actualiza milestone

---

**Tags:** #git #branch-protection #ci-cd #alesys