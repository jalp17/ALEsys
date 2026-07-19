---
fecha: 2026-07-19
tipo: plan
proyecto: alesys
fase: 14
tags: [alesys, collaboration, realtime, websocket, phase14]
status: en-progreso
---

# Phase 14: Real-Time Collaboration

## Objetivo
Múltiples usuarios editando y ejecutando código juntos en tiempo real.

## Arquitectura

### Componentes

1. **Collaboration Server** (`crates/core/src/collaboration/`)
   - WebSocket rooms por sesión/proyecto
   - Operational Transform (OT) para edición
   - Presence indicators
   - Cursor sync

2. **OT Engine** (`crates/core/src/collaboration/ot.rs`)
   - Transform de operaciones concurrentes
   - Resolución de conflictos
   - Historial de operaciones

3. **Presence System** (`crates/core/src/collaboration/presence.rs`)
   - Quién está online
   - Ubicación de cursor
   - Estado (typing, idle, etc.)

4. **Shared Terminal** (`crates/core/src/collaboration/terminal.rs`)
   - Terminal embebida compartida
   - Output visible para todos
   - Input por turnos

### Protocolo

```rust
// Operación OT
Operation {
    id: Uuid,
    user_id: String,
    position: usize,
    action: OpAction, // Insert, Delete, Retain
    content: Option<String>,
}

// Presencia
Presence {
    user_id: String,
    cursor_position: Option<usize>,
    selection: Option<Range>,
    status: UserStatus, // Active, Idle, Typing
}

// Mensaje WebSocket
CollabMessage {
    room_id: String,
    user_id: String,
    payload: CollabPayload,
}
```

## Fases

### 14.1 Collaboration Core (v1.22.0)
- [ ] WebSocket room management
- [ ] OT engine básico
- [ ] Presence system
- [ ] Cursor sync
- [ ] Tests unitarios

### 14.2 Shared Terminal + UI (v1.23.0)
- [ ] Shared terminal component
- [ ] User avatars/cursors en editor
- [ ] Presence panel
- [ ] Voice chat stub (WebRTC placeholder)
- [ ] Tests de integración

## Criterios de Éxito

- [ ] 5 usuarios editan mismo archivo sin conflictos
- [ ] Cursors remotos se actualizan en < 100ms
- [ ] Presence indicators funcionales
- [ ] Shared terminal muestra output para todos

---

**Tags:** #alesys #plan #collaboration #realtime #phase14
