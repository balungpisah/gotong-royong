# SurrealDB v3 — Feed Involvement-Only Benchmark (Live Docker DB)

Date: 2026-02-25T08:02:04Z
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
| Actor-only (`actor_id = $actor_id`) | yes | idx_probe_actor_latest | 10000 | 57943543 | n/a | 39664125 | 120 loops: 6s879ms587µs211ns |
| Participant-only (`$actor_id IN participant_ids`) | yes | idx_probe_time | 100000 | 565024374 | 27206169 | 35426127 | 120 loops: 59s803ms220µs875ns |
| Combined OR path | yes | idx_probe_time | 100000 | 466689294 | 37294995 | 79735750 | 120 loops: 1m57s191ms800µs957ns |
| Materialized edge lane | yes | uniq_probe_edge_actor_feed | 20000 | 115728498 | n/a | 11732875 | 120 loops: 15s818ms367µs77ns |

## Decision

- Legacy OR and participant-membership queries remain filter-heavy on global time-order scans.
- Materialized edge lane keeps lookup keyed by actor and removes array-membership filtering from the hot path.
- Use this benchmark as C5 stabilization evidence while fallback remains enabled; remove fallback only after sustained low mismatch + SLO pass in target environments.
- Keep triples/relations as enrichment/audit only; do not introduce graph traversal for feed listing.

## Raw Explain (Actor-only)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"access":"['u-hot']","direction":"Forward","index":"idx_probe_actor_latest"},"context":"Db","metrics":{"elapsed_ns":57943543,"output_batches":10,"output_rows":10000},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":39664125,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":39728250,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":21834,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Participant-only)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"predicate":"'u-hot' INSIDE participant_ids"},"children":[{"attributes":{"access":"","direction":"Forward","index":"idx_probe_time"},"context":"Db","metrics":{"elapsed_ns":565024374,"output_batches":100,"output_rows":100000},"operator":"IndexScan"}],"context":"Db","expressions":[{"role":"predicate","sql":"'u-hot' INSIDE participant_ids"}],"metrics":{"elapsed_ns":27206169,"output_batches":14,"output_rows":10000},"operator":"Filter"}],"context":"Db","metrics":{"elapsed_ns":35426127,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":35441419,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":12001,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Combined OR path)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"predicate":"actor_id = 'u-hot' OR 'u-hot' INSIDE participant_ids"},"children":[{"attributes":{"access":"","direction":"Forward","index":"idx_probe_time"},"context":"Db","metrics":{"elapsed_ns":466689294,"output_batches":100,"output_rows":100000},"operator":"IndexScan"}],"context":"Db","expressions":[{"role":"predicate","sql":"actor_id = 'u-hot' OR 'u-hot' INSIDE participant_ids"}],"metrics":{"elapsed_ns":37294995,"output_batches":20,"output_rows":20000},"operator":"Filter"}],"context":"Db","metrics":{"elapsed_ns":79735750,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":79767455,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":12709,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Materialized edge lane)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"access":"['u-hot']","direction":"Forward","index":"uniq_probe_edge_actor_feed"},"context":"Db","metrics":{"elapsed_ns":115728498,"output_batches":20,"output_rows":20000},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":11732875,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":11749875,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":13834,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```
