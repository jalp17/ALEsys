# ALEsys: Asistente de Desarrollo Multi-Proyecto (100% Local)

ALEsys es un asistente de desarrollo con Inteligencia Artificial diseñado para trabajar localmente con múltiples proyectos. Utiliza la Generación Aumentada por Recuperación (RAG) combinada con búsquedas en internet y una arquitectura optimizada para correr en hardware limitado (APUs AMD Vega, por ejemplo) con **llama.cpp / Vulkan**.

## Características Principales

1. **100% Local y Privado**: Todo el análisis, incrustaciones (embeddings) y generación de código se ejecutan en tu máquina.
2. **Sistema Multi-Proyecto**: Puedes inicializar, indexar y chatear con múltiples bases de código de manera independiente gracias a los comandos CLI intuitivos y la GUI.
3. **Análisis por CDLM**: Durante la indexación, un modelo rápido de Llama comprende el código en fragmentos y genera resúmenes.
4. **Metadatos de Habilidades Técnicas**: Usa el modelo especializado `imocha-ai-org/ssf-skill-extractor` para enriquecer la base vectorial identificando habilidades técnicas e integrándolas en el contexto del código.
5. **Generación RAG e Inferencia Local Avanzada**: El proyecto utiliza `Stephen-SMJ/DARE-R-Retriever` para un mapeo léxico avanzado en FAISS. El chat se lidera con modelos GGUF conversacionales punteros, como `ruvltra-1.1b-q4_k_m.gguf` o `ruvltra-claude-code-0.5b-q4_k_m.gguf`.
6. **Búsqueda Web de Respaldo**: Búsqueda en DuckDuckGo integrada para complementar la documentación, buscar manejo de errores y sintaxis del lenguaje.

## Prerrequisitos

ALEsys está fuertemente optimizado para ser ejecutado con **llama-cpp-python** usando el backend **Vulkan**, crucial para aprovechar tarjetas gráficas o APUs con memoria compartida.

```bash
# Reinstalación forzada con backend Vulkan
CMAKE_ARGS="-DGGML_VULKAN=on" pip install llama-cpp-python --force-reinstall --no-cache-dir
```

Tras compilar/instalar llama.cpp, instala las demás dependencias:

```bash
pip install -r requirements.txt
```

### Modelos Necesarios

Asegúrate de tener los modelos descargados:

*   **Directorio GGUF:** Estándar en `~/llama.cpp/build-vulkan/bin/models/`.
*   **Modelos de Lenguaje / Analistas:**
    *   `CDLM-0.5B.Q8_0.gguf` (Analista por defecto al indexar).
    *   `ruvltra-1.1b-q4_k_m.gguf` (Conversacional preferido).
    *   `ruvltra-claude-code-0.5b-q4_k_m.gguf` (Conversacional alternativo).
*   **Modelos de Embeddings y Extracción:**
    *   `Stephen-SMJ/DARE-R-Retriever` (SentenceTransformers, para vectores RAG).
    *   `imocha-ai-org/ssf-skill-extractor` (Para indexar y abstraer *technical skills* en cada archivo analizado).

## Uso a través de la Interfaz Gráfica (GUI)

ALEsys incluye una GUI ligera e intuitiva construida en `tkinter`, sin dependencias pesadas:

```bash
python gui.py
```
Desde la interfaz puedes configurar rutas, visualizar tus proyectos, gestionar la indexación (con o sin generador LLM), activar la ayuda web y conversar de forma interactiva en la pestaña *Chat*.

## Uso a través de la Interfaz de Comandos (CLI)

El punto de entrada principal desde el terminal es `main.py`. Cuenta con un set de subcomandos para la gestión del ciclo de vida del código:

### 1. Inicializar Proyecto (`init`)
Crea la configuración para un nuevo proyecto a partir de un directorio.
```bash
python main.py init mi_app /ruta/al/codigo
```

### 2. Indexar Código (`index`)
Inicia la fase de análisis con CDLM y vectorización. Aquí se creará la base de datos de fragmentos con Faiss, incorporando la abstracción de metadatos de habilidades.
```bash
python main.py index mi_app
```
*(Puedes añadir `--skip-summaries` para una indexación extremadamente rápida pero únicamente léxica/simbólica sin análisis profundo LLM).*

### 3. Iniciar Chat (`chat`)
Abre una sesión interactiva conversacional. ALEsys escaneará tu base vectorial y si fallara tratará de obtener referencias en internet.
```bash
python main.py chat mi_app
```

### Más Comandos
*   **Listar proyectos disponibles:** `python main.py list`
*   **Ver información métrica del proyecto:** `python main.py info mi_app`

---

**Licencia y Créditos:**
Desarrollado como una arquitectura RAG eficiente y respetuosa con los recursos priorizando las soluciones open-source.
