---
name: TICKET-29.4: Organizer Reorganización
about: Reorganize MinerU output into clean book structure
title: '[TICKET-29.4] Organizer Reorganización'
labels: '[TICKET-29.4], organizer, mineru, ingestion'
---

## Description
Port `reordenar_db_p.py` logic to Rust: reorganize MinerU output into clean book/chapter structure.

## Goals
- [ ] `reorganize(mineru_output_dir, target_dir)` → `OrganizedOutput`
- [ ] Parse MD: extrae `![]()` refs → Set<Path>
- [ ] Move referenciadas a `target_dir/images/`
- [ ] Move MD a `target_dir/chapter.md` (primer nivel)
- [ ] Cleanup: `rm -rf auto/`, duplicados, dirs vacíos
- [ ] Log generation: `_reorg_logs/{book}_{timestamp}.log`

## Technical Details
- Regex parsing for image references
- Path movement with validation
- Log per book processed

## Acceptance Criteria
- [ ] Output directory has clean structure
- [ ] Only referenced images moved
- [ ] No broken links in markdown

## Estimation
- ⏱️ 2 days