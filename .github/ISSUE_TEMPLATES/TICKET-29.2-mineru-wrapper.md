---
name: TICKET-29.2: MinerUWrapper Subprocess
about: Implement MinerU Python wrapper with tokio subprocess streaming
title: '[TICKET-29.2] MinerUWrapper Subprocess'
labels: 'fase29, mineru, ingestion, python'
---

## Description
Implement MinerUWrapper using tokio::process for subprocess isolation with streaming logs and configurable timeout.

## Goals
- [ ] `execute_magic_pdf(pdf_path, output_dir, options)` → `Result<MinerUOutput, Error>`
- [ ] Streaming stdout/stderr con `tracing` (info/debug/error)
- [ ] Timeout configurable (default 20h), kill graceful
- [ ] Auto-descarga modelos si no existen (`mineru_model_path`)
- [ ] GPU detection: `nvidia-smi` + `torch.cuda.is_available()`
- [ ] Retry logic: 1 reintento con fallback si OOM

## Technical Details
- Use tokio::process::Command
- Parse magic-pdf CLI output
- Handle both CUDA and CPU modes

## Acceptance Criteria
- [ ] MinerU executes successfully on test PDF
- [ ] Timeout works correctly
- [ ] GPU detection passes

## Estimation
- ⏱️ 3 days