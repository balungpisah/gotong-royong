# SurrealDB v3 — Live Docker DB Probe (Gotong)

Date: 2026-02-25T05:09:25Z
Compose: `compose.dev.yaml`
Endpoint: `ws://127.0.0.1:8000`
Server version (HTTP `/version`): `surrealdb-3.0.0`
Namespace/DB: `gotong_probe/bench`

## Result Summary

| Pattern | Result | Notes |
|---|---|---|
| Unique composite index (idempotency) | PASS | Duplicate insert blocked; count stays 1 |
| Deterministic ordering + catch-up cursor | PASS | Stable `(created_at,message_id)` cursor logic |
| `EXPLAIN` shows index scan | PASS | Verifies index-backed filtering; order still uses TopK sort |
| `LIVE SELECT` over WS | PASS | WS streams `CREATE` actions |
| `LIVE SELECT` over HTTP fails (expected) | PASS | Confirms WS-only realtime in this env |
| `LIVE SELECT DIFF` payload | PASS | Includes `op: 'change'` for field updates |
| Permission-filtered live subscription | PASS | Alice sees only her rows |
| Relation traversal mechanics | PASS | 1-hop traversal returns nested objects |
| Full-text index + `@1@` query | PASS | Uses `FULLTEXT` + `search::score(1)` |
| Vector KNN + HNSW index | PASS | Uses `<|k,ef|>` + `vector::distance::knn()` |

## Key Outputs (selected)

### Unique composite index (duplicate error)
```json
["Database index `uniq_entity_request` already contains ['thread:alpha', 'req-123'], with record `probe_idem:uh6rdcna0e39h1q4mtiw`"]
```

### Ordering query output
```json
[[{"created_at":"2026-02-15T03:30:00Z","message_id":"msg-001"},{"created_at":"2026-02-15T03:30:00Z","message_id":"msg-002"},{"created_at":"2026-02-15T03:30:00Z","message_id":"msg-003"}]]
```

### Catch-up query output
```json
[[{"created_at":"2026-02-15T03:30:00Z","message_id":"msg-002"},{"created_at":"2026-02-15T03:30:00Z","message_id":"msg-003"}]]
```

### EXPLAIN plan
```json
["SelectProject [ctx: Db] [projections: message_id, created_at]\n    Limit [ctx: Db] [limit: 20]\n        SortTopKByKey [ctx: Db] [sort_keys: created_at ASC, message_id ASC, limit: 20]\n            IndexScan [ctx: Db] [index: idx_probe_ts_order, access: ['thread:order'], direction: Forward]\n"]
```

### ORDER BY projection requirement (error excerpt)
```text
 --< Parse error: Missing order idiom `created_at` in statement selection
 --> [1:73]
  |
1 | ... ORDER BY created_at ASC, message_id ASC;
  |              ^^^^^^^^^^^^^^
 --> [1:8]
```

### LIVE SELECT over WS (excerpt)
```text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'5993d42e-5fed-43e4-89ca-d5ca27c56348']


{ action: 'CREATE', id: u'5993d42e-5fed-43e4-89ca-d5ca27c56348', result: { body: 'hello-live', created_at: d'2026-02-25T05:09:08.925019427Z', id: probe_live:agg1z5flkhlpan08nmsd, thread_id: 't-live' } }
```

### LIVE SELECT over HTTP (excerpt)
```text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
Unable to perform the realtime query
```

### LIVE SELECT DIFF (excerpt)
```text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'701febb9-ba5e-443b-b9c7-837e570c7cb4']


{ action: 'CREATE', id: u'701febb9-ba5e-443b-b9c7-837e570c7cb4', result: [{ op: 'replace', path: '', value: { body: 'hello', id: probe_diff:fyrzb7yqjqf3ziiz6w0b, thread_id: 't-diff' } }] }


{ action: 'UPDATE', id: u'701febb9-ba5e-443b-b9c7-837e570c7cb4', result: [{ op: 'change', path: '/body', value: '@@ -1,5 +1,12 @@\n hello\n+ edited\n' }] }
```

### Permission-filtered LIVE SELECT (excerpt)
```text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'f2f19eb6-ab71-4f31-8af7-1090b5452bff']


{ action: 'CREATE', id: u'f2f19eb6-ab71-4f31-8af7-1090b5452bff', result: { body: 'alice private', id: probe_private:bngzxcfhq0oe29r1o6ym, owner: probe_user:1j6y5i54yytqwc8d0xxt } }
```

### Graph traversal result
```json
[[{"->probe_follows":{"->probe_graph_entity":[{"id":"probe_graph_entity:rt05","label":"RT 05"}]}}]]
```

### Full-text search result
```json
[[{"id":"probe_search:1","score":0.5108256340026855}]]
```

### Vector search result
```json
[[{"dist":0.0034970067397347426,"id":"probe_vector:1"},{"dist":0.05639380312223785,"id":"probe_vector:3"}]]
```

### Vector EXPLAIN FULL (note: may not show vector index operator yet)
```json
[{"attributes":{"projections":"id"},"children":[{"attributes":{"dimension":"3","ef":"100","index":"idx_probe_vector_embedding","k":"2"},"context":"Db","metrics":{"elapsed_ns":85584,"output_batches":1,"output_rows":2},"operator":"KnnScan"}],"context":"Db","metrics":{"elapsed_ns":98000,"output_batches":1,"output_rows":2},"operator":"SelectProject","total_rows":2}]
```
