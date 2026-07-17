# 📅 Roadmap de Desarrollo ALEsys

## Visión General

ALEsys evoluciona de un sistema de ingesta de documentos a un **AI IDE completo** con capacidad de chat, generación de código, y ejecución sandboxeada.

---

## 🎯 Fases de Desarrollo

### **FASE 0: Setup del Monorepo** (Semana 1) ✅

**Objetivo:** Estructura base funcional

**Completado:**
- [x] Workspace Rust con 3 crates (core, api, cli)
- [x] Frontend React + TypeScript + Vite
- [x] Docker Compose para desarrollo
- [x] Scripts de setup
- [x] Documentación base

**Criterio de éxito:** `pnpm dev` levanta backend + frontend

---

### **FASE 1: Chat Básico con GraphRAG** (Semanas 2-4) 🟡

**Objetivo:** Chat funcional con contexto de documentos indexados

**Tareas:**

#### Backend Rust
- [ ] Implementar `GraphRAG::hybrid_search()` con pgvector
- [ ] Integrar mistralrs para inferencia LLM
- [ ] Endpoint `POST /api/chat` funcional
- [ ] WebSocket para streaming de respuestas
- [ ] Cache de embeddings con sled

#### Frontend
- [ ] Componente `Chat` completo
- [ ] Streaming de respuestas (WebSocket)
- [ ] Visualización de fuentes (GraphRAG context)
- [ ] Historial de chat local

#### Testing
- [ ] Tests de integración para GraphRAG
- [ ] E2E tests para chat flow

**Criterio de éxito:**
- Usuario hace pregunta → recibe respuesta con fuentes
- Streaming funciona (< 100ms por chunk)
- 100 queries de prueba exitosas

---

### **FASE 2: Generacion de Archivos** (Semanas 5-7) 🟡

**Objetivo:** Servicio backend de generacion de codigo desde prompts naturales

**NOTA:** El frontend actual (`Generate.tsx`) es un MVP funcional para probar el servicio. En Fase 7 sera reemplazado por Monaco editor + tree view + terminal. Ver `PHASE2_PLAN.md` para detalles.

**Tareas:**

#### Backend Rust (Servicio)
- [x] `CodeGenerator` con LLM compartido via AppState
- [x] Prompt templates por lenguaje (Python, JS, Rust, generic)
- [x] `SyntaxValidator` integrado post-generacion
- [x] Endpoint `POST /api/generate` funcional
- [ ] Context injection (archivos existentes en el request)
- [ ] Tests de integracion (5+ tests)

#### Frontend (MVP — reemplazado en Fase 7)
- [x] Pagina `/generate` con form + preview + download
- [ ] Historial de generaciones en localStorage

#### NO es responsabilidad de Fase 2
- ~~Editor inline (Monaco)~~ → Fase 7
- ~~Ejecucion de codigo (sandbox)~~ → Fase 7
- ~~Modificacion de archivos generados~~ → Fase 7
- ~~Tree view de archivos~~ → Fase 7
- ~~Terminal embebida~~ → Fase 7

**Criterio de exito:**
- Genera codigo Python/JS/Rust valido desde prompt
- LLM reutilizado (no crea instancias nuevas por request)
- Validacion de sintaxis integrada
- 28+ tests unitarios pasando

---

### **FASE 3: Gestión de Sesiones Multi-Usuario** (Semanas 8-9) 🟡

**Objetivo:** Múltiples usuarios con sesiones aisladas

**Tareas:**

#### Backend Rust
- [ ] `SessionManager` completo
- [ ] Aislamiento de contexto por sesión
- [ ] Historial de chat en DB
- [ ] endpoints `/api/sessions`

#### PHP Server
- [ ] Autenticación real (usuarios en DB)
- [ ] Session manager PHP
- [ ] Middlewares de auth
- [ ] Aislamiento de datos por usuario

#### Frontend
- [ ] Página `/sessions` para gestionar sesiones
- [ ] Login/Logout UI
- [ ] Selector de sesión activa
- [ ] Indicador de usuario conectado

**Criterio de éxito:**
- 2 usuarios pueden chatear simultáneamente
- Cada usuario ve solo su historial
- Sesiones persisten entre recargas

---

### **FASE 4: Optimización y Performance** (Semanas 10-12) 🟡

**Objetivo:** Production-ready performance

**Tareas:**

#### Backend Rust
- [ ] Profiling con `perf` + `flamegraph`
- [ ] Batch inserts para pipeline
- [ ] Connection pooling optimizado
- [ ] Parallel processing con `rayon`
- [ ] Cache de consultas frecuentes

#### Frontend
- [ ] Code splitting
- [ ] Lazy loading de componentes
- [ ] Optimización de re-renders
- [ ] Service worker para cache

#### Infrastructure
- [ ] Load testing con k6
- [ ] Benchmark suite
- [ ] Métricas con Prometheus

**Criterio de éxito:**
- Chat response < 500ms (p95)
- Soporta 50 usuarios concurrentes
- Memory usage < 500MB

---

### **FASE 5: Visualizador de Grafos** (Semanas 13-15) 🟡

**Objetivo:** Ver y navegar el grafo de conocimiento

**Tareas:**

#### Backend Rust
- [ ] Endpoint `GET /api/graph` (nodos + aristas)
- [ ] Algoritmos de centralidad (petgraph)
- [ ] Detección de comunidades
- [ ] Camino más corto entre nodos

#### Frontend
- [ ] Componente `GraphViewer` con Cytoscape.js
- [ ] Zoom/pan navigation
- [ ] Click en nodo → ver documento
- [ ] Búsqueda en grafo
- [ ] Filtrado por tipo de enlace

**Criterio de éxito:**
- Grafo de 1000 nodos renderiza en < 2s
- Usuario puede navegar y hacer zoom
- Click en nodo muestra metadata

---

### **FASE 6: Búsqueda Híbrida Avanzada** (Semanas 16-18) 🟡

**Objetivo:** Búsquedas complejas combinando vector + grafo + SQL

**Tareas:**

#### Backend Rust
- [ ] Query builder avanzado
- [ ] Filtros por fecha, tipo, área
- [ ] Reciprocal Rank Fusion (RRF)
- [ ] Query expansion con sinónimos
- [ ] Highlighting de términos

#### Frontend
- [ ] UI de búsqueda avanzada
- [ ] Filtros laterales
- [ ] Highlighting en resultados
- [ ] Guardar búsquedas frecuentes

**Criterio de éxito:**
- Búsqueda con 5 filtros en < 200ms
- Resultados relevantes (evaluar con dataset test)

---

### **FASE 7: Edicion y Ejecucion de Codigo** (SEMESTRE 2)

**Objetivo:** IDE completo con edicion y ejecucion sandboxeada

**NOTA:** Reutiliza el servicio `CodeGenerator` de Fase 2 tal cual. No se borra nada, se construye encima. El frontend `Generate.tsx` es reemplazado por Monaco editor.

**Tareas:**

#### Backend Rust - Sandbox
- [ ] `CodeSandbox::execute()` con Docker
- [ ] Limites de recursos (CPU, memoria, tiempo)
- [ ] Soporte para Python, JavaScript, Rust
- [ ] Streaming de stdout/stderr
- [ ] Filesystem aislado (solo /tmp)
- [ ] Sin acceso a red

#### Backend Rust - Edicion
- [ ] Endpoint `POST /api/modify`
- [ ] Diff generation
- [ ] Backup automatico
- [ ] Git integration (opcional)

#### Frontend (reemplaza Generate.tsx)
- [ ] Editor Monaco (VS Code engine)
- [ ] Terminal embebida (xterm.js)
- [ ] Tree view de archivos
- [ ] Diff viewer
- [ ] Boton "Ejecutar codigo"

#### Seguridad
- [ ] Auditoria de seguridad del sandbox
- [ ] Rate limiting por usuario
- [ ] Logs de auditoria
- [ ] Approval workflow para cambios

**Reutiliza de Fase 2:**
- `CodeGenerator` / `PromptTemplate` / `SyntaxValidator` — sin cambios
- `POST /api/generate` — se mantiene como endpoint
- `GenerateRequest`, `GenerationResult`, `BuildContext` — tipos compartidos

**Criterio de exito:**
- Ejecuta codigo Python de forma segura
- Terminal muestra output en tiempo real
- Editor permite modificar y guardar
- 0 escapes del sandbox en tests de seguridad

---

### **FASE 8: Tauri Desktop App** (Semana 19-20) 🟡

**Objetivo:** Aplicación desktop nativa

**Tareas:**

#### Tauri Setup
- [ ] Configurar `tauri.conf.json`
- [ ] Commands para operaciones nativas
- [ ] Acceso a filesystem local
- [ ] Notificaciones desktop
- [ ] System tray icon

#### Frontend
- [ ] `DesktopLayout` completo
- [ ] Detectar modo desktop vs web
- [ ] Atajos de teclado nativos
- [ ] Drag & drop de archivos

#### Build & Deploy
- [ ] CI/CD para builds multi-plataforma
- [ ] .deb, .rpm, .AppImage
- [ ] Auto-updater

**Criterio de éxito:**
- App desktop funcional en Linux
- Mismo código frontend que web
- Acceso nativo a archivos locales

---

## 📊 Métricas de Éxito por Fase

| Fase | Métrica | Objetivo |
|------|---------|----------|
| 1 | Chat response time | < 500ms (p95) |
| 2 | Código generado válido | > 80% |
| 3 | Usuarios concurrentes | 50+ |
| 4 | Memory usage | < 500MB |
| 5 | Grafo 1000 nodos | < 2s render |
| 6 | Búsqueda con filtros | < 200ms |
| 7 | Sandbox security | 0 escapes |
| 8 | Desktop app size | < 100MB |

---

## 🚨 Riesgos y Mitigación

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| mistralrs inmaduro | Alto | Fallback a llama-cpp-2 |
| pgvector performance | Medio | Índices HNSW, caching |
| Sandbox escape | Crítico | Docker + firecracker, auditoría |
| Frontend bloat | Bajo | Code splitting, profiling |

---

## 🎯 Próximo Hito Inmediato

**FASE 1 - Semana 2:**
- [ ] `GraphRAG::hybrid_search()` implementado
- [ ] `POST /api/chat` retorna respuestas reales
- [ ] Chat UI muestra respuestas (sin streaming aún)
- [ ] 10 queries de prueba exitosas

---

**Última actualización:** 2026-07-13
**Próxima revisión:** 2026-07-20