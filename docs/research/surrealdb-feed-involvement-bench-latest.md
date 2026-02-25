# SurrealDB v3 — Feed Involvement-Only Benchmark (Live Docker DB)

Date: 2026-02-25T09:13:27Z
Compose: `compose.dev.yaml`
Endpoint: `ws://127.0.0.1:8000`
Server version (HTTP `/version`): `surrealdb-3.0.0`
Namespace/DB: `gotong_feed_involvement_probe/bench`
Status note: compares the legacy OR-lane query against a Pack C-style materialized participant-edge lane.

## Benchmark Intent

Evaluate `involvement_only=true` query shapes:

```sql
-- legacy lane
SELECT feed_id, occurred_at
FROM discovery_feed_item
WHERE actor_id = $actor_id OR $actor_id IN participant_ids
ORDER BY occurred_at DESC, feed_id DESC
LIMIT 20

-- materialized edge lane
SELECT feed_id, occurred_at
FROM feed_participant_edge
WHERE actor_id = $actor_id
ORDER BY occurred_at DESC, feed_id DESC
LIMIT 20
```

This probe uses two structurally equivalent read models:
- Legacy table: `probe_feed_involvement_bench`
  - Indexes: `idx_probe_actor_latest(actor_id, occurred_at, feed_id)`, `idx_probe_time(occurred_at, feed_id)`
- Materialized table: `probe_feed_involvement_edge`
  - Indexes: `uniq_probe_edge_actor_feed(actor_id, feed_id)`, `idx_probe_edge_actor_latest(actor_id, occurred_at, feed_id)`
- Seed shape:
  - legacy rows: `100000`
  - edge rows: `210000`
  - hot actor edge rows: `20000`
  - extra non-hot noise rows requested: `80000`

## Summary

| Scenario | IndexScan used | Index used | Index rows scanned | Index elapsed (ns) | Filter elapsed (ns) | SortTopK elapsed (ns) | Loop duration |
|---|---|---|---:|---:|---:|---:|---|
| Actor-only (`actor_id = $actor_id`) | yes | idx_probe_actor_latest | 10000 | 97517123 | n/a | 38722166 | 120 loops: 8s42ms203µs45ns |
| Participant-only (`$actor_id IN participant_ids`) | yes | idx_probe_time | 100000 | 675263374 | 26449458 | 39383710 | 120 loops: 1m9s536ms474µs712ns |
| Combined OR path | yes | idx_probe_time | 100000 | 503102208 | 37648210 | 75795417 | 120 loops: 1m17s601ms579µs953ns |
| Materialized edge lane | yes | uniq_probe_edge_actor_feed | 20000 | 98288003 | n/a | 12537251 | 120 loops: 22s884ms90µs330ns |

## Decision

- Legacy OR and participant-membership queries remain filter-heavy on global time-order scans.
- Materialized edge lane keeps lookup keyed by actor and removes array-membership filtering from the hot path.
- Use this benchmark as C5 stabilization evidence while fallback remains enabled; remove fallback only after sustained low mismatch + SLO pass in target environments.
- Keep triples/relations as enrichment/audit only; do not introduce graph traversal for feed listing.

## Raw Explain (Actor-only)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"access":"['u-hot']","direction":"Forward","index":"idx_probe_actor_latest"},"context":"Db","metrics":{"elapsed_ns":97517123,"output_batches":10,"output_rows":10000},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":38722166,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":38746501,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":33376,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Participant-only)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"predicate":"'u-hot' INSIDE participant_ids"},"children":[{"attributes":{"access":"","direction":"Forward","index":"idx_probe_time"},"context":"Db","metrics":{"elapsed_ns":675263374,"output_batches":100,"output_rows":100000},"operator":"IndexScan"}],"context":"Db","expressions":[{"role":"predicate","sql":"'u-hot' INSIDE participant_ids"}],"metrics":{"elapsed_ns":26449458,"output_batches":16,"output_rows":10000},"operator":"Filter"}],"context":"Db","metrics":{"elapsed_ns":39383710,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":39407167,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":14749,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Combined OR path)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"predicate":"actor_id = 'u-hot' OR 'u-hot' INSIDE participant_ids"},"children":[{"attributes":{"access":"","direction":"Forward","index":"idx_probe_time"},"context":"Db","metrics":{"elapsed_ns":503102208,"output_batches":100,"output_rows":100000},"operator":"IndexScan"}],"context":"Db","expressions":[{"role":"predicate","sql":"actor_id = 'u-hot' OR 'u-hot' INSIDE participant_ids"}],"metrics":{"elapsed_ns":37648210,"output_batches":20,"output_rows":20000},"operator":"Filter"}],"context":"Db","metrics":{"elapsed_ns":75795417,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":75826919,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":10666,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Materialized edge lane)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"access":"['u-hot']","direction":"Forward","index":"uniq_probe_edge_actor_feed"},"context":"Db","metrics":{"elapsed_ns":98288003,"output_batches":20,"output_rows":20000},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":12537251,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":12553919,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":13668,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```
