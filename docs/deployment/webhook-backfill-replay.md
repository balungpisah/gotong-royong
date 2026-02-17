# Webhook Backfill and DLQ Replay

Last updated: 2026-02-17

This runbook defines the operator tooling for historical webhook backfill and dead-letter replay.

## Commands

The worker binary now supports two one-shot command modes:

1. `webhook-backfill`
2. `webhook-replay-dlq`

They can be run with `cargo run -p gotong-worker -- <command> ...` or via the built worker binary.

## 1) Backfill historical events

Input format:
- Newline-delimited JSON (`.ndjson`)
- One event payload per line
- Each payload must include `event_id`, `event_type`, `request_id`, `schema_version`, and `actor.user_id`

Deterministic dedupe:
- Outbox insert conflicts on duplicate `event_id`
- Duplicate `event_id` lines are counted as duplicates and skipped

Dry-run:

```bash
cargo run -p gotong-worker -- webhook-backfill \
  --file ./tmp/gotong-backfill.ndjson \
  --dry-run \
  --progress-every 200
```

Execute:

```bash
cargo run -p gotong-worker -- webhook-backfill \
  --file ./tmp/gotong-backfill.ndjson \
  --progress-every 200 \
  --max-attempts 5
```

Output includes running progress and final summary:
- `processed`
- `created`
- `enqueued`
- `duplicates`
- `failed`

## 2) Replay DLQ events

Default behavior replays `dead_letter` events.

Dry-run:

```bash
cargo run -p gotong-worker -- webhook-replay-dlq \
  --dry-run \
  --limit 1000 \
  --progress-every 100
```

Execute:

```bash
cargo run -p gotong-worker -- webhook-replay-dlq \
  --limit 1000 \
  --progress-every 100
```

Execution behavior:
- Select events by outbox status (default: `dead_letter`)
- Reset selected records to `pending`
- Reset attempts/error fields
- Enqueue new `webhook_retry` jobs

This provides deterministic DLQ drain/replay without mutating event payload identity.

## 3) Operational sequence

1. Run `webhook-backfill` in `--dry-run` mode.
2. Execute `webhook-backfill`.
3. Verify outbox/deployment metrics.
4. Run `webhook-replay-dlq --dry-run`.
5. Execute `webhook-replay-dlq`.
6. Confirm dead-letter volume returns to baseline.
