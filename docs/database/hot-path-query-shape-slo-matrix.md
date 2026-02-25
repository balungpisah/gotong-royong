# Hot-Path Query Shape + SLO Matrix (Canonical)

Status: active contract  
Last updated: 2026-02-25

Purpose:
- Make DB/index design follow real API query shapes.
- Bind each hot-path operation to explicit latency gates.
- Prevent drift between backend behavior and frontend expectations.

Primary references:
- `docs/architecture/hot-path-api-shapes.md`
- `docs/architecture/tandang-gameplay-rules.md`

## 1) Assumptions (Locked)

- Page-size defaults for latency gates:
  - Feed: `limit=20`
  - Notifications: `limit=20`
  - Chat catch-up: default `limit=50`, max `200`
- All hot writes use stable `x-request-id` for idempotency.
- Triples/graph traversals are enrichment-only and never required for request-path rendering.

## 2) Query Shape Matrix

| Path | Query shape | Required table(s) | Required index(es) | Consistency | SLO (p50 / p95 / p99) |
|---|---|---|---|---|---|
| `POST /v1/chat/threads/:thread_id/messages/send` | Membership check + idempotent lookup + append message | `chat_member`, `chat_message` | `idx_member_lookup`, `uniq_message_request`, `idx_message_order` | Read-your-write per thread | `≤50 / ≤120 / ≤250 ms` |
| `GET /v1/chat/threads/:thread_id/messages` | Cursor catch-up on `(created_at,message_id)` sorted ASC | `chat_message` | `idx_message_order` | Monotonic within thread | `≤70 / ≤170 / ≤320 ms` |
| `GET /v1/chat/threads/:thread_id/messages/ws|stream` | Catch-up + incremental fanout | `chat_message`, delivery lane | `idx_message_order` | At-least-once delivery; idempotent client apply | Delivery `≤80 / ≤200 / ≤400 ms` |
| `GET /v1/feed` | Time-ordered page, optional scope/privacy/time filters, cursor DESC | `discovery_feed_item` | `idx_feed_time`, `idx_feed_scope`, `idx_feed_source_latest` | Stable cursor ordering | `≤70 / ≤180 / ≤350 ms` |
| `GET /v1/search` | Feed-window search with same visibility gates | `discovery_feed_item` | `idx_feed_time` (+ FTS when enabled) | Same as feed visibility | `≤90 / ≤220 / ≤420 ms` |
| `GET /v1/notifications` | Per-user DESC cursor scan, include/exclude read | `discovery_notification` | `idx_notification_user` | Stable cursor ordering | `≤60 / ≤150 / ≤300 ms` |
| `POST /v1/notifications/:notification_id/read` | Point update + dedupe/idempotency | `discovery_notification` | `idx_notification_user` | Read-after-write for same user | `≤40 / ≤120 / ≤240 ms` |
| `GET /v1/notifications/unread-count` | Per-user unread aggregate | `discovery_notification` | `idx_notification_user` | Eventually consistent acceptable | `≤40 / ≤120 / ≤220 ms` |

## 3) Tandang Trust Reads (Hot-Adjacent)

These are UI-critical but must not block chat/feed critical loops:

| Path | Policy | Budget |
|---|---|---|
| `GET /v1/tandang/me/profile` | Serve cached snapshot with stale marker if upstream slow | Target `≤120 / ≤300 / ≤600 ms` |
| `GET /v1/tandang/users/:user_id/profile` | Serve cached snapshot with stale marker if upstream slow | Target `≤120 / ≤300 / ≤600 ms` |
| `GET /v1/tandang/users/:user_id/vouch-budget` | Cache + bounded TTL, fallback to last known value in UI | Target `≤120 / ≤300 / ≤600 ms` |
| `GET /v1/tandang/decay/warnings/:user_id` | Same as above | Target `≤120 / ≤300 / ≤600 ms` |
| `GET /v1/tandang/reputation/leaderboard` | Cache at list-level, async refresh | Target `≤150 / ≤350 / ≤700 ms` |

## 4) Read-Model Design Constraints

- Keep request-path queries single-table or single-index-friendly joins only.
- Keep cursor ordering deterministic on tie-break keys (`feed_id`, `notification_id`, `message_id`).
- Keep idempotency conflict checks O(log n) via unique indexes.
- Keep enrichment fields denormalized in feed payload; no runtime graph traversal for listing.

## 5) Explicit Non-Goals (Prevent Regressions)

- No multi-hop graph traversal on chat/feed/notification request paths.
- No synchronous dependency on ontology enrichment worker for API response success.
- No reliance on client-generated timestamps for ordering correctness.

## 6) Pack C Triggered Shape (Active with fallback)

When Pack C is activated for `involvement_only=true`, the feed path shifts to:
- `feed_participant_edge` (candidate IDs by actor + cursor/index)
- then hydrate `discovery_feed_item` by `feed_id` for final payload/visibility checks.

Current state:
- C1 schema pair is applied (`0027_*`).
- C2 dual-write is active on feed ingest (participant edges are written on create and replay-conflict paths).
- C3/C4 read cutover is active in `SurrealDiscoveryFeedRepository` for `involvement_only=true`:
  - edge-first lookup on `feed_participant_edge`,
  - legacy fallback merge when edge coverage is incomplete,
  - fallback-path ordering shadow check via warning logs.
- C5 stabilization is in progress:
  - backfill command available (`feed-participant-edge-backfill`),
  - explicit lane/shadow counters are emitted from involvement read paths,
  - live benchmark now compares legacy OR lane vs edge lane (`just feed-involvement-bench-surreal`),
  - runtime switch available: `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED` (`true`=fallback on, `false`=edge-only),
  - fallback removal remains pending final verification window.

Reference design:
- `docs/database/hot-path-pack-c-feed-participant-edge-design-v1.md`
- `docs/deployment/feed-involvement-fallback-removal-runbook.md`
