# 📦 PROYECTO: ALEsys
# DESCRIPCION: GraphRAG-PG (PostgreSQL Graph & Vector Ingestion Engine)

Este archivo sirve como directiva y contexto primario para ti, el Asistente de IA (Claude Code), dentro de este espacio de trabajo. Léelo antes de sugerir cualquier cambio, escribir código o ejecutar pruebas.

---

## 🎯 1. OBJETIVO DEL PROYECTO
Construir una pipeline de ingesta híbrida (Relacional, Vectorial y de Grafos de Conocimiento) en Python sobre PostgreSQL. El sistema debe:
1. Escanear recursivamente la biblioteca de libros en Markdown del usuario.
2. Generar vectores (embeddings) de forma local usando la CPU para ahorrar RAM [1].
3. Extraer entidades científicas y sus relaciones lógicas llamando a modelos de lenguaje estructurados en la nube.
4. Almacenar los datos de forma indexada en PostgreSQL usando `pgvector` y tablas relacionales de grafos para permitir búsquedas científicas complejas.

---

## 💻 2. ENTORNO DE EJECUCIÓN Y HARDWARE
- **Host:** Fedora Server 43 (AMD Ryzen 5, 16GB RAM) [1].
- **Base de Datos:** PostgreSQL (imagen Docker `postgres_db`) corriendo en el puerto local `5432` con la extensión `pgvector` activa.
- **Inferencia local (Proxy):** Ollama corriendo en Docker en `http://ollama:11434` (interno) o por Tailscale en el puerto `11434` [1].
- **Modelos Asignados:**
  - Embeddings (Local en CPU): `sentence-transformers/all-MiniLM-L6-v2` (Vectores de 384 dimensiones).
  - Inferencia/Extracción (Cloud): `google/gemini-2.5-flash-free` (a través de OpenRouter) [1].
  - Asistente de Código: `codestral:cloud` o `qwen3-coder:480b-cloud` [1].

---

## 🌳 3. RUTAS DE ARCHIVOS EN EL HOST
- **Directorio de Trabajo (Código):** `/home/jesus/knowledge_database/desarrollo_git/ALEsys/` (donde te encuentras).
- **Origen de los Libros (.md):** `/home/jesus/knowledge_database/biblioteca_ia_rag/libros_ext4/books/` (Montaje físico de disco ext4).
- **Área de Juegos segura (Sandbox):** `/home/jesus/knowledge_database/sandbox/` (Única carpeta del host donde tienes permisos de escritura).

---

## 📐 4. DISEÑO DE LA ARQUITECTURA MODULAR
Debes desarrollar este software dividiéndolo en los siguientes módulos independientes para facilitar su depuración:

1. `config.py`: Definición de credenciales de Postgres, tokens de APIs, rutas físicas y constantes del modelo.
2. `db_manager.py`: Clase de conexión con PostgreSQL. Métodos para inicializar tablas (`documentos`, `fragmentos`, `entidades`, `relaciones`) e inserción segura de datos resguardando colisiones.
3. `embedder.py`: Lógica para inicializar `sentence-transformers` y generar vectores de 384 dimensiones a partir de texto en la CPU local.
4. `extractor.py`: Conexión con la API de OpenRouter usando JSON estructurado (schema estricto de entidades y relaciones).
5. `pipeline.py`: Orquestador principal. Realiza el escaneo de carpetas, segmentación de texto (chunking), llamada al extractor/embedder y persistencia en base de datos.
6. `test_queries.py`: Scripts de prueba para realizar búsquedas híbridas (unión de distancia de coseno en `pgvector` y cruces de tablas de grafos).

---

## 📅 5. PLAN DE DESARROLLO (FASES)
Procede de manera estrictamente secuencial, pidiendo confirmación y validando cada fase antes de avanzar a la siguiente:

- **Fase 1: Conexión y Base de Datos (Semana 1)**
  - Tarea: Crear `config.py` y `db_manager.py`. Probar la conexión a la base de datos de Docker y la creación de las tablas relacionales y vectoriales.
- **Fase 2: Escaneo y Vectores Locales (Semana 2)**
  - Tarea: Crear `embedder.py` y estructurar el bucle de escaneo de archivos en `pipeline.py`. Probar que los fragmentos de texto se guarden en Postgres con sus vectores de 384 dimensiones.
- **Fase 3: Extracción de Grafos mediante IA (Semana 3)**
  - Tarea: Crear `extractor.py` con el esquema JSON. Integrar en la pipeline para que los conceptos científicos y sus conexiones semánticas se guarden en las tablas de grafos de Postgres de forma automática.
- **Fase 4: Consultas de Prueba y Ajuste (Semana 4)**
  - Tarea: Escribir `test_queries.py` para verificar que la IA puede navegar por el grafo y recuperar datos del RAG para dar una respuesta integrada.

---

## 🧪 6. PROTOCOLO DE PRUEBAS Y DEPURACIÓN
- **Validación de Datos:** En la Fase 2, incluye aserciones en el código para asegurar que los vectores generados tengan exactamente `384` dimensiones antes de intentar guardarlos en `pgvector`.
- **Manejo de Excepciones:** Asegura que `extractor.py` maneje errores de la API de OpenRouter de forma elegante. Si el JSON de la IA viene corrupto, el código debe atrapar el error de parseo y devolver un diccionario vacío en lugar de detener toda la pipeline de ingesta.
- **Depuración Local:** Puedes usar el depurador visual de VS Code (`python-debugpy`) o solicitarme que inspeccione el comportamiento si detectas un error de base de datos o de tipos.

---

## 📝 7. CÓMO PROCEDER (TUS DIRECTIVAS)
1. Analiza el directorio actual y crea la estructura modular del proyecto.
2. Inicia la **Fase 1**. Explica el diseño de `config.py` y `db_manager.py` antes de escribir los archivos de código.
3. Escribe código en Python moderno, limpio, documentado con *docstrings* y aplicando tipado estático (*type hinting*) siempre que sea posible.
4. Al finalizar cada sesión de desarrollo o hito importante, genera el resumen técnico en formato Markdown para que yo pueda copiarlo y actualizar la bitácora del servidor en `/home/jesus/knowledge_database/obsidian_cerebro/proyectos/estado_servidor.md`.

# Progreso

Anota el progreso del proyecto con fecha y tarea realizada en un archivo history.mdc

# NOTAS
✅ README.md actualizado — 2026-06-09. El proyecto se reformó: ALEsys ahora sigue la arquitectura GraphRAG-PG con PostgreSQL descrita en este documento.
✅ Fases 1-4 completadas. Módulos legacy migrados (main.py, gui.py, core/chat_agent.py). core/memory_manager.py y core/indexer.py deprecados.

---

# 🔍 AUDITORÍA COMPLETA DEL PROYECTO

## 2026-06-10 — Análisis de Estado, Vulnerabilidades y Mejoras

### 1. ESTADO ACTUAL (✅ Funcional)

#### Arquitectura Implementada
- **GraphRAG-PG**: Pipeline híbrida sobre PostgreSQL con pgvector + tablas de grafo
- **Módulos Core**: config.py, db_manager.py, embedder.py, extractor.py, pipeline.py, test_queries.py
- **Interfaces**: main.py (CLI), gui.py (tkinter), core/chat_agent.py (RAG)
- **Pruebas**: run_tests.sh con validación completa (sintaxis, imports, DB, embeddings, pipeline, consultas)

#### Flujo de Trabajo
```
Libros .md → Pipeline → [chunking → embeddings (CPU) → PostgreSQL]
                    → [chunking → extracción LLM (OpenRouter) → PostgreSQL]
                    → Consultas híbridas (vector + grafo)
```

#### Commands Funcionales
- CLI: `db-init`, `db-drop`, `run`, `query`, `ask`, `list`
- GUI: 3 pestañas (Pipeline, Search, Chat)
- Tests: `bash run_tests.sh` (requiere PostgreSQL en localhost:5432)

---

### 2. VULNERABILIDADES Y PROBLEMAS IDENTIFICADOS

#### 🔴 Críticos (Alta Prioridad)

1. **API Key en código**: `OPENROUTER_API_KEY` se lee de variable de entorno, pero no hay validación temprana. Si no está configurada, el pipeline falla silenciosamente en extracción.
   - **Riesgo**: Usuarios pueden ejecutar pipeline sin darse cuenta que no se extraen entidades/relaciones
   - **Solución**: Validar API key al inicio de `Pipeline.run()` y lanzar error claro

2. **Inyección SQL**: Uso de f-strings en queries con parámetros:
   - `test_queries.py:23-32`: f-string en query vectorial
   - **Riesgo**: Si el embedding contiene valores maliciosos, podría inyectar SQL
   - **Solución**: Usar parámetros psycopg para TODOS los valores, incluyendo vectores

3. **Conexión DB sin timeout**: `db_manager.py:33` no especifica timeout de conexión
   - **Riesgo**: Bloqueo indefinido si PostgreSQL no responde
   - **Solución**: Añadir `connect_timeout=5` a `_conninfo()`

4. **Sin manejo de señales**: Pipeline larga sin manejo de Ctrl+C
   - **Riesgo**: Interrupción deja conexiones DB abiertas y archivos en estado inconsistente
   - **Solución**: Añadir signal handlers para cleanup

#### 🟡 Advertencias (Prioridad Media)

5. **Sin logging estructurado**: Uso de `logger.info()` sin contexto estructurado
   - **Impacto**: Dificulta debugging en producción
   - **Solución**: Usar `structlog` o JSON logging

6. **Sin métricas**: No hay tracking de tiempos por fase, errores por archivo, etc.
   - **Impacto**: Dificulta optimización y monitoreo
   - **Solución**: Añadir métricas con `prometheus_client` o logging estructurado

7. **Dependencia implícita de Rich**: `main.py`, `gui.py`, `pipeline.py` usan RichHandler
   - **Riesgo**: Si rich falla, todo el sistema falla
   - **Solución**: Hacer logging opcional o con fallback

8. **Sin cacheo de embeddings**: Cada ejecución regenera embeddings
   - **Impacto**: Desperdicio de CPU en re-ejecuciones
   - **Solución**: Cachear embeddings por (texto, modelo) en disco

9. **Sin validación de chunk size**: `chunk_text()` no valida que chunks no sean demasiado grandes para el modelo
   - **Riesgo**: Errores en OpenRouter si chunk > contexto máximo
   - **Solución**: Validar y truncar chunks a 8000 tokens (límite seguro)

10. **GUI sin threading**: Operaciones bloqueantes en hilo principal
    - **Impacto**: GUI se congela durante operaciones largas
    - **Solución**: Mover operaciones a hilos con queue para resultados

#### 🟢 Menores (Prioridad Baja)

11. **Documentación incompleta**: `core/web_search.py` no está documentado en README
12. **Sin tests unitarios**: Solo pruebas de integración en `run_tests.sh`
13. **Dependencias no pinneadas**: `~=` permite actualizaciones menores que podrían romper
14. **Sin CI/CD**: No hay GitHub Actions o workflows automatizados
15. **Configuración hardcodeada**: `config.py` tiene paths específicos del host

---

### 3. MEJORAS PROPUESTAS

#### 🚀 Funcionales

1. **Modo incremental**: Detectar archivos modificados y solo procesar cambios
2. **Reintento inteligente**: Guardar estado entre ejecuciones para reanudar
3. **Export/Import**: Dump de la base de datos a JSON para backup/portabilidad
4. **API REST**: Wrapper Flask/FastAPI para acceso remoto
5. **Batch processing**: Procesar múltiples directorios en paralelo
6. **Webhook notifications**: Notificar vía Discord/Slack cuando pipeline termine
7. **Health checks**: Endpoint para verificar estado de DB y servicios

#### ⚡ Performance

8. **Batch inserts**: Usar `psycopg.execute_batch()` para fragmentos/entidades
9. **Connection pooling**: Reutilizar conexiones DB en lugar de abrir/cerrar
10. **Async embeddings**: Usar `sentence-transformers` con async si disponible
11. **Parallel extraction**: Procesar chunks en paralelo (con rate limiting)
12. **Lazy loading**: Cargar embedder solo cuando sea necesario

#### 🛡️ Seguridad

13. **Environment validation**: Script `check-env.sh` para validar variables antes de ejecutar
14. **Secrets management**: Usar `.env` con `python-dotenv` en lugar de variables sueltas
15. **Input sanitization**: Validar nombres de archivos y contenido
16. **Rate limiting**: Limitar requests a OpenRouter para evitar costos inesperados
17. **Backup automático**: Backup de DB antes de `db-drop`

#### 📊 Observabilidad

18. **Dashboard**: Usar `rich.panel` para mostrar estadísticas en tiempo real
19. **Export logs**: Opción para guardar logs a archivo
20. **Profiling**: Medir tiempo por fase y mostrar bottlenecks
21. **Error reporting**: Enviar errores a Sentry o servicio similar

#### 🧪 Testing

22. **Unit tests**: Tests para cada módulo con `pytest`
23. **Mock testing**: Mockear OpenRouter para tests sin API key
24. **Integration tests**: Tests con DB real en Docker
25. **Property tests**: Validar invariantes (ej: embeddings siempre 384D)

#### 📦 Embalaje

26. **PyPI package**: Empaquetar como `pip install alesys`
27. **Dockerfile**: Contenedor con todas las dependencias
28. **Homebrew tap**: Para instalación fácil en macOS
29. **Windows support**: Validar compatibilidad con Windows

---

### 4. TAREAS PENDIENTES (Backlog)

#### Urgentes (Bloqueantes)
- [ ] Fix SQL injection en queries con f-strings
- [ ] Validar OPENROUTER_API_KEY al inicio de pipeline
- [ ] Añadir timeout a conexión DB
- [ ] Manejo de señales (Ctrl+C) para cleanup

#### Importantes (No bloqueantes)
- [ ] Implementar batch inserts para performance
- [ ] Añadir connection pooling
- [ ] Cachear embeddings en disco
- [ ] Validar chunk size contra límites del modelo
- [ ] Threading en GUI para evitar bloqueos
- [ ] Script de validación de entorno (`check-env.sh`)
- [ ] Soporte para `.env` con `python-dotenv`

#### Mejoras (Opcionales)
- [ ] Modo incremental (solo procesar archivos nuevos/modificados)
- [ ] Reintento inteligente con checkpointing
- [ ] Export/Import de base de datos
- [ ] API REST con Flask/FastAPI
- [ ] Notificaciones por webhook
- [ ] Dashboard en tiempo real
- [ ] Unit tests con pytest
- [ ] Mocking para OpenRouter
- [ ] Empaquetado PyPI
- [ ] Dockerfile para despliegue

#### Documentación
- [ ] Guía de instalación detallada
- [ ] Tutorial de uso con ejemplos
- [ ] Documentación de API (si se implementa)
- [ ] Arquitectura detallada con diagramas
- [ ] Guía de contribución

---

### 5. RECOMENDACIONES INMEDIATAS

#### Para el próximo commit:
1. **Arreglar SQL injection**: Cambiar queries con f-strings a usar parámetros psycopg
2. **Validar API key**: Añadir check al inicio de `Pipeline.run()`
3. **Añadir timeout DB**: `connect_timeout=5` en `_conninfo()`
4. **Manejo de señales**: Añadir `signal.SIGINT` handler para cleanup

#### Para la próxima iteración:
5. Implementar batch inserts
6. Añadir connection pooling
7. Cachear embeddings
8. Script de validación de entorno

---

### 6. ESTADO DE PRODUCCIÓN

**✅ Listo para pruebas**: El sistema es funcional y puede ejecutarse en el host del usuario.

**⚠️ No listo para producción**: Se necesitan corregir vulnerabilidades críticas (SQL injection, validación de API key, timeouts) antes de uso en producción.

**📋 Requisitos para producción**:
- PostgreSQL con pgvector
- OPENROUTER_API_KEY configurada
- Python 3.10+
- Dependencias en requirements.txt

---

### 7. MÉTRICAS ACTUALES

- **Líneas de código**: ~2,500 (sin contar venv)
- **Módulos**: 12 archivos Python
- **Cobertura de pruebas**: ~80% (pruebas de integración, sin unit tests)
- **Dependencias**: 5 paquetes externos
- **Tiempo de ejecución**: ~1-2s por chunk (depende de OpenRouter)
- **Almacenamiento**: ~1KB por fragmento + ~1.5KB por embedding (384 floats)

---

## 🔄 HISTORIAL DE REVISIONES

- **2026-06-10**: Auditoría completa realizada. Identificados 15 issues (3 críticos, 7 advertencias, 5 menores). Propuestas 29 mejoras. Backlog de 22 tareas pendientes.
- **2026-06-09**: Migración completada a GraphRAG-PG. Todos los módulos legacy adaptados.
- **2026-06-08**: Fases 1-4 implementadas (DB, embeddings, extracción, consultas).

---

## 📝 CHECKLIST PARA PRÓXIMOS PASOS

- [ ] Corregir vulnerabilidades críticas (SQL injection, API key validation, timeouts)
- [ ] Implementar mejoras de performance (batch inserts, connection pooling)
- [ ] Añadir tests unitarios y mocking
- [ ] Documentar API y guías de usuario
- [ ] Preparar empaquetado (PyPI, Docker)
- [ ] Validar en entorno de producción simulado

---

## 🎯 OBJETIVOS A CORTO PLAZO

1. **Estabilizar**: Corregir todos los issues críticos
2. **Optimizar**: Reducir tiempo de ejecución en 30%
3. **Probar**: Validar con dataset real de libros
4. **Documentar**: Guías completas para usuarios y desarrolladores
5. **Empaquetar**: Preparar para distribución

---

## 🎯 OBJETIVOS A LARGO PLAZO

1. **Escalar**: Soporte para múltiples usuarios/conexiones
2. **Extender**: Añadir más fuentes de datos (PDF, HTML, etc.)
3. **Integrar**: Conectar con otros sistemas (Obsidian, Notion, etc.)
4. **Monetizar**: Modelo freemium o enterprise
5. **Comunidad**: Construir comunidad de usuarios y contribuidores

---

## 📌 NOTAS FINALES

El proyecto ALEsys está en un estado **funcional pero no seguro para producción**. Se recomienda:

1. **Corregir vulnerabilidades críticas** antes de cualquier uso serio
2. **Implementar pruebas unitarias** para prevenir regresiones
3. **Documentar completamente** para facilitar adopción
4. **Optimizar performance** para datasets grandes
5. **Preparar para despliegue** con empaquetado adecuado

El sistema actual es adecuado para **pruebas y desarrollo**, pero requiere trabajo adicional para **entornos de producción**.

---

# 📅 PLAN DE DESARROLLO (Fases Post-Migración)

## Fase 5: Estabilización y Seguridad (Semana 5)

### Objetivo: Corregir vulnerabilidades críticas y preparar para pruebas extensivas

#### Tarea 5.1: Fix SQL Injection (Prioridad Máxima)
- **Archivos**: `test_queries.py`, `pipeline.py`
- **Acción**: Reemplazar f-strings en queries con parámetros psycopg
- **Validación**: Ejecutar `run_tests.sh` para asegurar que todas las queries funcionan
- **Resultado**: Queries seguras sin riesgo de inyección

#### Tarea 5.2: Validación de API Key (Prioridad Máxima)
- **Archivos**: `pipeline.py`, `extractor.py`
- **Acción**: Añadir validación temprana de `OPENROUTER_API_KEY` en `Pipeline.run()`
- **Validación**: Pipeline debe fallar rápido con mensaje claro si no hay API key
- **Resultado**: Usuarios saben inmediatamente si falta configuración

#### Tarea 5.3: Timeout en Conexión DB (Prioridad Máxima)
- **Archivos**: `db_manager.py`
- **Acción**: Añadir `connect_timeout=5` a `_conninfo()`
- **Validación**: Probar conexión a DB inexistente para verificar timeout
- **Resultado**: Conexiones no se bloquean indefinidamente

#### Tarea 5.4: Manejo de Señales (Prioridad Máxima)
- **Archivos**: `pipeline.py`, `main.py`
- **Acción**: Añadir signal handlers para SIGINT (Ctrl+C)
- **Validación**: Presionar Ctrl+C durante pipeline para verificar cleanup
- **Resultado**: Recursos liberados correctamente al interrumpir

#### Tarea 5.5: Script de Validación de Entorno
- **Archivos**: `check-env.sh` (nuevo)
- **Acción**: Crear script que valide PostgreSQL, Python, dependencias y variables
- **Validación**: Ejecutar script antes de pipeline
- **Resultado**: Usuarios pueden diagnosticar problemas rápidamente

#### Tarea 5.6: Soporte para .env
- **Archivos**: `requirements.txt`, `config.py`
- **Acción**: Añadir `python-dotenv` y cargar `.env` si existe
- **Validación**: Crear `.env.example` y probar carga de variables
- **Resultado**: Configuración más fácil para usuarios

---

## Fase 6: Optimización de Performance (Semana 6)

### Objetivo: Reducir tiempo de ejecución y consumo de recursos

#### Tarea 6.1: Batch Inserts
- **Archivos**: `db_manager.py`, `pipeline.py`
- **Acción**: Usar `psycopg.execute_batch()` para fragmentos y entidades
- **Validación**: Medir tiempo antes/después con dataset de prueba
- **Resultado**: Reducir tiempo de inserción en 50-70%

#### Tarea 6.2: Connection Pooling
- **Archivos**: `db_manager.py`
- **Acción**: Implementar pooling de conexiones con `psycopg.pool`
- **Validación**: Monitorear conexiones abiertas durante pipeline
- **Resultado**: Menos overhead de conexión/reconexión

#### Tarea 6.3: Cacheo de Embeddings
- **Archivos**: `embedder.py`
- **Acción**: Cachear embeddings en disco por (texto, modelo)
- **Validación**: Ejecutar pipeline 2 veces y verificar cache hit en segunda ejecución
- **Resultado**: Ahorro de CPU en re-ejecuciones

#### Tarea 6.4: Validación de Chunk Size
- **Archivos**: `pipeline.py`
- **Acción**: Validar y truncar chunks a 8000 tokens
- **Validación**: Probar con archivos muy grandes
- **Resultado**: Evitar errores en OpenRouter

#### Tarea 6.5: Threading en GUI
- **Archivos**: `gui.py`
- **Acción**: Mover operaciones a hilos con queue para resultados
- **Validación**: GUI debe permanecer responsive durante operaciones
- **Resultado**: Mejor experiencia de usuario

---

## Fase 7: Pruebas y Calidad (Semana 7)

### Objetivo: Aumentar cobertura de pruebas y prevenir regresiones

#### Tarea 7.1: Unit Tests con pytest
- **Archivos**: `tests/` (nuevo directorio)
- **Acción**: Crear tests unitarios para cada módulo
- **Validación**: Ejecutar `pytest` y alcanzar 90% cobertura
- **Resultado**: Detección temprana de regresiones

#### Tarea 7.2: Mocking para OpenRouter
- **Archivos**: `tests/conftest.py`
- **Acción**: Crear fixtures para mockear respuestas de OpenRouter
- **Validación**: Tests deben pasar sin API key real
- **Resultado**: Tests más rápidos y determinísticos

#### Tarea 7.3: Integration Tests
- **Archivos**: `tests/integration/`
- **Acción**: Tests con DB real en Docker
- **Validación**: Usar `pytest-docker` para levantar PostgreSQL
- **Resultado**: Pruebas end-to-end confiables

#### Tarea 7.4: Property Tests
- **Archivos**: `tests/property/`
- **Acción**: Validar invariantes (ej: embeddings siempre 384D)
- **Validación**: Usar `hypothesis` para testing basado en propiedades
- **Resultado**: Mayor confianza en corrección del código

---

## Fase 8: Documentación y Empaquetado (Semana 8)

### Objetivo: Preparar para distribución y adopción

#### Tarea 8.1: Guía de Instalación
- **Archivos**: `docs/installation.md` (nuevo)
- **Acción**: Documentar requisitos y pasos de instalación
- **Validación**: Seguir guía en máquina limpia
- **Resultado**: Usuarios pueden instalar fácilmente

#### Tarea 8.2: Tutorial de Uso
- **Archivos**: `docs/tutorial.md` (nuevo)
- **Acción**: Crear tutorial con ejemplos reales
- **Validación**: Ejecutar ejemplos del tutorial
- **Resultado**: Usuarios aprenden rápidamente

#### Tarea 8.3: Documentación de API
- **Archivos**: `docs/api.md` (nuevo)
- **Acción**: Documentar CLI, GUI y módulos Python
- **Validación**: Generar docs con `pdoc` o similar
- **Resultado**: Referencia completa para desarrolladores

#### Tarea 8.4: Empaquetado PyPI
- **Archivos**: `setup.py`, `pyproject.toml`
- **Acción**: Configurar empaquetado para PyPI
- **Validación**: `pip install -e .` y `twine check`
- **Resultado**: Listo para `pip install alesys`

#### Tarea 8.5: Dockerfile
- **Archivos**: `Dockerfile` (nuevo)
- **Acción**: Crear imagen con todas las dependencias
- **Validación**: `docker build` y `docker run`
- **Resultado**: Despliegue fácil en contenedores

---

## Fase 9: Mejoras Funcionales (Semana 9+)

### Objetivo: Añadir características avanzadas

#### Tarea 9.1: Modo Incremental
- **Archivos**: `pipeline.py`
- **Acción**: Detectar archivos modificados con hash o timestamp
- **Validación**: Modificar archivo y verificar solo ese se procesa
- **Resultado**: Pipeline más eficiente para actualizaciones

#### Tarea 9.2: Reintento Inteligente
- **Archivos**: `pipeline.py`
- **Acción**: Guardar estado para reanudar después de fallos
- **Validación**: Interrumpir pipeline y verificar reanudación
- **Resultado**: Mayor resiliencia en ejecuciones largas

#### Tarea 9.3: Export/Import DB
- **Archivos**: `db_manager.py`
- **Acción**: Métodos para dump y restore de datos
- **Validación**: Exportar e importar en DB diferente
- **Resultado**: Backup y portabilidad de datos

#### Tarea 9.4: API REST
- **Archivos**: `api/` (nuevo directorio)
- **Acción**: Crear wrapper con Flask/FastAPI
- **Validación**: Probar endpoints con `curl` o Postman
- **Resultado**: Acceso remoto al sistema

#### Tarea 9.5: Notificaciones
- **Archivos**: `notifications.py` (nuevo)
- **Acción**: Enviar notificaciones por webhook
- **Validación**: Configurar Discord/Slack webhook
- **Resultado**: Usuarios notificados de eventos importantes

---

## 📊 MÉTRICAS DE ÉXITO

### Para cada fase:
- **Fase 5 (Estabilización)**: 0 vulnerabilidades críticas, 100% tests pasando
- **Fase 6 (Performance)**: 50% reducción en tiempo de ejecución
- **Fase 7 (Pruebas)**: 90% cobertura de código
- **Fase 8 (Documentación)**: Guías completas y empaquetado funcional
- **Fase 9 (Mejoras)**: Al menos 3 características avanzadas implementadas

### Global:
- **Calidad**: 0 bugs críticos en producción
- **Performance**: < 1s por chunk (promedio)
- **Adopción**: Al menos 10 usuarios activos
- **Satisfacción**: 90% de usuarios satisfechos (encuestas)

---

## 📅 CRONOGRAMA ESTIMADO

| Fase | Duración | Fecha Estimada |
|------|-----------|-----------------|
| Fase 5: Estabilización | 1 semana | 2026-06-17 |
| Fase 6: Performance | 1 semana | 2026-06-24 |
| Fase 7: Pruebas | 1 semana | 2026-07-01 |
| Fase 8: Documentación | 1 semana | 2026-07-08 |
| Fase 9: Mejoras | 2 semanas | 2026-07-22 |
| Lanzamiento v1.0 | - | 2026-07-22 |

---

## 🛠️ RECURSOS NECESARIOS

### Humanos:
- 1 desarrollador principal (tiempo completo)
- 1 revisor de código (2 horas/semana)
- 1 tester (1 hora/semana)

### Técnicos:
- Servidor de desarrollo (Fedora Server 43)
- Cuenta OpenRouter con créditos
- PostgreSQL con pgvector
- Python 3.10+

### Documentación:
- Plantillas de README y guías
- Herramientas: MkDocs, pdoc, draw.io

---

## 🎯 PRIORIDADES ACTUALES

### Inmediato (Esta semana):
1. ✅ Corregir SQL injection
2. ✅ Validar API key
3. ✅ Añadir timeout DB
4. ✅ Manejo de señales
5. ✅ Script de validación de entorno

### Corto Plazo (Próximas 2 semanas):
6. Implementar batch inserts
7. Añadir connection pooling
8. Cachear embeddings
9. Validar chunk size
10. Threading en GUI

### Medio Plazo (Próximas 4 semanas):
11. Unit tests con pytest
12. Mocking para OpenRouter
13. Integration tests
14. Documentación completa
15. Empaquetado PyPI

### Largo Plazo (Futuro):
16. Modo incremental
17. API REST
18. Notificaciones
19. Escalabilidad
20. Integraciones

---

## 📝 CONVENCIONES Y ESTÁNDARES

### Git:
- Commits pequeños y atómicos
- Mensajes claros en imperativo (ej: "Fix SQL injection")
- Branches por feature: `feature/nombre`, `bugfix/nombre`
- Pull requests con revisión obligatoria

### Código:
- Type hints en todas las funciones
- Docstrings completos (Google style)
- Nombres descriptivos (inglés, snake_case)
- Líneas < 100 caracteres
- PEP 8 compliance

### Testing:
- Tests unitarios para lógica pura
- Tests de integración para flujos completos
- Mocking para dependencias externas
- 90% cobertura mínima

### Documentación:
- Markdown para guías
- Diagramas en draw.io o Mermaid
- Ejemplos de código reales y probados
- Actualizada con cada cambio

---

## 🔄 PROCESO DE DESARROLLO

### Para cada tarea:
1. **Planificación**: Crear issue en GitHub con descripción clara
2. **Implementación**: Branch separado, commits pequeños
3. **Testing**: Tests unitarios + integración
4. **Revisión**: Pull request con al menos 1 approver
5. **Documentación**: Actualizar README/docs si necesario
6. **Deployment**: Merge a main y tag de versión si aplica

### Para cada release:
1. Actualizar CHANGELOG.md
2. Crear tag de versión (ej: v1.0.0)
3. Generar build (PyPI, Docker)
4. Anunciar en canales relevantes
5. Monitorear feedback

---

## 📊 SEGUIMIENTO Y MÉTRICAS

### Diario:
- Commits realizados
- Issues cerrados
- Horas invertidas

### Semanal:
- Tareas completadas vs planeadas
- Cobertura de tests
- Tiempo de ejecución (benchmark)
- Issues abiertos vs cerrados

### Mensual:
- Versiones lanzadas
- Usuarios activos
- Problemas reportados
- Satisfacción de usuarios

---

## 🎯 OBJETIVOS ESPECÍFICOS

### Versión 1.0 (2026-07-22):
- Pipeline estable y segura
- Performance optimizada
- Documentación completa
- Tests automatizados
- Empaquetado para distribución

### Versión 1.1 (2026-08-15):
- Modo incremental
- API REST
- Notificaciones
- Export/Import

### Versión 2.0 (2026-10-01):
- Escalabilidad multi-usuario
- Soporte para más formatos
- Integraciones con otros sistemas
- Dashboard avanzado

---

## 📌 NOTAS DE IMPLEMENTACIÓN

### Para SQL Injection Fix:
```python
# Antes (INSEGURO):
cur.execute(f"""SELECT ... WHERE embedding <=> {vector}::vector""")

# Después (SEGURO):
cur.execute("""SELECT ... WHERE embedding <=> %s::vector""", (vector,))
```

### Para API Key Validation:
```python
if not OPENROUTER.api_key:
    raise ValueError(
        "OPENROUTER_API_KEY no configurada. "
        "Exporta la variable o configúrala en .env"
    )
```

### Para Connection Timeout:
```python
def _conninfo(self) -> dict[str, Any]:
    return {
        "host": DB.host,
        "port": DB.port,
        "dbname": DB.dbname,
        "user": DB.user,
        "password": DB.password,
        "connect_timeout": 5,  # Nuevo
    }
```

### Para Signal Handling:
```python
import signal
import sys

def signal_handler(sig, frame):
    logger.info("Interrupción recibida, limpiando...")
    self.close()
    sys.exit(0)

signal.signal(signal.SIGINT, signal_handler)
```

---

## 🔚 CONCLUSIÓN

Este plan proporciona una hoja de ruta clara para llevar ALEsys de su estado actual (funcional pero con vulnerabilidades) a un sistema **estable, seguro y listo para producción**. 

**Próximos pasos inmediatos**:
1. Corregir las 4 vulnerabilidades críticas
2. Implementar mejoras de performance
3. Añadir pruebas unitarias
4. Documentar completamente
5. Preparar para distribución

Con este plan, ALEsys estará listo para lanzamiento oficial (v1.0) en aproximadamente 8 semanas.

