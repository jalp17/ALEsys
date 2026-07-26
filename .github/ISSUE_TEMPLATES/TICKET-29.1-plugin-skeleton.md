---
name: TICKET-29.1: Plugin Skeleton + Config
about: Create ingestion module structure and plugin with Fase 11 Plugin System
title: '[TICKET-29.1] Plugin Skeleton + Config'
labels: 'fase29, plugin, ingestion, foundation'
---

## Description
Create the ingestion module structure and implement IngestionPlugin using Fase 11 Plugin System with granular permissions.

## Goals
- [ ] Create `crates/core/src/ingestion/` structure
- [ ] Implement `IngestionPlugin` with `Plugin` trait (Fase 11)
- [ ] Config schema: `mineru_model_path`, `output_base_dir`, `fallback_enabled`, `default_ocr_langs`
- [ ] `on_init`: verifica Python 3.10+, `magic-pdf --version`, device CUDA
- [ ] Tests: plugin load/unload, config validation

## Technical Details
- Permissions: `filesystem:read,write`, `execute:python3,magic-pdf`
- IngestionMode enum: Full (with GraphRAG) / FilesOnly (Colab-compatible)
- Default mode: FilesOnly per current config

## Acceptance Criteria
- [ ] Plugin initializes without errors
- [ ] Configuration validates correctly
- [ ] Can execute `ingest.pdf` command
- [ ] Tests pass

## Estimation
- ⏱️ 2 days

## Dependencies
- Fase 11 Plugin System must be available