# SurrealDB v3 (Stable) — Feature Audit + Benchmark Matrix

Status: research + live benchmark evidence on v3.0.0 stable

Primary goal: make **chat** and **feed** predictably fast under worst-credible load, while keeping the data model robust (idempotent, auditable, permission-safe).

This document is the reference for:
- which SurrealDB v3 features we should leverage (and which we should avoid on hot paths),
- which behaviors must be verified in our environment,
- how to benchmark our exact query shapes before locking schema decisions.

---

## 0) Version alignment (current reality)

### What we are actually running

- SurrealDB Docker image observed running locally: `surrealdb/surrealdb:v3.0.0` (stable).
- Rust SDK pinned in workspace: `surrealdb = "=3.0.0"` and `surrealdb-types = "=3.0.0"` (see `Cargo.toml`).

### Docs mismatch to fix later

Several Gotong docs still reference `v3.0.0-beta.4` and beta-era probes. The intent of this doc is to re-audit on **stable v3** and define benchmarks; we can update the stack-lock ADR/docs after the benchmarks confirm parity.

---

## 1) Non-negotiable product constraints

### Chat must be fast

Chat read/write paths must be:
- append-only by default (edits are updates, but inserts dominate),
- index-friendly (thread-scoped range scans),
- reconnect-safe (deterministic ordering + catch-up query),
- realtime-ready (WS primary, SSE/poll fallback).

### Feed must be relevant and fast

Feed must:
- load quickly (avoid graph traversal and cross-table joins on the request path),
- filter by visibility/rahasiа/community/mode/location/domain quickly,
- be rankable without blocking request latency (async ranking pipeline).

---

## 2) Core stance: hybrid DB patterns

SurrealDB gives us both graph and document capabilities, but we should be explicit about where each belongs.

### Use SurrealDB “graph” for correctness, not for serving hot lists

Graph relations are useful for:
- provenance: who created / attested / challenged,
- permissions: membership edges,
- adaptive path structure: entity → phases → checkpoints,
- background ranking signals.

Avoid multi-hop traversals for:
- feed list rendering,
- chat message list rendering.

### Use materialized (denormalized) “read models” for hot paths

We should expect to keep separate read-optimized tables such as:
- `chat_message` (time series by thread),
- `feed_item` (time series by scope, with cached rank fields),
- `notification` (time series by user),
- `webhook_outbox_event` (time series by next_attempt_at/status),
- `idempotency_key` (lookup by key).

Graph edges can still exist in parallel for auditability and background computation.

---

## 3) SurrealDB v3 feature audit (what to leverage)

This section lists features we should actively consider using, plus known caveats.

### 3.1 Indexing and constraints

**Must-have**
- Composite indexes for time series range scans (e.g., `(thread_id, created_at, message_id)`).
- Unique composite indexes for idempotency (e.g., `(entity_id, request_id)`).

**Verified on v3.0.0 (stable)**
- Unique composite index reliably rejects duplicates at the DB level.

**Benchmark questions**
- Does `ORDER BY created_at, message_id` on a thread-scoped query stay index-friendly?
- What is the p95 cost of unique index enforcement under concurrent writes?

### 3.2 Query planning introspection

**Leverage**
- `EXPLAIN` for query plan inspection during benchmarking (catch accidental scans).

**Benchmark questions**
- Which indexes are actually used in our real query shapes?
- Do permission filters change the plan materially?

### 3.3 Realtime: `LIVE SELECT` + `LIVE SELECT DIFF`

**Leverage**
- SurrealDB live queries for internal server-side subscriptions (Gotong backend can consume and fan out to clients).
- `DIFF` payload is promising for efficient “message edited” style updates.

**Caveats**
- Live query streaming is WS-centric; do not assume HTTP streaming support.
- Permission-filtering on live streams must be treated as security-critical and benchmarked (CPU cost + correctness).

**Benchmark questions**
- Cost of 1 live query per thread + N subscribers (fanout model).
- Cost of 1 live query per user for notifications/unread counts.

### 3.4 Permissions (row/field-level)

**Leverage**
- Table + record permissions for “private thread” / “vault” leakage prevention.

**Caveats**
- Complex permission expressions can become invisible latency multipliers on hot paths.

**Benchmark questions**
- What is the overhead of the permission policy we actually need (not a toy)?
- Does `LIVE SELECT` permission filtering behave correctly under updates and joins/traversals?

### 3.5 Relations and traversal

**Leverage**
- `DEFINE TABLE <edge> TYPE RELATION` for structured edges.
- 1-hop traversals for entity detail pages (not feed lists).

**Caveats**
- Graph traversal ergonomics and return shapes must be validated per query pattern.
- Prefer “reverse walk” patterns where they’re more reliable, and use `.*` when full objects are needed.

**Benchmark questions**
- When do traversals return record IDs vs objects, and what’s the cost?
- Are `CONTAINS`/`IN` membership checks on traversal results reliable for our use?

### 3.6 Full-text search (FTS) via `FULLTEXT` indexes

**Leverage**
- `DEFINE ANALYZER ...`
- `DEFINE INDEX ... FIELDS <field> FULLTEXT ANALYZER <analyzer> BM25 ...`
- Query operator: `field @@ 'query'`

Use cases:
- feed search (Catatan Komunitas facts, witness notes),
- discovery/search within a community,
- quick moderation lookups.

**Caveats**
- FTS index maintenance has a real write cost; consider async index building (`HIGHLIGHTS`, `BM25`) and whether to `DEFER` indexing (if appropriate).

**Benchmark questions**
- How does FTS write overhead affect chat/message throughput if we index message bodies? (likely: do NOT FTS-index chat messages.)

### 3.7 Vector search (HNSW)

**Leverage**
- HNSW indexes for approximate nearest-neighbor search.

Use cases:
- Duplicate detection / near-duplicate clustering (AI-03).
- Semantic feed relevance (optional).

**Caveats**
- HNSW indexes are memory-heavy; must size and benchmark.
- Consider `DEFER` on vector indexes if ingestion throughput matters more than immediate searchability.

**Benchmark questions**
- KNN latency under realistic embedding dimensionality (e.g., 384/768/1536).
- Memory growth and rebuild behavior as the dataset scales.

### 3.8 Transactions

**Leverage**
- Transactions can be valid when issued as a single batch query over RPC/SDK.

**Caveats**
- The SQL shell can be misleading if it executes statements in a way that breaks transaction state across statements.

**Design stance**
- Do not rely on multi-record transactions as the only correctness mechanism.
- Prefer idempotent multi-statement writes + unique constraints + outbox patterns so retries are safe.

### 3.9 TTL / automatic expiry

**Observed**
- There is no supported `EXPIRE <duration>` clause in SurrealDB v3.0.0 (stable) as tested.

**Design stance**
- Use explicit expiry fields (`expires_at`) and cleanup jobs.
- Keep “hard delete vs tombstone” policy per domain (vault vs public notes vs chat).

---

## 4) Recommended modeling patterns for hot paths

These are not final schemas; they’re patterns we should benchmark.

### 4.1 Chat tables (time-series optimized)

**Primary query shapes**
- `List last N messages`: thread-scoped ordered range scan.
- `Catch-up`: “messages after (created_at, message_id) cursor”.
- `Insert message`: single insert + delivery/audit append.
- `Update read cursor`: upsert by `(thread_id, user_id)`.

**Indexes to benchmark**
- `(thread_id, created_at, message_id)` for message ordering.
- `(request_id, thread_id)` or `(request_id, thread_id, actor_id)` for idempotency on send.

**Realtime**
- Use backend WS fanout; Surreal live queries are an internal building block, not the public socket.

### 4.2 Feed tables (materialized read model)

Feed should read from a **materialized** `feed_item` table, not from graph traversals.

**Fields to include**
- `scope_id` (community/city/global),
- `mode` (Komunitas / Catatan Komunitas / Siaga / Vault-visibility constraints),
- `visibility` / `rahasia_level` / `ai_readable`,
- `domain_tags` / `skill_tags`,
- `location_bucket`,
- `created_at`,
- `rank_score` + `rank_band` (coarse).

**Query shape to benchmark**
- Filtered list by scope + mode + visibility + time window, order by `(rank_band desc, created_at desc, id desc)` with a bounded window.

**Ranking pipeline**
- Rank computation should be async; feed must degrade to time-sort if ranking is behind.

### 4.3 Event/outbox/idempotency (robustness)

**Outbox**
- `webhook_outbox_event` rows: `(status, next_attempt_at)` index.
- Worker: claim batches (with a claim token) and update attempt counters.

**Idempotency**
- durable `idempotency_key` rows with unique index on key.
- used for: chat send, evidence submit, webhook publish.

**Audit**
- append-only `audit_event` with actor/entity/time.

---

## 5) Benchmark matrix (what to measure before locking schema)

### 5.1 Worst-credible envelope (design target)

Use this envelope to generate test data and load profiles:
- 10k concurrent connected clients (Gotong WS layer)
- 5k req/s reads (feed + chat list dominate)
- 500 req/s writes (messages, cursors, feed events, moderation actions)
- evidence blobs stored in S3; DB only holds metadata + hashes

We should validate at smaller scales too, but this is the “don’t paint ourselves into a corner” target.

### 5.2 Bench table

| Area | Query / operation | Data shape | Dataset scale | Load profile | SLO target | Notes |
|---|---|---|---:|---:|---:|---|
| Chat | List last 50 messages | `chat_message` time series | 100k threads, avg 1k msgs/thread | 2k rps | p95 < 50ms | Must be index-backed; no traversals. |
| Chat | Catch-up after cursor | same | same | 1k rps | p95 < 50ms | Predicate uses `(created_at,message_id)` cursor. |
| Chat | Send message (idempotent) | insert + idempotency | same | 300 rps | p95 < 80ms | Unique index contention under concurrent writes. |
| Chat | Update read cursor | upsert cursor | same | 300 rps | p95 < 30ms | Should be single-row upsert. |
| Feed | Load feed page | `feed_item` | 10M items | 1k rps | p95 < 120ms | Filters + bounded sort strategy. |
| Feed | Apply ranking updates | update rank fields | 10M items | 200 rps | p95 < 80ms | Must not block feed reads. |
| Search | FTS query | FULLTEXT index | 10M docs | 200 rps | p95 < 150ms | Avoid indexing chat messages. |
| AI | Vector KNN (duplicate check) | HNSW index | 1M embeddings | 50 rps | p95 < 200ms | Measure memory; consider `DEFER`. |
| Realtime | Thread live fanout | live query → WS fanout | 1k hot threads | 10k clients | stability | no drops | Backend must throttle and shed load gracefully. |
| Permissions | Private thread read | permission filters | mixed | 200 rps | p95 < 80ms | Validate non-leakage + overhead. |

Targets are intentionally aggressive; adjust only after we measure.

---

## 6) Benchmark runbook (how to simulate)

This is a suggested procedure; we should automate it later.

### 6.1 Start SurrealDB v3 (stable) in Docker

Example (memory engine, local):
- Use `surrealdb/surrealdb:v3.0.0`
- Bind `:8000`
- Use root user/pass

### 6.2 Load schema + indexes

Load candidate schema variants:
1) Chat-only schema + indexes
2) Feed read model schema + indexes
3) Optional: FTS + vector indexes

### 6.3 Generate synthetic data

Generate records with realistic skew:
- A small set of “hot threads” with very high message volume.
- Long tail of cold threads.
- Feed items biased by community and mode.

### 6.4 Load test

Use a load generator which can:
- drive HTTP endpoints (Gotong API),
- drive WS endpoints (Gotong realtime),
- measure p50/p95/p99 and error rates,
- run fixed-duration tests + ramp-up.

### 6.5 Validate query plans

For each hot query shape:
- capture `EXPLAIN` output,
- ensure index scans are used,
- record any scans or unexpected traversal overhead.

---

## 7) Decision checkpoints (what we decide after benchmarks)

After the first benchmark pass, we should explicitly decide:

1) **Feed strategy**: can we serve fully sorted results in Surreal, or do we need bounded windows + app-side ranking?
2) **Realtime strategy**: do Surreal live queries scale for our fanout model, or do we use DB polling + backend fanout?
3) **Vector search**: keep embeddings in Surreal (HNSW) vs external vector store.
4) **Permission model**: how complex can the permission expressions be before they hurt p95?
5) **Engine choice**: TiKV vs other supported engines for staging/prod (and migration/backup strategy).

---

## Appendix A — quick probes already run on v3.0.0 stable (local)

These are sanity checks, not performance tests:
- Unique composite index enforcement: PASS.
- `EXPLAIN` query plans: available and usable.
- Transactions: reliable when issued as a single batched query (RPC/SDK); do not rely on SQL shell behavior alone.
- `EXPIRE <duration>` syntax: rejected (plan for manual TTL cleanup).

Reference outputs:
- Release-gate probe (isolated ephemeral DB): `docs/research/surrealdb-go-no-go-latest.md`
- Live Docker dev DB probe (`compose.dev.yaml` on `ws://127.0.0.1:8000`): `docs/research/surrealdb-live-db-probe-latest.md`
  - Re-run: `bash scripts/surrealdb-live-db-probe.sh`
- Chat hot-path benchmark (catch-up/idempotent/member/read-cursor): `docs/research/surrealdb-chat-bench-latest.md`
  - Re-run: `just chat-bench-surreal`
- Feed source lookup benchmark: `docs/research/surrealdb-feed-index-bench-latest.md`
  - Re-run: `just feed-index-bench-surreal`
- Feed involvement benchmark (`involvement_only` legacy OR vs materialized edge lane): `docs/research/surrealdb-feed-involvement-bench-latest.md`
  - Re-run: `just feed-involvement-bench-surreal`
- Notification hot-path benchmark (list unread/list all/unread count): `docs/research/surrealdb-notification-bench-latest.md`
  - Re-run: `just notification-bench-surreal`

---

## Appendix B — live Docker probe takeaways (v3.0.0 on `compose.dev.yaml`)

These are the “what we learned for product architecture” notes from the live probe:

- **Realtime is WS-only**: `LIVE SELECT` works over `ws://...` but fails over `http://...` in this environment. Backend must own fanout over our WS layer; don’t plan on HTTP streaming.
- **Chat/feed hot-path queries are index-backed, but still TopK-sort**: `EXPLAIN` shows `IndexScan` + `SortTopKByKey` for ordered timeline reads. Keep limits small and always benchmark “hot thread” skew.
- **Permission-filtered live streams work**: record access + table `PERMISSIONS` can prevent row leakage even on live subscriptions; treat permission expressions as part of the p95 budget.
- **DIFF stream payloads are viable**: `LIVE SELECT DIFF` emits compact patch-style updates (good fit for “message edited” / “status changed”).
- **Graph traversal works, but shouldn’t serve hot lists**: 1-hop relations return nested objects; use relations for correctness/audit/background computations, not for assembling feed pages.
- **FTS + vector KNN are available on v3.0.0**:
  - FULLTEXT queries via `@1@` + `search::score(1)` work.
  - HNSW + KNN queries can produce `KnnScan` in `EXPLAIN FULL` and return `vector::distance::knn()` distances.
  - Still treat both as “benchmark-required” before committing to indexing high-write tables (e.g., do **not** fulltext-index chat messages).
- **`involvement_only` was the scaling trigger**: actor-only feed lookups stay index-backed, while `participant_ids` membership predicates are filter-heavy; Pack C (`feed_participant_edge` materialization) is now active with legacy fallback while C5 stabilization completes (backfill command + lane/shadow counters shipped, fallback removal pending).
  - Pack C design reference: `docs/database/hot-path-pack-c-feed-participant-edge-design-v1.md`
