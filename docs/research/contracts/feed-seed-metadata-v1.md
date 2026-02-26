# Feed Seed Metadata Contract (v1)

Last updated: 2026-02-26
Owner: frontend + API cutover slice
Status: active reference

## Purpose

Define a single optional metadata shape for seeded/sample feed cards used in development and test environments.

Goals:
- make seeded cards explicit in UI and QA flows,
- keep seed labeling separate from business semantics (`entity_tags`, `taxonomy`, `program_refs`),
- keep runtime-compatible with future DB/operator seeding.

## Contract

`FeedItem.dev_meta` (frontend) maps from `payload.dev_meta` (API feed payload):

```json
{
  "dev_meta": {
    "is_seed": true,
    "seed_batch_id": "fixture-taxonomy-v1",
    "seed_origin": "fixture"
  }
}
```

Fields:
- `is_seed` (`boolean`, required when `dev_meta` exists)
- `seed_batch_id` (`string`, optional)
- `seed_origin` (`"fixture" | "db" | "operator_stub"`, optional)

## Runtime Rules

- `dev_meta` is optional and non-breaking.
- Production-authored cards should omit `dev_meta`.
- Frontend may render `SEED` badge/filter only in development mode.
- Seed labeling must not be encoded in `entity_tags`.

## Source Compatibility

- `seed_origin=fixture`: local fixture matrix from frontend fixture files.
- `seed_origin=db`: backend returns seeded rows from local/staging dev databases.
- `seed_origin=operator_stub`: backend/operator stubs emit seeded contract payloads.

## Validation Guidance

- Ignore invalid `dev_meta` payloads safely.
- Accept only allowed `seed_origin` enum values.
- Preserve all existing feed mapping behavior when `dev_meta` is absent.
