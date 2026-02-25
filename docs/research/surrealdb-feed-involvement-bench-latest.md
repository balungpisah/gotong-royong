# SurrealDB v3 — Feed Involvement-Only Benchmark (Live Docker DB)

Date: 2026-02-25T05:15:29Z
Compose: `compose.dev.yaml`
Endpoint: `ws://127.0.0.1:8000`
Server version (HTTP `/version`): `surrealdb-3.0.0`
Namespace/DB: `gotong_feed_involvement_probe/bench`
Status note: this captures the **pre-cutover OR-path baseline**. Pack C edge-first reads were activated later on 2026-02-25 and should be compared against this baseline.

## Benchmark Intent

Evaluate the hot-path feed query shape behind `involvement_only=true`:

```sql
SELECT feed_id, occurred_at
FROM discovery_feed_item
WHERE actor_id = $actor_id OR $actor_id IN participant_ids
ORDER BY occurred_at DESC, feed_id DESC
LIMIT 20
```

This probe uses a structurally equivalent table and index setup:
- Table: `probe_feed_involvement_bench`
- Indexes: `idx_probe_actor_latest(actor_id, occurred_at, feed_id)`, `idx_probe_time(occurred_at, feed_id)`
- Seed shape: `20000` rows total (participant-heavy + actor-heavy mix)

## Summary

| Scenario | IndexScan used | Index used | Index rows scanned | Index elapsed (ns) | Filter elapsed (ns) | SortTopK elapsed (ns) | Loop duration |
|---|---|---|---:|---:|---:|---:|---|
| Actor-only (`actor_id = $actor_id`) | yes | idx_probe_actor_latest | 10000 | 36341793 | n/a | 36706042 | 120 loops: 5s283ms119µs336ns |
| Participant-only (`$actor_id IN participant_ids`) | yes | idx_probe_time | 20000 | 51607874 | 3518498 | 34419332 | 120 loops: 6s857ms86µs656ns |
| Combined OR path | yes | idx_probe_time | 20000 | 60621668 | 3858291 | 70326125 | 120 loops: 8s672ms839µs754ns |

## Decision

- Treat this as the baseline for the legacy OR-path.
- Pack C materialization (`feed_participant_edge`) is now activated for `involvement_only` with fallback; re-run edge-lane benchmark for post-cutover comparison.
- Keep triples/relations as enrichment/audit only; do not introduce graph traversal for feed listing.

## Raw Explain (Actor-only)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"access":"['u-hot']","direction":"Forward","index":"idx_probe_actor_latest"},"context":"Db","metrics":{"elapsed_ns":36341793,"output_batches":10,"output_rows":10000},"operator":"IndexScan"}],"context":"Db","metrics":{"elapsed_ns":36706042,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":36729499,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":22333,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Participant-only)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"predicate":"'u-hot' INSIDE participant_ids"},"children":[{"attributes":{"access":"","direction":"Forward","index":"idx_probe_time"},"context":"Db","metrics":{"elapsed_ns":51607874,"output_batches":20,"output_rows":20000},"operator":"IndexScan"}],"context":"Db","expressions":[{"role":"predicate","sql":"'u-hot' INSIDE participant_ids"}],"metrics":{"elapsed_ns":3518498,"output_batches":18,"output_rows":10000},"operator":"Filter"}],"context":"Db","metrics":{"elapsed_ns":34419332,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":34435875,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":11791,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```

## Raw Explain (Combined OR path)
```json
[{"attributes":{"projections":"feed_id, occurred_at"},"children":[{"attributes":{"limit":"20"},"children":[{"attributes":{"limit":"20","sort_keys":"occurred_at DESC, feed_id DESC"},"children":[{"attributes":{"predicate":"actor_id = 'u-hot' OR 'u-hot' INSIDE participant_ids"},"children":[{"attributes":{"access":"","direction":"Forward","index":"idx_probe_time"},"context":"Db","metrics":{"elapsed_ns":60621668,"output_batches":20,"output_rows":20000},"operator":"IndexScan"}],"context":"Db","expressions":[{"role":"predicate","sql":"actor_id = 'u-hot' OR 'u-hot' INSIDE participant_ids"}],"metrics":{"elapsed_ns":3858291,"output_batches":20,"output_rows":20000},"operator":"Filter"}],"context":"Db","metrics":{"elapsed_ns":70326125,"output_batches":1,"output_rows":20},"operator":"SortTopKByKey"}],"context":"Db","expressions":[{"role":"limit","sql":"20"}],"metrics":{"elapsed_ns":70343252,"output_batches":1,"output_rows":20},"operator":"Limit"}],"context":"Db","metrics":{"elapsed_ns":15084,"output_batches":1,"output_rows":20},"operator":"SelectProject","total_rows":20}]
```
