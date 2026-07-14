# ALEsys

**ALEsys** es un pipeline de ingesta híbrida que combina almacenamiento vectorial y de grafos de conocimiento sobre PostgreSQL. Escanea bibliotecas de libros en Markdown, extrae entidades y relaciones científicas mediante IA en la nube (OpenRouter), genera embeddings localmente en CPU y persiste todo en PostgreSQL con `pgvector`.

---

## 1. Objetivo

Construir un sistema que permita búsquedas semánticas y navegación por grafos de conocimiento a partir de documentos Markdown no estructurados.

1. Escanear recursivamente una biblioteca de libros en Markdown.
2. Generar embeddings (384 dimensiones) localmente en CPU con `sentence-transformers`.
3. Extraer entidades científicas y sus relaciones mediante LLMs estructurados vía OpenRouter.
4. Almacenar en PostgreSQL con `pgvector` y tablas relacionales de grafo.
5. Consultar combinando búsqueda vectorial + navegación por grafo.

---

## 2. Entorno de ejecución

| Componente | Especificación |
|------------|---------------|
| Host | Fedora Server 43, AMD Ryzen 5, 16 GB RAM |
| Base de Datos | PostgreSQL en Docker (`postgres_db`, puerto `5432`) con extensión `pgvector` |
| Modelo de embeddings | `sentence-transformers/all-MiniLM-L6-v2` (384 dim, CPU) |
| Modelo de extracción | `google/gemini-2.5-flash-free` vía OpenRouter |
| Asistente de código | `codestral:cloud` o `qwen3-coder:480b-cloud` |

---

## 3. Arquitectura modular

```
ALEsys/
├── config.py          # Credenciales, rutas y constantes
├── db_manager.py      # Conexión PostgreSQL, tablas y CRUD
├── embedder.py        # Embeddings locales con sentence-transformers
├── extractor.py       # Extracción de entidades/relaciones vía OpenRouter
├── pipeline.py        # Orquestador: escaneo → chunking → embedding → extracción → persistencia
├── test_queries.py    # Consultas de ejemplo (vectorial, grafo, híbrida, ask)
├── main.py            # CLI: db-init, db-drop, run, query, ask, list
├── gui.py             # GUI tkinter con pestañas Pipeline / Search / Chat
├── run_tests.sh       # Suite automatizada de validación
├── core/
│   ├── chat_agent.py  # Chat con contexto RAG (vectorial + grafo)
│   └── web_search.py  # Búsqueda web DuckDuckGo para contexto adicional
├── requirements.txt   # Dependencias
└── projects/          # Proyectos (estructura legacy)
```

### Flujo de datos

```
Libros .md → Pipeline
               ├→ chunking → embedding (CPU) → PostgreSQL (fragmentos + vector)
               └→ chunking → extracción LLM (OpenRouter) → PostgreSQL (entidades + relaciones)
                                                              └→ búsqueda híbrida (vector + grafo)
```

---

## 4. Instalación

```bash
git clone <url>
cd ALEsys
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### Requisitos del sistema

- Docker con imagen PostgreSQL + extensión `pgvector`
- Cuenta en [OpenRouter](https://openrouter.ai) (clave en `OPENROUTER_API_KEY`)
- Variable `OPENROUTER_API_KEY` configurada

---

## 5. Uso

### 5.1 Inicializar base de datos

```bash
python main.py db-init
```

### 5.2 Pipeline de ingesta

```bash
# Vista previa (dry-run) sin indexar
python main.py run --input /ruta/a/libros/ --dry-run

# Pipeline completa
python main.py run --input /ruta/a/libros/ --chunk-size 1000 --chunk-overlap 200
```

### 5.3 Consultas

```bash
# Búsqueda vectorial
python main.py query "mecánica cuántica"

# Búsqueda en grafo por entidad
python main.py query --graph "Heisenberg"

# Búsqueda híbrida (vector + grafo)
python main.py query "ecuación de Schrödinger" --hybrid

# Preguntar con contexto RAG
python main.py ask "¿Qué es el principio de incertidumbre?" --top-k 5
```

### 5.4 Listar documentos indexados

```bash
python main.py list
```

### 5.5 Eliminar tablas

```bash
python main.py db-drop          # pide confirmación
python main.py db-drop --force  # sin confirmar
```

### 5.6 Interfaz gráfica

```bash
python gui.py
```

Ofrece pestañas para:
- **Pipeline**: inicializar BD, ejecutar ingesta, vista previa
- **Search**: búsqueda vectorial, por grafo o híbrida
- **Chat**: preguntar con contexto RAG

### 5.7 Suite de validación

```bash
bash run_tests.sh
```

Ejecuta: sintaxis, imports, conexión PostgreSQL, embeddings, pipeline en modo prueba, consultas y verificación de deduplicación.

---

## 6. Licencia

MIT — ver `LICENSE`.

---

## Créditos

Desarrollado por Jalp17. Inspirado en GraphRAG y pgvector.
