---
fecha: 2026-07-19
tipo: plan
proyecto: alesys
fase: 13
tags: [alesys, voice, multimodal, whisper, phase13]
status: en-progreso
---

# Phase 13: Voice + Multimodal

## Objetivo
Interacción por voz y comprensión de imágenes en ALEsys.

## Arquitectura

### Componentes

1. **Voice Engine** (`crates/core/src/voice/`)
   - Whisper integration (local, offline)
   - Text-to-voice con Piper
   - Audio capture via cpal
   - VAD (Voice Activity Detection)

2. **Image Understanding** (`crates/core/src/multimodal/`)
   - Screenshot → descripción → código
   - Diagrama → código (Mermaid, PlantUML)
   - OCR para texto en imágenes

3. **Command Parser** (`crates/core/src/voice/parser/`)
   - "abre archivo X"
   - "ejecuta tests"
   - "genera código para..."
   - "busca en el grafo..."

### Stack

- **Whisper**: whisper.cpp via whisper-rs
- **TTS**: Piper TTS (local, offline)
- **Audio**: cpal para capture
- **Image**: vision models via llama.cpp

## Fases

### 13.1 Voice Core (v1.20.0)
- [ ] Whisper integration para speech-to-text
- [ ] Audio capture con cpal
- [ ] VAD básico
- [ ] Command parser para voz
- [ ] Tests unitarios

### 13.2 Image + UI (v1.21.0)
- [ ] Image upload → descripción
- [ ] Screenshot → código
- [ ] UI: micrófono button en chat
- [ ] UI: drag-and-drop de imágenes
- [ ] Tests de integración

## Criterios de Éxito

- [ ] Reconocimiento de voz > 90% accuracy (offline)
- [ ] Latencia voz→texto < 500ms
- [ ] Genera código desde screenshot funcional
- [ ] Comandos de voz ejecutan acciones reales

---

**Tags:** #alesys #plan #voice #multimodal #whisper #phase13
