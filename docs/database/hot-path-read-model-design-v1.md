# Hot-Path Read-Model Design (SurrealDB v3)

Status: active baseline (Pack A + Pack B + Pack C1-C4 applied with fallback; C5 in progress)  
Last updated: 2026-02-25

Scope:
- Chat, feed, notifications, and webhook outbox lanes.
- Index and query-shape design only (no API contract changes).

Out of scope:
- Ontology graph redesign.
- Cross-platform identity/consent model changes.

## 1) Design Principles

- Read-model first: tables/indexes are shaped by request-path queries.
- Deterministic cursor ordering on tie-break keys.
- Idempotent writes via unique keys + stable `x-request-id`.
- Graph/triples used only for enrichment/audit side-paths.

## 2) Current Schema Coverage (What Already Works)

### Chat (`0001`, `0002`)
- `idx_message_order(thread_id, created_at, message_id)` supports catch-up/pagination.
- `uniq_message_request(request_id, thread_id)` supports idempotent send without biasing thread-only catch-up scans.
- `idx_member_lookup(user_id, thread_id)` supports membership checks.
- `idx_read_cursor_lookup(user_id, thread_id)` supports read-cursor fetch/update.

### Feed / Notifications (`0008`, `0023`, `0024`)
- Feed cursor: `idx_feed_time(occurred_at, feed_id)` and `idx_feed_scope(scope_id, occurred_at, feed_id)`.
- Feed idempotent ingest: `uniq_feed_source_request(source_type, source_id, request_id)`.
- Latest-by-source lane: `idx_feed_source_latest(source_type, source_id, occurred_at, feed_id)`.
- Notification cursor: `idx_notification_user(user_id, created_at, notification_id)`.
- Notification idempotent ingest: `uniq_notification_dedupe(user_id, dedupe_key)`.
- Flexible payload objects for nested lifecycle/enrichment patches (`0024`).

### Outbox (`0011`)
- `idx_outbox_status(status, created_at, event_id)` for retry/dequeue scans.
- `uniq_outbox_request(request_id)` for enqueue idempotency.

## 3) Remaining Gap vs Frozen Hot-Path Query Shapes

1. Pack C stabilization remains: complete production backfill coverage and eventual fallback removal (C5 final step).

## 4) Proposed Migration Packs

### Pack A — Must-have for hot-path stability (applied via `0025`)

- `uniq_chat_thread_id` on `chat_thread(thread_id)` UNIQUE
- `idx_chat_thread_scope_latest` on `chat_thread(scope_id, created_at, thread_id)`
- `idx_chat_member_thread_active` on `chat_member(thread_id, left_at, user_id)`
- `idx_chat_member_user_active` on `chat_member(user_id, left_at, thread_id)`
- `uniq_chat_read_cursor_key` on `chat_read_cursor(thread_id, user_id)` UNIQUE
- `uniq_feed_id` on `discovery_feed_item(feed_id)` UNIQUE
- `uniq_notification_id` on `discovery_notification(notification_id)` UNIQUE

### Pack B — Performance hardening (applied via `0026`)

- `idx_notification_unread` on `discovery_notification(user_id, read_at, created_at, notification_id)`
- `idx_feed_privacy_latest` on `discovery_feed_item(privacy_level, occurred_at, feed_id)`
- Optional `idx_feed_actor_latest` recheck if actor-involvement paths dominate.

### Pack C — Involvement scaling (activated with fallback)

Selected approach:
- participant fanout/materialized lane (`feed_participant_edge`) for `involvement_only`.
- edge-first reads with legacy fallback guard until C5 stabilization is complete.

Pack C is active on `involvement_only` reads using `feed_participant_edge` first.
Detailed Pack C design reference:
- `docs/database/hot-path-pack-c-feed-participant-edge-design-v1.md`

## 5) Execution State

1. ✅ Applied Pack A migration pair:
   - `0025_hot_path_pack_a_indexes.surql`
   - `0025_hot_path_pack_a_indexes_check.surql`
2. ✅ Ran live probe and feed benchmark gates against frozen SLO query shapes.
3. ✅ Applied Pack B migration pair:
   - `0026_hot_path_pack_b_indexes.surql`
   - `0026_hot_path_pack_b_indexes_check.surql`
4. ✅ Applied Pack C Phase C1 migration pair (schema only):
   - `0027_hot_path_pack_c_feed_participant_edge.surql`
   - `0027_hot_path_pack_c_feed_participant_edge_check.surql`
5. ✅ Applied Pack C Phase C2 dual-write:
   - `DiscoveryService::ingest_feed` writes `discovery_feed_item` + participant edges.
   - Replay-conflict path also re-upserts participant edges for self-heal.
6. ✅ Applied Pack C Phases C3/C4:
   - `SurrealDiscoveryFeedRepository` uses edge-first reads for `involvement_only` on feed/search.
   - Legacy query fallback auto-fills when edge lane coverage is incomplete.
   - Fallback path performs ordering shadow check and warns on divergence.
7. 🔄 Pack C Phase C5 in progress:
   - ✅ backfill command added (`feed-participant-edge-backfill`),
   - ✅ explicit lane/shadow counters added for involvement reads,
   - ✅ live benchmark script compares legacy OR lane vs materialized edge lane (`just feed-involvement-bench-surreal`),
   - ✅ one-command readiness gate runs mandatory pre-cutover checks (`just pack-c-cutover-readiness`),
   - ✅ runtime fallback switch available (`DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED`),
   - ⏳ remove fallback after sustained correctness/SLO verification.

Current benchmark artifacts:
- `docs/research/surrealdb-live-db-probe-latest.md`
- `docs/research/surrealdb-chat-bench-latest.md`
- `docs/research/surrealdb-feed-index-bench-latest.md`
- `docs/research/surrealdb-feed-involvement-bench-latest.md`
- `docs/research/pack-c-cutover-readiness-latest.md`
- `docs/research/surrealdb-notification-bench-latest.md`

## 6) Validation Gates

- Contract source: `docs/database/hot-path-query-shape-slo-matrix.md`
- Live DB baseline: `just dev-db-up`
- Release checks: `just release-gates-surreal`
- Chat benchmark: `just chat-bench-surreal`
- Feed source benchmark: `just feed-index-bench-surreal`
- Feed involvement benchmark: `just feed-involvement-bench-surreal`
- Pack C cutover readiness gate: `just pack-c-cutover-readiness`
- Pack C Stage A kickoff helper: `just pack-c-stage-a-kickoff`
- Notification benchmark: `just notification-bench-surreal`
- Re-run hot-path smoke/bench scripts after each migration pack.
