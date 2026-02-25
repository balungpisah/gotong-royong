# Ontology Feed Expiry Backfill

Purpose:
- Hide already-expired ontology feed rows that were created before TTL->feed lifecycle hiding was implemented.
- Keep discovery feed/search aligned with ontology TTL cleanup behavior.

## Command

Dry-run first:

```bash
just ontology-feed-backfill-expired --dry-run
```

Apply:

```bash
just ontology-feed-backfill-expired
```

Direct worker invocation:

```bash
cargo run -p gotong-worker -- ontology-feed-backfill-expired [flags]
```

## Flags

- `--dry-run`  
  Scans and reports candidates without mutating feed payloads.
- `--page-size <N>`  
  Number of feed rows fetched per page (default `500`, max `10000`).
- `--progress-every <N>`  
  Progress print interval by scanned row count (default `500`).
- `--cutoff-ms <EPOCH_MS>`  
  Optional explicit TTL cutoff; default is current `now_ms()` at command start.

## Selection Rule

A feed row is hidden when all conditions are true:
- `source_type == "ontology_note"`
- `payload.lifecycle.hidden != true`
- Effective TTL (`payload.note.ttl_expires_ms` OR fallback lookup from `note.ttl_expires`) is `<= cutoff_ms`

Fallback behavior for legacy rows:
- If `payload.note.ttl_expires_ms` is missing, command batches `source_id` lookups to `note` table.
- Lookup uses normalized note IDs (`note:<id>` and `<id>` both supported).
- If no TTL is found in payload or note table, row is counted as `missing_ttl_unresolved` and not hidden.

Patch written:

```json
{
  "lifecycle": {
    "hidden": true,
    "hidden_reason": "ontology_ttl_expired",
    "hidden_at_ms": 1234567890
  }
}
```

## Notes

- The command uses `merge_payload`, so existing enrichment data remains intact.
- Feed/search visibility already excludes rows with `payload.lifecycle.hidden=true`.
