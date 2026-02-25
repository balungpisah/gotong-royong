# Feed Participant-Edge Backfill

Last updated: 2026-02-25

This runbook covers Pack C5 historical backfill for `feed_participant_edge`.

## Command

Run from repo root:

```bash
cargo run -p gotong-worker -- feed-participant-edge-backfill [flags]
```

or:

```bash
just feed-participant-edge-backfill [flags]
```

## Flags

- `--dry-run` — scan only, do not write edges
- `--page-size <n>` — batch size for feed scan (`default: 500`, `max: 10000`)
- `--progress-every <n>` — progress logging interval (`default: 500`)
- `--from-ms <epoch_ms>` — optional lower bound filter on `occurred_at_ms`
- `--to-ms <epoch_ms>` — optional upper bound filter on `occurred_at_ms`
- `--max-rows <n>` — optional stop limit for chunked runs

Validation rules:
- `--page-size >= 1`
- `--progress-every >= 1`
- `--max-rows >= 1`
- `--from-ms >= 0`, `--to-ms >= 0`, and `from-ms <= to-ms` when both are set

## Example rollout

1. Dry-run on a bounded slice:

```bash
just feed-participant-edge-backfill --dry-run --from-ms 1735603200000 --to-ms 1738281600000 --max-rows 5000
```

2. Execute same slice:

```bash
just feed-participant-edge-backfill --from-ms 1735603200000 --to-ms 1738281600000 --max-rows 5000
```

3. Full historical pass:

```bash
just feed-participant-edge-backfill --page-size 1000 --progress-every 1000
```

## Output counters

The command prints cumulative progress and final summary:
- `scanned`
- `candidate_edges`
- `upserted_items`
- `failed_items`

Because edge writes are idempotent (`uniq_feed_participant_edge`), reruns are safe.
