# Ontology Enrichment Guardrails

Purpose:
- Keep ontology-to-feed enrichment behavior stable as code evolves.
- Provide one quick command to detect regressions in the high-risk areas.

## Guardrails

1. `POST /v1/ontology/feed` stays idempotent (`ontology_note_create` idempotency key).
2. `HasAction` triples always normalize to `action:*` record IDs.
3. Async enrichment is only enqueued when sync enrichment fails.
4. Feedback and tags use separate timestamps:
- `feedback_enriched_at_ms` for vouch/challenge updates.
- `tags_enriched_at_ms` for worker tag/label refresh updates.
5. Public-only feed ingestion for ontology notes (`rahasia_level == 0`).
6. Chat/feed/notification request paths never depend on graph traversal; ontology stays enrichment/audit side-path.
7. If enrichment read/write fails, API still returns core feed item and keeps `enrichment.status` deterministic (`pending|computed`).

## One-command checks

- Fast structural checks:
```bash
just ontology-enrichment-check
```

- Structural checks + tests:
```bash
just ontology-enrichment-check-full
```

## When to run

Run these checks before merging any change that touches:
- `crates/api/src/routes/mod.rs`
- `crates/worker/src/main.rs`
- `crates/domain/src/ports/jobs.rs`
- `docs/architecture/hot-path-api-shapes.md`

## What this does not replace

- The script is a guardrail, not a benchmark.
- Continue using live-data validation for SurrealDB behavior (`just dev-db-up`, smoke tests, release gates).
