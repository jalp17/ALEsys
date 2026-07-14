# ALEsys

**ALEsys** (Asistente de Desarrollo Multi‑Proyecto con IA) es una plataforma avanzada de ingeniería que opera enteramente en entornos locales. Su objetivo es facilitar la comprensión, búsqueda y generación de código en múltiples proyectos mediante técnicas de Recuperación Aumentada por Generación (RAG) y procesamiento completamente ejecutado en GPU/Vulkan para maximizar el rendimiento en hardware con recursos limitados.

---

## 1. Visión general

ALEsys permite indexar cualquier conjunto de archivos fuente, extraer metadatos técnicos y proporcionar un asistente conversacional capaz de responder preguntas basándose en el contexto del código y, opcionalmente, información obtenida de la web. Gracias a su motor de inferencia orientado a Vulkan y modelos GGUF, es posible ejecutar la totalidad del pipeline en APUs como AMD Vega 3 con 7 GB de memoria compartida.

## 2. Objetivos de diseño

* **Operación 100 % local**: no se transmiten datos a servicios externos; el usuario conserva control total de su código y consultas.
* **Escalabilidad multi‑proyecto**: la infraestructura admite múltiples bases de código aisladas, cada una con su propio índice y metadatos.
* **Optimización para hardware modesto**: el gestor de memoria controla la VRAM y los modelos se pueden precargar en hilos paralelos o cargarse secuencialmente según preferencias. Los parámetros de contexto son configurables por el usuario. El backend Vulkan de `llama-cpp-python` reduce la huella de memoria en comparación con CUDA.
* **Actualización dinámica**: el subcomando `index` puede ejecutarse en modo vigilante (`--watch`), monitorizando el directorio fuente y relanzando la indexación automáticamente cuando se detectan cambios en los archivos.
* **Interfaz enriquecida**: la GUI muestra metadatos detallados al seleccionar un modelo, incluyendo arquitectura, base model, parámetros y límites de contexto; además permite configurar la carga paralela y el tamaño de contexto.
* **Modularidad**: componentes separados de indexación, búsqueda, agentes conversacionales y GUI permiten mantenimiento y extensión sencillos.

## 3. Requisitos de sistema

| Componente | Mínimo | Recomendado |
|------------|--------|-------------|
| CPU        | Quad‑core x86\_64 | Hexa‑core+
| GPU/APU    | AMD Vega 3 (7 GB) o equivalente | NVidia con soporte Vulkan
| RAM        | 8 GB | 16 GB+
| Almacenamiento | 5 GB libres para modelos | 20 GB+
| Python     | 3.10‑3.14 | —

**Dependencias clave**
* Python: `llama-cpp-python`, `sentence-transformers`, `faiss-cpu` (o `faiss-gpu`), `duckduckgo-search`/`ddgs`, `PyQt6`/`customtkinter` (GUI opcional), `rich`.
* Compilación de `llama.cpp` con `-DGGML_VULKAN=on`.

### Modelos
en el directorio `~/llama.cpp/build-vulkan/bin/models/` debe haber al menos:
* `CDLM-0.5B.Q8_0.gguf` – modelo analista rápido.
* `ruvltra-1.1b-q4_k_m.gguf` – modelo conversacional por defecto.
* `ruvltra-claude-code-0.5b-q4_k_m.gguf` – alternativa conversacional.
* Modelos de embeddings: `imocha-ai-org/ssf-skill-extractor` y/o `Stephen-SMJ/DARE-R-Retriever`.

## 4. Arquitectura del código

```
/IA-Dev-System
├── core/                # Lógica de indexación y agentes
│   ├── memory_manager.py  # Control de carga/descarga de modelos
│   ├── indexer.py         # Escaneo/análisis/vectorización de código
│   ├── chat_agent.py      # Motor RAG conversacional
│   └── web_search.py      # Integración DuckDuckGo
├── gui/                 # Implementación de interfaz gráfica
│   ├── main_window.py
│   └── components.py
├── main.py              # CLI principal
├── requirements.txt     # Dependencias Python
└── projects/            # Proyectos indexados (config + vector_db)
```

### Flujo de trabajo
1. **Inicialización**: `main.py init <nombre> <ruta>` crea la configuración y prepara el directorio del proyecto.
2. **Indexación**: `main.py index <nombre>` recorre los archivos, fragmenta el código, genera resúmenes con `CDLM-0.5B` y calcula embeddings para almacenar en FAISS junto a metadatos de habilidades técnicas.
3. **Consulta**: `main.py chat <nombre>` arranca un bucle REPL. Para cada pregunta, se recuperan los chunks más relevantes, se construye un prompt con contexto y se solicita al modelo conversacional. Si está habilitada, se añade información de la web via `WebSearcher`.
4. **GUI opcional**: la aplicación gráfica replica las operaciones CLI con controles visuales y una pestaña de logs para depuración.

## 5. Instalación y configuración

```bash
# clonar el repositorio
git clone <url>
cd ALEsys

# preparar entorno virtual
python -m venv venv
source venv/bin/activate

# compilar llama.cpp con Vulkan
cd ~/llama.cpp
mkdir build && cd build
cmake -DLLAMA_CUBLAS=off -DGGML_VULKAN=on ..
make -j

# instalar dependencias Python
git -C /home/jesus/Documentos/proyectos/ALEsys checkout master
pip install --upgrade pip
pip install -r requirements.txt
```

> **Nota:** si `pip install llama-cpp-python` se realiza sin variables de entorno, especifique `CMAKE_ARGS="-DGGML_VULKAN=on"` para asegurarse de que el paquete se compile con soporte Vulkan.

## 6. Uso

### Interfaz de línea de comandos
```bash
python main.py init ejemplo ~/mi_proyecto
# indexar con parámetros avanzados
python main.py index ejemplo --context-size 4096 --no-parallel
python main.py index ejemplo --watch  # reindexa automáticamente al guardar cambios
python main.py chat ejemplo --model ruvltra-1.1b-q4_k_m.gguf --context-size 8192
python main.py chat ejemplo --no-web
python main.py list
python main.py info ejemplo
```

### Interfaz gráfica
```bash
python gui.py
```

### Variables de entorno de depuración
* `LOG_LEVEL=DEBUG` para ver salida detallada.
* `LLAMA_CPP_VERBOSE=true` para mensajes de `llama-cpp`.

## 7. Desarrollo y contribución

* Las pruebas unitarias se encuentran en `tests/` y cubren funcionalidades básicas de proyectos de ejemplo.
* Siga la guía de estilo definida en `pyproject.toml` (PEP 8/Black).
* Añada documentación detallada en docstrings y mantenga el formato compatible con Sphinx/ReST.

## 8. Licencia

Este proyecto se distribuye bajo la **Licencia MIT**. Consulte el fichero `LICENSE` para más detalles.

---

**Créditos**

Desarrollado por Jesús y colaboradores, con base en librerías open‑source como `llama-cpp`, `sentence-transformers` y `faiss`.
