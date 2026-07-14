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
