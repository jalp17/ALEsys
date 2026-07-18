# Phase 7 Plan: Edicion y Ejecucion de Codigo

## Objetivo
IDE completo con editor de código, ejecución sandboxeada, y terminal embebida.

## Semana 19: CodeSandbox Backend

### Tareas
1. **`CodeSandbox` struct** (`crates/core/src/sandbox/mod.rs`)
   - `execute(code, language, timeout_ms, memory_limit_mb)` → `ExecutionResult`
   - Docker container management via `bollard` crate
   - Resource limits: CPU, memory, time
   - Filesystem aislado (solo /tmp)
   - Sin acceso a red

2. **Language runners**
   - Python: `python3 -c "{code}"`
   - JavaScript: `node -e "{code}"`
   - Rust: Compilar a binario temporal, ejecutar

3. **Streaming output**
   - stdout/stderr via Docker attach
   - Timeout handling

4. **Endpoint** `POST /api/v1/execute`
   ```rust
   pub struct ExecuteRequest {
       pub code: String,
       pub language: String,
       pub timeout_ms: Option<u64>,
       pub memory_limit_mb: Option<u64>,
   }
   ```

5. **Tests de seguridad**
   - No network access
   - No filesystem writes fuera de /tmp
   - Resource limits funcionan
   - Timeouts matan el proceso

---

## Semana 20: File Editor Backend

### Tareas
1. **`FileEditor` struct** (`crates/core/src/editor/mod.rs`)
   - `read_file(path)` → contenido
   - `write_file(path, content)` → resultado
   - `modify_file(path, old_content, new_content)` → diff
   - `list_files(dir)` → tree
   - Backup automático antes de modificar

2. **Diff generation**
   - `similar` crate para diffs
   - Unified diff format
   - Stats (líneas agregadas/eliminadas)

3. **Endpoint** `POST /api/v1/modify`
   ```rust
   pub struct ModifyRequest {
       pub path: String,
       pub old_content: String,
       pub new_content: String,
   }
   ```

4. **Endpoint** `GET /api/v1/files?path=...`
   - List directory
   - Read file content

5. **Tests**
   - Read/write cycles
   - Diff generation correcta
   - Backup creado

---

## Semana 21: Monaco Editor Frontend

### Tareas
1. **Instalar dependencias**
   ```bash
   pnpm add @monaco-editor/react
   ```

2. **`MonacoEditor` component** (`webui/src/components/editor/MonacoEditor.tsx`)
   - Language detection from file extension
   - Theme: dark (vs-dark)
   - Auto-save on blur
   - Keyboard shortcuts (Ctrl+S save, Ctrl+Z undo)

3. **`FileTree` component** (`webui/src/components/editor/FileTree.tsx`)
   - Tree view de archivos del proyecto
   - Click → abre en editor
   - Right-click → context menu (rename, delete)

4. **`editorService.ts`** (`webui/src/pages/editor/editorService.ts`)
   - `readFile(path)` → content
   - `writeFile(path, content)` → result
   - `listFiles(dir)` → tree
   - `executeCode(code, language)` → result

5. **`EditorPage`** (`webui/src/pages/editor/EditorPage.tsx`)
   - Layout: sidebar (file tree) + main (editor + tabs)
   - Tabs para múltiples archivos
   - Status bar (language, line/col)

---

## Semana 22: Terminal + Execute Button

### Tareas
1. **`Terminal` component** (`webui/src/components/editor/Terminal.tsx`)
   - xterm.js integration
   - WebSocket para output streaming
   - Input handling para comandos

2. **`ExecuteButton`**
   - Detecta lenguaje del archivo activo
   - Envía código a `/api/v1/execute`
   - Muestra output en terminal

3. **`DiffViewer` component**
   - Muestra diff antes de aplicar cambios
   - Botones: Apply / Discard
   - Syntax highlighting

4. **Integration**
   - Generate → abre en editor
   - Edit → preview diff
   - Execute → output en terminal

---

## Dependencias Nuevas

### Backend (Cargo)
```toml
bollard = "0.17"  # Docker API
similar = "2.6"   # Diff generation
```

### Frontend (pnpm)
```bash
pnpm add @monaco-editor/react
pnpm add @xterm/xterm
pnpm add @xterm/addon-fit
```

---

## Archivos a Crear

### Backend
- `crates/core/src/sandbox/mod.rs` — CodeSandbox
- `crates/core/src/sandbox/executor.rs` — Language runners
- `crates/core/src/editor/mod.rs` — FileEditor
- `crates/core/src/editor/diff.rs` — Diff utilities
- `crates/api/src/handlers/execute.rs` — Execute endpoint
- `crates/api/src/handlers/editor.rs` — Editor endpoints
- `docker/Dockerfile.sandbox` — Sandbox image

### Frontend
- `webui/src/components/editor/MonacoEditor.tsx`
- `webui/src/components/editor/FileTree.tsx`
- `webui/src/components/editor/Terminal.tsx`
- `webui/src/components/editor/DiffViewer.tsx`
- `webui/src/pages/editor/editorService.ts`
- `webui/src/pages/editor/EditorPage.tsx`

---

## Orden de Implementación

1. **Semana 19**: CodeSandbox (prioridad alta - core functionality)
2. **Semana 20**: File Editor (prioridad alta - needed for frontend)
3. **Semana 21**: Monaco + FileTree (prioridad media - UI)
4. **Semana 22**: Terminal + Integration (prioridad media - polish)

---

## Criterio de Éxito

- [ ] Ejecuta Python de forma segura
- [ ] Terminal muestra output en tiempo real
- [ ] Editor permite modificar y guardar
- [ ] 0 escapes del sandbox en tests
- [ ] Diff viewer funcional
- [ ] File tree navegable
