# SurrealDB v3 — Chat Hot-Path Benchmark (Live Docker DB)

Date: 2026-02-25T08:53:20Z
Compose: `compose.dev.yaml`
Endpoint: `ws://127.0.0.1:8000`
Server version (HTTP `/version`): `surrealdb-3.0.0`
Namespace/DB: `gotong_chat_probe/bench`
Status note: validates chat message send/read query shapes used by `POST /v1/chat/threads/:thread_id/messages/send` and `GET /v1/chat/threads/:thread_id/messages`.

## Benchmark Intent

Evaluate chat fast-path read patterns:

```sql
-- catch-up page after cursor
SELECT message_id, created_at
FROM chat_message
WHERE thread_id = $thread_id
  AND (created_at > $t OR (created_at = $t AND message_id > $message_id))
ORDER BY created_at ASC, message_id ASC
LIMIT 50

-- idempotent send lookup
SELECT message_id, request_id
FROM chat_message
WHERE thread_id = $thread_id AND request_id = $request_id
LIMIT 1

-- membership check
SELECT user_id, role
FROM chat_member
WHERE user_id = $actor_id AND thread_id = $thread_id AND left_at IS NONE
ORDER BY joined_at DESC
LIMIT 1

-- read cursor lookup
SELECT user_id, thread_id, message_id, read_at
FROM chat_read_cursor
WHERE user_id = $actor_id AND thread_id = $thread_id
LIMIT 1
```

This probe uses structurally equivalent benchmark tables and indexes:
- `probe_chat_message_bench`: `idx_probe_message_order(thread_id, created_at, message_id)`, `uniq_probe_message_request(thread_id, request_id)`
- `probe_chat_member_bench`: `idx_probe_member_lookup(user_id, thread_id)`
- `probe_chat_cursor_bench`: `idx_probe_read_cursor_lookup(user_id, thread_id)`

Seed shape:
- total message rows: `60000`
- hot thread rows: `10000`
- global noise rows requested: `50000`
- catch-up cursor row: `2026-02-25T08:52:53.904652253Z` / `msg-hot-5000`

## Summary

| Scenario | IndexScan used | Index used | Index rows scanned | Index elapsed (ns) | Filter elapsed (ns) | SortTopK elapsed (ns) | Loop duration |
|---|---|---|---:|---:|---:|---:|---|
| Catch-up list | yes | uniq_probe_message_request | 10000 | 44260209 | 10644126 | 5261000 | 120 loops: 6s22ms386µs170ns |
| Idempotent send lookup | yes | uniq_probe_message_request | 1 | 53417 | n/a | n/a | 120 loops: 6ms643µs584ns |
| Member lookup | yes | idx_probe_member_lookup | 1 | 27332 | 3417 | 2668 | 120 loops: 7ms887µs84ns |
| Read cursor lookup | yes | uniq_probe_read_cursor_key | 1 | 31958 | n/a | n/a | 120 loops: 8ms963µs750ns |

## Decision

- Keep chat read/send path index-backed with `idx_message_order` + `uniq_message_request`.
- Keep explicit member and read-cursor lookup indexes (`idx_member_lookup`, `idx_read_cursor_lookup`) as baseline for chat authorization/read-state checks.
- Keep hot-thread catch-up window bounded (default/target `limit=50`) and benchmark again before raising catch-up limits.
- Keep graph/triple traversal out of chat request-path queries.

## Raw Explain (Catch-up)
```json
[{"attributes":{"projections":"message_id, created_at"},"children":[{"attributes":{"limit":"50"},"children":[{"attributes":{"limit":"50","sort_keys":"created_at ASC, message_id ASC"},"children":[{"attributes":{"predicate":"created_at > <datetime>  '2026-02-25T08:52:53.904652253Z' OR created_at = <datetime>  '2026-02-25T08:52:53.904652253Z' AND message_id > 'msg-hot-5000'"},"children":[{"attributes":{"access":"['thread-hot']","direction":"Forward","index":"uniq_probe_message_request"},"context":"Db","metrics":{"elapsed_ns":44260209,"output_batches":10,"output_rows":10000},"operator":"IndexScan"}],"context":"Db","expressions":[{"role":"predicate","sql":"created_at > <datetime>  '2026-02-25T08:52:53.904652253Z' OR created_at = <datetime>  '2026-02-25T08:52:53.904652253Z' AND message_id > 'msg-hot-5000'"}],"metrics":{"elapsed_ns":10644126,"output_batches":6,"output_rows":4999},"operator":"Filter"}],"context":"Db","metrics":{"elapsed_ns":5261000,"output_batches":1,"output_rows":50},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"50"}],"metrics":{"elapsed_ns":5286375,"output_batches":1,"output_rows":50},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":101208,"output_batches":1,"output_rows":50},"operator":"SelectProject","total_rows":50}]
```

## Raw Explain (Idempotent send lookup)
```json
[{"attributes":{"projections":"message_id, request_id"},"children":[{"attributes":{"access":"['thread-hot', 'req-hot-5000']","direction":"Forward","index":"uniq_probe_message_request","limit":"1"},"context":"Db","metrics":{"elapsed_ns":53417,"output_batches":1,"output_rows":1},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":2958,"output_batches":1,"output_rows":1},"operator":"SelectProject","total_rows":1}]
```

## Raw Explain (Member lookup)
```json
[{"attributes":{"projections":"user_id, role, joined_at"},"children":[{"attributes":{"limit":"1"},"children":[{"attributes":{"limit":"1","sort_keys":"joined_at DESC"},"children":[{"attributes":{"predicate":"left_at = NONE"},"children":[{"attributes":{"access":"['u-hot', 'thread-hot']","direction":"Forward","index":"idx_probe_member_lookup"},"context":"Db","metrics":{"elapsed_ns":27332,"output_batches":1,"output_rows":1},"operator":"IndexScan"}],"context":"Db","expressions":[{"role":"predicate","sql":"left_at = NONE"}],"metrics":{"elapsed_ns":3417,"output_batches":1,"output_rows":1},"operator":"Filter"}],"context":"Db","metrics":{"elapsed_ns":2668,"output_batches":1,"output_rows":1},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"1"}],"metrics":{"elapsed_ns":5667,"output_batches":1,"output_rows":1},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":2624,"output_batches":1,"output_rows":1},"operator":"SelectProject","total_rows":1}]
```

## Raw Explain (Read cursor lookup)
```json
[{"attributes":{"projections":"user_id, thread_id, message_id, read_at"},"children":[{"attributes":{"access":"['thread-hot', 'u-hot']","direction":"Forward","index":"uniq_probe_read_cursor_key","limit":"1"},"context":"Db","metrics":{"elapsed_ns":31958,"output_batches":1,"output_rows":1},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":2999,"output_batches":1,"output_rows":1},"operator":"SelectProject","total_rows":1}]
```
