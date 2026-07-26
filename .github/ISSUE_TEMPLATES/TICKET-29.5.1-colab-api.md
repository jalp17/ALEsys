---
name: TICKET-29.5.1: Colab-Compatible API
about: Create API endpoints without GraphRAG dependency for Google Colab compatibility
title: '[TICKET-29.5.1] Colab-Compatible API'
labels: 'fase29, api, colab, enhancement'
---

## Description
Create API endpoints that generate ZIP output with MD + images without GraphRAG database writes, enabling Google Colab compatibility.

## Goals
- [ ] Create `routes_colab.rs`: endpoints sin GraphRAG dependency
- [ ] Output ZIP con estructura: `book/chapter.md`, `book/images/`, `metadata.json`
- [ ] Endpoint `/ingestion/colab/process` → response con ZIP download URL
- [ ] Script `notebooks/ingest_colab.ipynb`: wrapper Python → API calls

## Technical Details
- Mode parameter in API: `files_only` skips GraphRAG indexing
- ZIP packaging at `/api/v1/ingestion/colab/process`
- Streaming progress via WS for long-running ingestion

## Acceptance Criteria
- [ ] POST `/api/v1/ingestion/colab/process` returns ZIP download URL
- [ ] ZIP contains valid markdown with working image references
- [ ] No database writes when mode=`files_only`
- [ ] Notebook `notebooks/ingest_colab.ipynb` executes successfully
- [ ] API responds < 200ms, WS progress every 5s

## Dependencies
- TICKET-29.5 (PDFProcessor Orchestrator) must be completed

## Estimation
- ⏱️ 1 day