# SurrealDB v3 — Notification Hot-Path Benchmark (Live Docker DB)

Date: 2026-02-25T08:10:32Z
Compose: `compose.dev.yaml`
Endpoint: `ws://127.0.0.1:8000`
Server version (HTTP `/version`): `surrealdb-3.0.0`
Namespace/DB: `gotong_notification_probe/bench`
Status note: validates list/unread-count query shapes used by `GET /v1/notifications` and `GET /v1/notifications/unread-count`.

## Benchmark Intent

Evaluate hot-path notification reads:

```sql
-- default list (include_read=false)
SELECT notification_id, created_at
FROM discovery_notification
WHERE user_id = $actor_id AND read_at IS NONE
ORDER BY created_at DESC, notification_id DESC
LIMIT 20

-- include_read=true list
SELECT notification_id, created_at
FROM discovery_notification
WHERE user_id = $actor_id
ORDER BY created_at DESC, notification_id DESC
LIMIT 20

-- unread count
SELECT count() AS unread_count
FROM discovery_notification
WHERE user_id = $actor_id AND read_at IS NONE
```

This probe uses a structurally equivalent table and indexes:
- Table: `probe_notification_bench`
- Indexes:
  - `idx_probe_notification_user(user_id, created_at, notification_id)`
  - `idx_probe_notification_unread(user_id, read_at, created_at, notification_id)`
- Seed shape:
  - total rows: `100000`
  - hot user rows: `20000`
  - hot user unread rows: `10000`
  - extra global noise rows requested: `80000`

## Summary

| Scenario | IndexScan used | Index used | Index rows scanned | Index elapsed (ns) | Filter elapsed (ns) | SortTopK elapsed (ns) | Loop duration |
|---|---|---|---:|---:|---:|---:|---|
| Unread list (include_read=false) | yes | idx_probe_notification_unread | 10000 | 37002457 | n/a | 29356417 | 120 loops: 6s112ms961µs86ns |
| Full list (include_read=true) | yes | idx_probe_notification_user | 20000 | 76461668 | n/a | 63992416 | 120 loops: 11s21ms480µs991ns |
| Unread count | yes | idx_probe_notification_unread | 10000 | 37127543 | n/a | n/a | 120 loops: 5s46ms168µs378ns |

## Decision

- Keep unread-list path anchored to `(user_id, read_at, created_at, notification_id)` index.
- Keep include-read list path anchored to `(user_id, created_at, notification_id)` index.
- Keep unread-count shape as simple index-backed aggregate; no graph/triple traversal involvement.
- Re-run this benchmark when notification fanout semantics change (new notification types, digest batching, or read-state model updates).

## Raw Explain (Unread list)
```json
[{"attributes":{"projections":"notification_id, created_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"created_at DESC, notification_id DESC"},"children":[{"attributes":{"access":"['u-hot', NONE]","direction":"Forward","index":"idx_probe_notification_unread"},"context":"Db","metrics":{"elapsed_ns":37002457,"output_batches":10,"output_rows":10000},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":29356417,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":29367833,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":14417,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Full list)
```json
[{"attributes":{"projections":"notification_id, created_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"created_at DESC, notification_id DESC"},"children":[{"attributes":{"access":"['u-hot']","direction":"Forward","index":"idx_probe_notification_user"},"context":"Db","metrics":{"elapsed_ns":76461668,"output_batches":20,"output_rows":20000},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":63992416,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":64004625,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":8291,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Unread count)
```json
[{"attributes":{"projections":"unread_count"},"children":[{"attributes":{"fields":"unread_count = count(...)"},"children":[{"attributes":{"access":"['u-hot', NONE]","direction":"Forward","index":"idx_probe_notification_unread"},"context":"Db","metrics":{"elapsed_ns":37127543,"output_batches":10,"output_rows":10000},"operator":"IndexScan"}],"context":"Db","expressions":[{"role":"unread_count","sql":"count(...)"}],"metrics":{"elapsed_ns":5267416,"output_batches":10,"output_rows":10000},"operator":"Compute"}],"context":"Db","metrics":{"elapsed_ns":4690334,"output_batches":10,"output_rows":10000},"operator":"SelectProject","total_rows":10000}]
```
