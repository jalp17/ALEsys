# ALEsys — GraphRAG-PG

**ALEsys** (GraphRAG-PG: PostgreSQL Graph & Vector Ingestion Engine) es una pipeline de ingesta híbrida que combina almacenamiento relacional, vectorial y de grafos de conocimiento sobre PostgreSQL. Está diseñada para escanear bibliotecas de libros en Markdown, extraer entidades y relaciones científicas mediante IA en la nube, generar embeddings localmente en CPU y persistir todo en PostgreSQL con `pgvector`.

---

## 1. Objetivo

Construir un sistema que permita búsquedas semánticas y navegación por grafos de conocimiento científicos, partiendo de documentos Markdown sin estructurar.

1. Escanear recursivamente una biblioteca de libros en Markdown.
2. Generar embeddings (vectores de 384 dimensiones) localmente en CPU.
3. Extraer entidades científicas y sus relaciones mediante LLMs estructurados en la nube (OpenRouter).
4. Almacenar datos en PostgreSQL con `pgvector` y tablas relacionales de grafos.

---

## 2. Entorno de ejecución

| Componente | Especificación |
|------------|---------------|
| Host | Fedora Server 43, AMD Ryzen 5, 16 GB RAM |
| Base de Datos | PostgreSQL en Docker (`postgres_db`, puerto `5432`) con extensión `pgvector` |
| Inferencia local | Ollama en Docker (`http://ollama:11434`) |
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
├── test_queries.py    # Pruebas de búsqueda híbrida (vectorial + grafo)
├── requirements.txt   # Dependencias
├── main.py            # CLI (legacy — pendiente de adaptación)
├── gui.py             # GUI (legacy — pendiente de adaptación)
├── core/              # Módulos legacy (pendientes de migración)
├── projects/          # Proyectos (estructura legacy)
└── tests/             # Pruebas unitarias
```

### Flujo de datos

```
Libros .md → Pipeline
               ├→ chunking → embedding (local CPU) → PostgreSQL (fragmentos + vector)
               └→ chunking → extracción (OpenRouter) → PostgreSQL (entidades + relaciones)
                                                           └→ búsquedas híbridas (test_queries.py)
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

- Docker con imagen PostgreSQL + pgvector
- Ollama en Docker (para proxy de embeddings)
- Cuenta en OpenRouter (para extracción de grafos)

---

## 5. Uso

```bash
# Fase 1: Verificar conexión a base de datos
python -c "from db_manager import DatabaseManager; db = DatabaseManager(); db.initialize_tables()"

# Fase 2-4: Pipeline completa (cuando esté implementada)
python pipeline.py --input /ruta/a/libros/
```

---

## 6. Licencia

MIT — ver `LICENSE`.

---

## Créditos

Desarrollado por Jalp17. Inspirado en técnicas de GraphRAG y pgvector.
