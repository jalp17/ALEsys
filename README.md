# 🚀🐱 ALEsys v2.0.0 - AI IDE Framework Completo

**ALEsys** es un framework RAG + agente configurable con capacidades avanzadas de IA para desarrollo de software, investigación científica y gestión de conocimiento.

---

## 📋 Tabla de Contenidos

- [Arquitectura](#arquitectura)
- [Features por Fase](#features-por-fase)
- [Setup Inicial](#setup-inicial)
- [API Reference](#api-reference)
- [Licencia](#licencia)

---

## 🏗️🐱 Arquitectura

```
┌─────────────────────────────────────────────────────────────┐
│                    ALEsys ECOSYSTEM                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────────┐         ┌─────────────────────┐    │
│  │   TAURI DESKTOP     │         │   WEBUI MULTI-USU   │    │
│  │   (1 usuario)       │         │   (Múltiples users) │    │
│  └──────────┬──────────┘         └──────────┬──────────┘    │
│             │                                │               │
│             │  MISMO CÓDIGO FRONTEND         │               │
│             │  (React + TypeScript)          │               │
│             └────────────┬───────────────────┘               │
│                          │                                    │
│                          ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐│
│  │           ALESYS CORE (Rust Backend)                     ││
│  │  - API REST + WebSocket                                  ││
│  │  - GraphRAG (PostgreSQL + pgvector)                      ││
│  │  - 27 Módulos Especializados                             ││
│  │  - 335 Tests Unitarios                                   ││
│  └──────────────────────────────────────────────────────────┘│
│                          │                                    │
│                          ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐│
│  │     PostgreSQL + pgvector + Grafos                       ││
│  └──────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### 🛠️🐱 Stack Tecnológico

| Componente | Tecnología |
|------------|------------|
| Backend | Rust (axum, sqlx, pgvector, mistralrs, ort) |
| Frontend | React + TypeScript + TailwindCSS |
| Base de Datos | PostgreSQL 16 + pgvector |
| Desktop | Tauri v2 |
| Multi-usuario | PHP 8.2 |

---

## 🎯🐱 Features por Fase

### Fases Tempranas (0-14)
- **Fase 0-5:** Core GraphRAG, LLM Engine, Sesiones, Generación de Código
- **Fase 6-10:** Executor Local, FS Ops, Automation, Agent Manager, Plugins
- **Fase 11-14:** Voice/Multimodal, Colaboración, Pair Programmer

### Fases Avanzadas (15-27) ✅ COMPLETAS

| Fase | Versión | Feature Principal | Estado |
|------|---------|-------------------|--------|
| 15 | v1.29.0 | AI Pair Programmer | ✅ |
| 16 | v1.30.0 | Context-Aware Learning | ✅ |
| 17 | v1.31.0 | Debug Assistant | ✅ |
| 18 | v1.32.0 | Test Generation | ✅ |
| 19 | v1.33.0 | Advanced Refactoring | ✅ |
| 20 | v1.34.0 | Knowledge Base Curation | ✅ |
| 21 | v1.35.0 | Multi-Agent Collaboration | ✅ |
| 22 | v1.36.0 | Advanced Analytics | ✅ |
| 23 | v1.37.0 | Workflow Automation | ✅ |
| 24 | v1.38.0 | Advanced Search & Filters | ✅ |
| 25 | v1.39.0 | Performance Optimization | ✅ |
| 26 | v1.40.0 | Security Hardening | ✅ |
| 27 | v1.41.0 | Production Deployment | ✅ |
| 28 | v2.0.0 | Final Integration | ✅ |

---

## 📦🐱 Módulos del Core (27 total)

### Core Base
- `llm` - Motor multi-backend (llama.cpp, mistralrs, candle, vLLM)
- `graphrag` - GraphRAG con pgvector + petgraph
- `session` - Gestión de sesiones multi-usuario
- `agent` - Agent manager y orchestrator
- `plugin` - Sistema de plugins extensible

### Desarrollo
- `generator` - Generación de código con LLM
- `executor` - Ejecución local con límites
- `fs_ops` - Operaciones de archivos
- `automation` - Automatización (LaTeX, Markdown, red)
- `pair_programmer` - AI Pair Programmer
- `debug_assistant` - Asistente de debugging
- `test_generation` - Generación automática de tests
- `advanced_refactoring` - Refactorización AST-based

### Conocimiento
- `voice` - Procesamiento de voz
- `multimodal` - Entradas multimodales
- `collaboration` - Colaboración en tiempo real
- `learning` - Aprendizaje contextual
- `knowledge_curation` - Curación de base de conocimiento
- `multi_agent` - Colaboración multi-agente
- `search_adv` - Búsqueda avanzada con filtros

### Operaciones
- `analytics` - Analíticas de uso y performance
- `workflow` - Automatización de workflows
- `performance` - Cache, pooling, batching, profiling
- `security` - Rate limiting, validación, auditoría
- `deployment` - Config, health checks, backups
- `e2e` - Tests end-to-end y stress testing

---

## 🛠️🐱 Setup Inicial

### Prerrequisitos

- Rust 1.80+
- Node.js 20+
- pnpm 9+
- Docker + Docker Compose
- PostgreSQL 16 (opcional, si no usas Docker)

### Instalación

```bash
# 1. Clone el repositorio
git clone https://github.com/tu-usuario/ALEsys
cd ALEsys

# 2. Ejecutar setup
./scripts/setup-dev.sh

# 3. Configurar variables de entorno
cp docker/.env.example .env
# Edita .env con tus configuraciones

# 4. Iniciar servicios
docker compose -f docker/docker-compose.yml up -d

# 5. Iniciar desarrollo
pnpm dev
```

### Verificar instalación

```bash
# Backend Rust
cargo run --bin alesys-cli -- --help

# Frontend Web
open http://localhost:5173

# Tests
cargo test --workspace --no-default-features --features test
```

---

## 🔌🐱 API Reference

### Endpoints Principales

#### Chat & GraphRAG
- `POST /api/v1/chat` - Chat con GraphRAG + sesiones
- `GET /api/v1/sessions` - Listar sesiones activas
- `POST /api/v1/sessions` - Crear sesión
- `GET /ws/chat` - WebSocket para streaming

#### Desarrollo
- `POST /api/v1/generate` - Generar archivos
- `POST /pair-programmer/analyze` - Analizar código
- `POST /pair-programmer/refactor` - Refactorizar
- `POST /debug/analyze` - Analizar errores
- `POST /test-generate` - Generar tests
- `POST /refactoring/analyze` - Analizar refactorización
- `POST /refactoring/preview` - Preview de cambios

#### Conocimiento
- `POST /kb/merge` - Fusionar documentos
- `POST /kb/split` - Dividir documentos
- `POST /kb/archive` - Archivar documentos
- `POST /kb/duplicates` - Detectar duplicados
- `POST /kb/quality` - Evaluar calidad
- `POST /search/advanced` - Búsqueda avanzada
- `POST /search/faceted` - Búsqueda con facetas
- `POST /search/suggest` - Autocompletado

#### Multi-Agente
- `GET /agents` - Listar agentes
- `GET /agents/stats` - Estadísticas de agentes
- `POST /agents/:id/execute` - Ejecutar agente
- `GET /collab/status` - Estado de colaboración
- `GET /collab/tasks` - Tasks de colaboración
- `POST /collab/consensus` - Consenso entre agentes

#### Operaciones
- `GET /analytics/usage` - Métricas de uso
- `GET /analytics/performance` - Performance
- `GET /analytics/users` - Analíticas por usuario
- `GET /analytics/reports` - Reportes
- `GET /workflows` - Listar workflows
- `POST /workflows` - Crear workflow
- `POST /workflows/:id/run` - Ejecutar workflow

#### Sistema
- `GET /health` - Health check
- `GET /metrics` - Métricas del sistema
- `GET /plugins` - Listar plugins
- `POST /plugins/:id/enable` - Habilitar plugin
- `POST /plugins/:id/disable` - Deshabilitar plugin

---

## 🧪🐱 Testing

```bash
# Todos los tests
cargo test --workspace --no-default-features --features test

# Tests con output detallado
cargo test --workspace --no-default-features --features test -- --test-threads=1

# Tests de un módulo específico
cargo test -p alesys-core --lib -- module_name::tests
```

**Total: 335 tests unitarios passing**

---

## 📊🐱 Métricas del Proyecto

| Métrica | Valor |
|---------|-------|
| **Versión** | v2.0.0 |
| **Módulos Core** | 27 |
| **Endpoints API** | ~50 |
| **Tests Unitarios** | 335 |
| **Líneas de Código** | ~15,000+ |
| **Fases Completadas** | 28 |

---

## 📄🐱 Licencia

GNU AGPL v3.0

---

## 📝🐱 CHANGELOG v2.0.0

### Agregado
- ✅ 13 fases completadas (15-27)
- ✅ 27 módulos del core
- ✅ 335 tests unitarios
- ✅ Documentación completa de API
- ✅ Tests end-to-end y stress testing

### Mejorado
- Performance con cache LRU + TTL
- Security hardening (rate limiting, validación)
- Health checks y backup manager
- Búsqueda avanzada con facetas

### Cambiado
- Versión establecida a v2.0.0
- Todos los módulos integrados y testeados

---

**Hecho con ❤️🐱 por el equipo ALEsys**