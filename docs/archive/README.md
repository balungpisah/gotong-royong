# Archive

This directory contains documents that have been superseded, merged into current files, or preserved for historical traceability only. Nothing here is canonical.

## Why We Archive Instead of Delete

All unique content is preserved here to maintain audit trail, allow retrieval of historical reasoning, and prevent loss of context that may become relevant during future refactoring.

## Archived Files

### `ENGAGEMENT-STRATEGY-v0.1.md`

**Archived from**: `docs/design/specs/ENGAGEMENT-STRATEGY-v0.1.md`
**Reason**: Superseded by `ENGAGEMENT-STRATEGY-v0.2.md` (which is still active in `docs/design/specs/`). All unique Octalysis-framework content from v0.1 was confirmed to be present in v0.2 before archiving.
**Status**: Historical only — do not reference in implementation tickets.

### `research/surrealdb-pattern-sampling-legacy.md`

**Archived from**: `docs/research/surrealdb-pattern-sampling.md` (flat research root)
**Reason**: Superseded by `docs/research/references/surrealdb-pattern-sampling-v3-beta4.md`, which contains updated SurrealDB v3 beta patterns. The archived version covers early exploration on the v2/alpha API surface.
**Status**: Historical only — use `references/surrealdb-pattern-sampling-v3-beta4.md` for all current work.

## What Belongs Here

- Documents superseded by a newer version where unique content has been merged
- Documents describing decisions or approaches that were tried and abandoned
- Documents describing stack choices that were evaluated and rejected

## What Does NOT Belong Here

- Documents that are simply old but still referenced in active work
- ADRs — those live in `docs/architecture/adr/` permanently
- Research logs that informed current decisions — those stay in `docs/research/`
