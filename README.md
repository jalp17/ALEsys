#  ALEsys - GraphRAG-PG

**GraphRAG-PG: PostgreSQL Graph & Vector Ingestion Engine**

Sistema de ingesta híbrida (vectorial + grafos de conocimiento) sobre PostgreSQL para indexar documentos Markdown y permitir búsquedas científicas complejas con LLM.

---

## 📋 Tabla de Contenidos

- [Arquitectura](#arquitectura)
- [Setup Inicial](#setup-inicial)
- [Uso](#uso)
- [Licencia](#licencia)

---

## 🏗️ Arquitectura

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
│  │  - LLM Engine (mistralrs + ort)                          ││
│  └──────────────────────────────────────────────────────────┘│
│                          │                                    │
│                          ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐│
│  │     PostgreSQL + pgvector + Grafos                       ││
│  └──────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Stack Tecnológico

| Componente | Tecnología |
|------------|------------|
| Backend | Rust (axum, sqlx, pgvector, mistralrs, ort) |
| Frontend | React + TypeScript + TailwindCSS |
| Base de Datos | PostgreSQL 16 + pgvector |
| Desktop | Tauri v2 |
| Multi-usuario | PHP 8.2 |

---

## 🛠️ Setup Inicial

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

# API (después de iniciar el servidor)
curl http://localhost:3000/health
```

---

## 💻 Uso

### Estructura del Proyecto

```
ALEsys/
├── crates/              # Backend Rust
│   ├── core/           # Lógica de negocio
│   ├── api/            # API REST + WebSocket
│   └── cli/            # CLI standalone
├── webui/              # Frontend compartido
├── server/             # PHP backend (WebUI multi-usuario)
├── desktop/            # Tauri wrapper
└── docker/             # Docker configs
```

### Comandos Útiles

```bash
# Desarrollo (backend + frontend)
pnpm dev

# Solo backend Rust
cargo run --bin alesys-api

# Solo frontend
cd webui && npm run dev

# Build completo
pnpm build

# Tests
pnpm test

# Lint
pnpm lint
```

### Producción con Docker

```bash
# Build de imágenes
docker compose -f docker/docker-compose.yml build

# Levantar servicios
docker compose -f docker/docker-compose.yml up -d

# Ver logs
docker compose logs -f
```

---

## 📚 Documentación

- [API Reference](docs/api.md)
- [Tutorial](docs/tutorial.md)
- [Instalación](docs/installation.md)
- [Uso](docs/usage.md)

---

## 📄 Licencia

GNU Affero General Public License v3.0 - ver [LICENSE](LICENSE) para detalles.

---

**Tags:** #alesys #graphrag #rust #react #postgresql #pgvector #llm #rag
