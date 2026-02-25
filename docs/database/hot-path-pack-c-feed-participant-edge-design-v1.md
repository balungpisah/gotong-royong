# Pack C Design — Feed Participant Edge Read Model (SurrealDB v3)

Status: active rollout (C1 schema + C2 dual-write + C3 shadow-on-fallback + C4 switch-with-fallback applied; C5 in progress)  
Last updated: 2026-02-25

## 1) Why Pack C exists

`involvement_only=true` currently relies on:

```sql
WHERE actor_id = $actor_id OR $actor_id IN participant_ids
```

on `discovery_feed_item` in `SurrealDiscoveryFeedRepository::list_feed` (`crates/infra/src/repositories/impls.rs`).

Latest live benchmark (`docs/research/surrealdb-feed-involvement-bench-latest.md`) shows:
- actor-only path is index-backed (`idx_probe_actor_latest`);
- participant and OR paths scan by time index then filter (`idx_probe_time` + `Filter` + `SortTopKByKey`), which is the scaling risk.

Pack C introduces a materialized participant lane to keep `involvement_only` reads index-first.

## 2) Scope and non-goals

In scope:
- A new read-model table for participant membership lookup.
- Feed read query rewrite for `involvement_only=true`.
- Feed write-path fanout into the new read-model.
- Backfill + staged rollout plan.

Out of scope:
- Feed ranking redesign.
- Search relevance/FTS redesign.
- Any graph traversal changes for hot-path reads.

## 3) Proposed schema

New table: `feed_participant_edge`

Proposed fields:
- `edge_id: string` (deterministic key: `${actor_id}:${feed_id}`)
- `actor_id: string` (the user for involvement lookup)
- `feed_id: string` (FK-like pointer to `discovery_feed_item.feed_id`)
- `occurred_at: datetime` (copied from feed item for cursor ordering)
- `scope_id: option<string>` (copied for scope filtering)
- `privacy_level: option<string>` (copied for quick filter)
- `source_type: string` (debug/ops aid)
- `source_id: string` (debug/ops aid)
- `created_at: datetime` (edge write time)
- `request_id: string` (origin feed request id for traceability)

Proposed indexes:
- `uniq_feed_participant_edge` on `(actor_id, feed_id)` UNIQUE
- `idx_feed_participant_actor_latest` on `(actor_id, occurred_at, feed_id)`
- `idx_feed_participant_actor_scope_latest` on `(actor_id, scope_id, occurred_at, feed_id)`
- Optional (only if needed): `idx_feed_participant_actor_privacy_latest` on `(actor_id, privacy_level, occurred_at, feed_id)`

Proposed permissions (`0019` style):
- `FOR select WHERE actor_id = string::split(type::string($auth.id), ':')[1] OR $auth.platform_role IN ["admin","moderator"]`
- `FOR create/update/delete NONE` for record sessions (service/root writes only).

## 4) Query rewrite (read path)

### 4.1 `GET /v1/feed` with `involvement_only=false`

No change; keep current `discovery_feed_item` query path.

### 4.2 `GET /v1/feed` with `involvement_only=true`

Step A: fetch candidate IDs from edge table (cursor-compatible):

```sql
SELECT feed_id, occurred_at
FROM feed_participant_edge
WHERE actor_id = $actor_id
  -- optional: scope/privacy/time/cursor predicates
ORDER BY occurred_at DESC, feed_id DESC
LIMIT $limit
```

Step B: hydrate feed rows from `discovery_feed_item` by `feed_id`.

Step C: re-order in application by Step A order, then apply existing visibility rules (`is_visible_to_actor`) as defense-in-depth.

### 4.3 `GET /v1/search` with `involvement_only=true`

Same pattern: edge-first candidate narrowing, then hydrate feed rows and run existing text matching/scoring logic.

This avoids global-table OR + array-membership scans during involvement search.

## 5) Write-path changes (ingest)

Current feed ingest flow:
- `DiscoveryService::ingest_feed` creates one `discovery_feed_item` (idempotent via `uniq_feed_source_request`).

Pack C write model:
- On each successful feed ingest, also upsert edges for:
  - `actor_id` (always)
  - every `participant_id` (deduped)

Effective participant set:
- `involvement_actor_ids = dedupe([actor_id] + participant_ids)`

Idempotency strategy:
- deterministic `edge_id` (`actor_id:feed_id`) and/or unique `(actor_id, feed_id)`.
- use upsert semantics to make retries safe.

Important edge-case:
- On ingest replay (`DomainError::Conflict` → `get_by_source_request`), ensure edge upsert still runs so old partial writes can self-heal.

## 6) Backfill and rollout plan

### Phase C1 — Schema only
- Add migration/check pair for `feed_participant_edge`.
- Keep read path unchanged.
- ✅ Implemented:
  - `database/migrations/0027_hot_path_pack_c_feed_participant_edge.surql`
  - `database/checks/0027_hot_path_pack_c_feed_participant_edge_check.surql`

### Phase C2 — Dual write
- Feed ingest writes both `discovery_feed_item` and `feed_participant_edge`.
- Add backfill worker command for historical feed items.
- ✅ Implemented:
  - `DiscoveryService::ingest_feed` now upserts participant edges for both create and replay-conflict paths.
  - `FeedRepository::upsert_participant_edges_for_item` implemented in:
    - `SurrealDiscoveryFeedRepository`
    - `InMemoryDiscoveryFeedRepository`

### Phase C3 — Dual read shadow
- For sampled requests, run both:
  - legacy OR query path
  - edge-first path
- Compare ordered `feed_id` sets and emit mismatch metrics.
- ✅ Implemented (lightweight shadow on fallback):
  - `SurrealDiscoveryFeedRepository` compares edge-vs-legacy prefix ordering when fallback executes.
  - Divergence is surfaced via `tracing::warn!` for rollout monitoring.

### Phase C4 — Switch with fallback
- Primary path: edge-first for `involvement_only=true`.
- Fallback to legacy OR path when edge coverage is missing/incomplete.
- ✅ Implemented:
  - `list_feed` and `search_feed` now route `involvement_only=true` through `feed_participant_edge` first.
  - If edge hydration returns fewer rows than requested, repository runs legacy query with a wider limit and merges/dedupes by `feed_id`.

### Phase C5 — Stabilize
- Remove fallback after sustained SLO + correctness pass.
- ✅ Implemented:
  - historical backfill command: `feed-participant-edge-backfill` (`crates/worker/src/main.rs`)
  - explicit counters:
    - `gotong_api_feed_involvement_lane_requests_total{endpoint,lane}`
    - `gotong_api_feed_involvement_shadow_mismatch_total{endpoint,mismatch}`
- ⏳ Remaining:
  - run/verify full historical backfill in target environments
  - remove fallback after sustained correctness + SLO window

## 7) Operational checks

SLO guard (unchanged target from canonical matrix):
- `GET /v1/feed` p95 `<= 180ms` at `limit=20`.

Add metrics:
- `gotong_api_feed_involvement_lane_requests_total{endpoint,lane}`
- `gotong_api_feed_involvement_shadow_mismatch_total{endpoint,mismatch}`
- `feed_participant_edge_backfill_progress_ratio` (optional derived metric from backfill runs)

Add verification scripts:
- ✅ `just feed-involvement-bench-surreal` now benchmarks both legacy OR lane and materialized edge lane in one run.
- Default benchmark profile includes `NOISE_ROWS=80000` to stress worst-case global-table scans; override as needed for lighter local checks.
- Keep historical pre-cutover baseline notes in `docs/research/surrealdb-feed-involvement-bench-latest.md` history for before/after comparison.

## 8) Risks and mitigations

Risk: stale edge rows if feed metadata changes later.  
Mitigation: feed metadata is mostly immutable; visibility still enforced on hydrated feed rows.

Risk: partial writes if feed item write succeeds but edge write fails.  
Mitigation: dual-read fallback + replay-safe upsert + backfill repair job.

Risk: complexity creep in repository layer.  
Mitigation: keep Pack C isolated to `involvement_only` branch; non-involvement path remains unchanged.

## 9) Implementation checklist

1. ✅ Migration + check files for `feed_participant_edge`.
2. ✅ Repository query branch for involvement-only edge-first read.
3. ✅ Feed ingest dual-write for participant edges.
4. ✅ Backfill command in worker (`feed-participant-edge-backfill`).
5. ✅ Shadow-read comparison and metrics (warning logs + explicit counters).
6. ✅ Cutover with fallback guard.
