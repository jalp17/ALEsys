---
name: TICKET-29.3: PyMuPDFFallback Implementation
about: Implement CPU-only fallback for PDF extraction without GPU
title: '[TICKET-29.3] PyMuPDFFallback Implementation'
labels: 'fase29, pymupdf, fallback, ingestion'
---

## Description
Implement PyMuPDF fallback for environments without GPU, using pdfplumber for text/tables and fitz for images.

## Goals
- [ ] `extract_text(pdf_path)` → `Vec<PageText>` (pdfplumber)
- [ ] `extract_images(pdf_path, output_dir)` → `Vec<ImageRef>` (pymupdf)
- [ ] `extract_tables(pdf_path)` → `Vec<Table>` (pdfplumber)
- [ ] OCR opcional: `tesseract` subprocess por imagen
- [ ] Benchmark: comparar output vs MinerU en 10 papers

## Technical Details
- Python subprocess via tokio::process
- Optional Tesseract OCR
- No formula detection (limitación conocida)

## Acceptance Criteria
- [ ] Fallback works on test PDF without GPU
- [ ] Text extraction matches >80% of MinerU output
- [ ] Images extracted correctly

## Estimation
- ⏱️ 2 days