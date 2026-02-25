# SurrealDB v3 — Feed Source Lookup Benchmark (Live Docker DB)

Date: 2026-02-25T05:09:49Z
Compose: `compose.dev.yaml`
Endpoint: `ws://127.0.0.1:8000`
Server version (HTTP `/version`): `surrealdb-3.0.0`
Namespace/DB: `gotong_feed_index_probe/bench`

## Benchmark Intent

Evaluate the hot-path lookup used by `get_latest_by_source`:

```sql
SELECT feed_id, occurred_at
FROM discovery_feed_item
WHERE source_type = $source_type AND source_id = $source_id
ORDER BY occurred_at DESC, feed_id DESC
LIMIT 1
```

This probe uses a structurally equivalent table and index:
- Table: `probe_feed_index_bench`
- Index: `(source_type, source_id, occurred_at, feed_id)`
- Seed shape: `20000` rows total, `10000` hot-source rows + `10000` distributed cold-source rows

## Summary

| Scenario | IndexScan used | Index direction | Index rows scanned | Index elapsed (ns) | SortTopK elapsed (ns) | Loop duration |
|---|---|---|---:|---:|---:|---|
| DESC with index | yes | Forward | 10000 | 55422543 | 11564876 | 120 loops: 5s671ms223µs628ns |
| ASC with index | yes | Forward | 10000 | 33385628 | 6423041 | 120 loops: 4s798ms62µs669ns |
| DESC without index | no | n/a | n/a | n/a | n/a | 30 loops: 1s809ms443µs834ns |

## Decision

- Keep the current index shape (`source_type, source_id, occurred_at, feed_id`).
- No DESC-specific index action now. In SurrealDB v3.0.0, this index is used for both ASC and DESC query shapes and planner reports the same `IndexScan ... direction: Forward` plus `SortTopKByKey`.
- Revisit only if production latency shows regression under higher per-source fan-out than this probe.

## Raw Explain (DESC with index)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"1"},"children":[{"attributes":{"limit":"1","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"access":"['ontology_note', 'hot-note']","direction":"Forward","index":"idx_probe_feed_source_latest"},"context":"Db","metrics":{"elapsed_ns":55422543,"output_batches":10,"output_rows":10000},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":11564876,"output_batches":1,"output_rows":1},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"1"}],"metrics":{"elapsed_ns":11583667,"output_batches":1,"output_rows":1},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":11874,"output_batches":1,"output_rows":1},"operator":"SelectProject","total_rows":1}]
```

## Raw Explain (ASC with index)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"1"},"children":[{"attributes":{"limit":"1","sort_keys":"occurred_at ASC, feed_id ASC"},"children":[{"attributes":{"access":"['ontology_note', 'hot-note']","direction":"Forward","index":"idx_probe_feed_source_latest"},"context":"Db","metrics":{"elapsed_ns":33385628,"output_batches":10,"output_rows":10000},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":6423041,"output_batches":1,"output_rows":1},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"1"}],"metrics":{"elapsed_ns":6434500,"output_batches":1,"output_rows":1},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":4750,"output_batches":1,"output_rows":1},"operator":"SelectProject","total_rows":1}]
```

## Raw Explain (DESC without index)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"1"},"children":[{"attributes":{"limit":"1","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"direction":"Forward","predicate":"source_type = 'ontology_note' AND source_id = 'hot-note'","table":"probe_feed_index_bench"},"context":"Db","metrics":{"elapsed_ns":85429666,"output_batches":2,"output_rows":10000},"operator":"TableScan"}],"context":"Db","metrics":{"elapsed_ns":6984042,"output_batches":1,"output_rows":1},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"1"}],"metrics":{"elapsed_ns":6995875,"output_batches":1,"output_rows":1},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":9584,"output_batches":1,"output_rows":1},"operator":"SelectProject","total_rows":1}]
```
