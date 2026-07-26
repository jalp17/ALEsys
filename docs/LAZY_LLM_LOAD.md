# Carga Lazy del Modelo LLM

## Resumen

ALEsys ahora soporta **carga bajo demanda** del modelo LLM. El modelo **NO se carga automáticamente** al iniciar el servidor, sino que debe cargarse explícitamente mediante un endpoint API o botón en el frontend.

## Beneficios

### 1. **Ahorro de RAM Inmediato**
- **Antes:** 4-8 GB RAM al iniciar (modelo cargado automáticamente)
- **Ahora:** ~200 MB RAM al iniciar (solo backend configurado, sin modelo)
- **Ahorro:** 95% menos RAM en idle

### 2. **Control Total**
- Tú decides CUÁNDO cargar el modelo
- Puedes descargar el modelo cuando no lo uses
- Ideal para servidores con recursos limitados

### 3. **Modo Search-Only**
- El servidor puede correr indefinidamente sin el LLM
- Búsquedas en el grafo funcionan sin modelo
- Perfecto para APIs de solo consulta

## Arquitectura

### Flujo Anterior (Auto-Load)

```
Inicio del Servidor
    ↓
AppState::new()
    ↓
LLMBackend::from_config()  ← Carga el modelo INMEDIATAMENTE
    ↓
4-8 GB RAM consumidos
```

### Flujo Nuevo (Lazy-Load)

```
Inicio del Servidor
    ↓
AppState::new()
    ↓
LLMBackend::from_config_lazy()  ← SOLO configura, NO carga
    ↓
~200 MB RAM (sin modelo)
    ↓
... tiempo pasa ...
    ↓
Usuario llama a POST /api/v1/llm/load
    ↓
LLMBackend::load()  ← Recarga el modelo
    ↓
4-8 GB RAM consumidos
```

## Endpoints Nuevos

### 1. GET `/api/v1/llm/status`

Verifica el estado actual del LLM.

**Response:**
```json
{
  "loaded": false,
  "backend": "llama_cpp",
  "state": "unloaded",
  "model_path": "/models/Qwen3-MoE-q4_k_m.gguf",
  "message": "LLM no cargado (backend=llama_cpp). Usar POST /api/v1/llm/load para cargar."
}
```

**Estados posibles:**
- `unloaded` - Modelo no cargado (0 MB RAM)
- `loaded` - Modelo cargado y listo
- `error` - Error al cargar

### 2. POST `/api/v1/llm/load`

Carga el modelo en memoria.

**Request:**
```json
{
  "force": false  // Opcional: forzar recarga si ya está cargado
}
```

**Response (éxito):**
```json
{
  "success": true,
  "backend": "llama_cpp",
  "model_path": "/models/Qwen3-MoE-q4_k_m.gguf",
  "estimated_ram_mb": 1024,
  "message": "Modelo cargado exitosamente en memoria"
}
```

**Response (ya cargado):**
```json
{
  "status": 409,
  "error": "LLM ya está cargado. Usar force=true para recargar."
}
```

### 3. POST `/api/v1/llm/unload`

Descarga el modelo de la memoria (libera RAM).

**Response:**
```json
{
  "success": true,
  "message": "Modelo descargado. 1024 MB de RAM liberados.",
  "ram_freed_mb": 1024
}
```

## Uso desde el Frontend

### Ejemplo con React

```tsx
// Componente LLMControl
function LLMControl() {
  const [llmStatus, setLlmStatus] = useState(null);
  const [loading, setLoading] = useState(false);

  // Verificar estado al montar
  useEffect(() => {
    checkStatus();
  }, []);

  const checkStatus = async () => {
    const res = await fetch('/api/v1/llm/status');
    const data = await res.json();
    setLlmStatus(data);
  };

  const loadModel = async () => {
    setLoading(true);
    const res = await fetch('/api/v1/llm/load', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ force: false })
    });
    
    if (res.ok) {
      alert('✅ Modelo cargado');
      checkStatus();
    } else {
      const err = await res.json();
      alert(`❌ Error: ${err.error}`);
    }
    setLoading(false);
  };

  const unloadModel = async () => {
    setLoading(true);
    const res = await fetch('/api/v1/llm/unload', {
      method: 'POST'
    });
    
    if (res.ok) {
      const data = await res.json();
      alert(`✅ Modelo descargado. ${data.ram_freed_mb} MB liberados`);
      checkStatus();
    }
    setLoading(false);
  };

  return (
    <div className="llm-control">
      <h3>LLM Status</h3>
      {llmStatus && (
        <div>
          <p>Backend: {llmStatus.backend}</p>
          <p>Estado: {llmStatus.state}</p>
          <p>Modelo: {llmStatus.model_path || 'N/A'}</p>
          <p>RAM Estimada: {llmStatus.loaded ? '~4 GB' : '0 MB'}</p>
        </div>
      )}
      
      <div className="buttons">
        {!llmStatus?.loaded ? (
          <button onClick={loadModel} disabled={loading}>
            🚀 Cargar Modelo
          </button>
        ) : (
          <button onClick={unloadModel} disabled={loading}>
            🛑 Descargar Modelo (Liberar RAM)
          </button>
        )}
        <button onClick={checkStatus} disabled={loading}>
          🔄 Refresh
        </button>
      </div>
    </div>
  );
}
```

## Uso desde CLI

### Verificar estado

```bash
curl http://localhost:3000/api/v1/llm/status | jq
```

### Cargar modelo

```bash
curl -X POST http://localhost:3000/api/v1/llm/load \
  -H "Content-Type: application/json" \
  -d '{"force": false}' | jq
```

### Descargar modelo

```bash
curl -X POST http://localhost:3000/api/v1/llm/unload | jq
```

### Script de utilidad

```bash
#!/bin/bash
# llm-control.sh

API_URL="http://localhost:3000/api/v1/llm"

case "$1" in
  status)
    curl -s "$API_URL/status" | jq
    ;;
  load)
    curl -s -X POST "$API_URL/load" \
      -H "Content-Type: application/json" \
      -d '{"force": false}' | jq
    ;;
  unload)
    curl -s -X POST "$API_URL/unload" | jq
    ;;
  *)
    echo "Uso: $0 {status|load|unload}"
    exit 1
    ;;
esac
```

## Cambios en el Código

### 1. `crates/core/src/llm/backend.rs`

- Nuevo enum `LLMState` (Unloaded, Loaded, Error)
- `LLMBackend` ahora usa `Option<Engine>` para cada backend
- Nuevo método `from_config_lazy()` - configura sin cargar
- Nuevo método `load()` - carga el modelo on-demand
- Nuevo método `unload()` - descarga el modelo
- Nuevo método `is_loaded()` - verifica estado
- Nuevo método `state()` - retorna LLMState

### 2. `crates/api/src/state.rs`

- `AppState.llm_engine` ahora es `Arc<RwLock<LLMBackend>>`
- `LLMQueue` ahora soporta estado lazy
- `LLMQueue::new_lazy()` - crea queue sin cargar modelo
- `LLMQueue::load()` - carga el modelo
- `LLMQueue::unload()` - descarga el modelo
- `LLMQueue::is_loaded()` - verifica estado
- `LLMQueue::state()` - retorna estado

### 3. `crates/api/src/handlers.rs`

- Nuevos handlers: `get_llm_status()`, `load_llm()`, `unload_llm()`
- Nuevas structs: `LLMStatusResponse`, `LoadLLMRequest`, `LoadLLMResponse`, `UnloadLLMResponse`
- Función `estimate_model_ram()` - estima RAM basada en el modelo

### 4. `crates/api/src/main.rs`

- Nuevas rutas:
  - `GET /api/v1/llm/status`
  - `POST /api/v1/llm/load`
  - `POST /api/v1/llm/unload`

## Estimación de RAM

La función `estimate_model_ram()` estima el consumo basado en el nombre del modelo:

| Modelo | RAM Estimada |
|--------|--------------|
| Tiny/0.5B/1B | 600 MB |
| 2B/3B/Phi | 2 GB |
| 4B-8B | 4 GB |
| 13B-30B | 8 GB |
| MoE (Qwen3-MoE) | 1 GB |
| MoE (Mixtral 8x7B) | 8 GB |
| Default | 2 GB |

## Escenarios de Uso

### 1. Servidor de Desarrollo

```bash
# Iniciar servidor (sin cargar modelo)
cargo run --bin alesys-api

# Verificar que está en ~200 MB RAM
ps -o pid,rss,comm -C alesys-api

# Cargar modelo solo cuando se necesita
curl -X POST http://localhost:3000/api/v1/llm/load

# Trabajar...

# Descargar modelo cuando se termina
curl -X POST http://localhost:3000/api/v1/llm/unload
```

### 2. Servidor de Producción (Recursos Limitados)

```bash
# Iniciar en Docker
docker run -d \
  -e LLM_BACKEND=llama_cpp \
  -e LLM_MODEL_PATH=/models/Qwen3-MoE.gguf \
  --memory=1g \  # Solo 1 GB inicial
  alesys-api

# Cargar modelo solo durante horario laboral
# (usar cron o Kubernetes Job)
0 8 * * * curl -X POST http://alesys:3000/api/v1/llm/load
0 18 * * * curl -X POST http://alesys:3000/api/v1/llm/unload
```

### 3. Servidor Search-Only

```bash
# Nunca cargar el modelo
# El servidor funciona como API de búsqueda en el grafo

# Búsquedas funcionan normal
curl http://localhost:3000/api/v1/graph/search?q=rust

# Chat NO funciona (retorna error "LLM no cargado")
curl -X POST http://localhost:3000/api/v1/chat \
  -H "Content-Type: application/json" \
  -d '{"query": "hola"}'
# Error: "LLM no cargado. Usar POST /api/v1/llm/load para cargar."
```

## Migración desde Versión Anterior

### Cambios Requeridos

**No hay cambios requeridos** - la carga lazy es el comportamiento por defecto.

### Comportamiento Anterior (si se necesita)

Si necesitas el comportamiento antiguo (carga automática al inicio):

```rust
// En crates/api/src/state.rs
// Cambiar:
let llm_engine = LLMBackend::from_config_lazy(llm_config.clone()).await?;

// Por:
let llm_engine = LLMBackend::from_config(llm_config.clone()).await?;
```

## Troubleshooting

### "LLM no cargado" error en chat

**Solución:** Cargar el modelo primero

```bash
curl -X POST http://localhost:3000/api/v1/llm/load
```

### "LLM ya está cargado" error

**Solución:** El modelo ya está en memoria. Usar `force=true` para recargar:

```bash
curl -X POST http://localhost:3000/api/v1/llm/load \
  -H "Content-Type: application/json" \
  -d '{"force": true}'
```

### Modelo no se descarga

**Posible causa:** Hay requests en progreso

**Solución:** Esperar a que terminen las requests activas, luego intentar de nuevo.

### RAM no se libera después de unload

**Posible causa:** El garbage collector de Rust aún no libera la memoria

**Solución:** Esperar unos segundos o reiniciar el servidor si es crítico.

## Métricas

### Ver consumo de RAM

```bash
# Linux
ps -o pid,rss,vsz,comm -C alesys-api

# Ver cambio en tiempo real
watch -n1 'ps -o pid,rss,comm -C alesys-api'
```

### Ver métricas de Prometheus

```bash
# Endpoint de métricas
curl http://localhost:3000/metrics | grep llm

# Posibles métricas (si se implementan)
# alesys_llm_loaded{backend="llama_cpp"} 1
# alesys_llm_ram_bytes 1073741824
```

## Futuras Mejoras

- [ ] Auto-unload después de X minutos de inactividad
- [ ] Métricas de Prometheus para estado del LLM
- [ ] Webhook/notificación cuando el modelo se carga/descarga
- [ ] Streaming implementado para lazy load
- [ ] Precalentamiento del modelo (warm-up)
- [ ] Múltiples modelos cargados simultáneamente

---

**Documentación creada:** 2026-07-20  
**Versión:** ALEsys v2.0.0+lazy-llm
