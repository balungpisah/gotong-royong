# Gotong-Royong — Endpoint Inventory + Hot-Path API Shapes (Freeze Doc)

Status: draft reference (no implementation changes)  
Last updated: 2026-02-24

Objective:
- List all current Gotong HTTP endpoints grouped by domain.
- **Freeze the “hot path” API shapes** (chat + feed + notifications) so SurrealDB schema and indexes are driven by real query patterns.
- Freeze **latency/SLO gates** for those hot paths so backend and frontend integration use the same performance contract.
- Provide the **read-model-first** SurrealDB mapping (tables + indexes + representative queries), and then the triples/relations layer for enrichment/audit.

Non-goals:
- Changing endpoint behavior or database schema in this document.
- Re-architecting other domains (adaptive path, vault, moderation, ontology authoring, etc).

---

## 1) Endpoint Inventory (Grouped by Domain)

Notes:
- Most `/v1/*` routes inside the `protected` router require auth (`require_auth_middleware`).
  - Exceptions: `/v1/auth/*` and `/v1/echo` are mounted outside `protected`.
- `x-request-id` is set server-side if missing, but **clients must send a stable `x-request-id`** to get idempotent retries (otherwise each retry becomes a new request).
- `x-correlation-id` is auto-generated if missing; clients should still pass it for trace continuity.

### System

| Method | Path | Purpose |
|---|---|---|
| GET | `/health` | Liveness + version + env |
| GET | `/metrics` | Prometheus metrics |

### Utilities

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/echo` | Debug echo |
| POST | `/v1/idempotent-echo` | Debug idempotency lane (auth required) |

### Auth

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/auth/signup` | Register |
| POST | `/v1/auth/signin` | Login |
| POST | `/v1/auth/refresh` | Refresh session |
| POST | `/v1/auth/logout` | Logout |
| GET | `/v1/auth/me` | Current session identity |

### Contributions

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/contributions` | Create contribution (idempotent) |
| GET | `/v1/contributions` | List contributions |
| GET | `/v1/contributions/:contribution_id` | Get contribution by id |
| GET | `/v1/contributions/:contribution_id/evidence` | List evidence attached to contribution |

### Evidence (PoR / attachments)

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/evidence` | Submit evidence (idempotent) |
| GET | `/v1/evidence/:evidence_id` | Get evidence |

### Vouches

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/vouches` | Submit vouch (idempotent) |
| GET | `/v1/vouches` | List vouches |

### Adaptive Path

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/adaptive-path/plans` | Create plan (idempotent) |
| GET | `/v1/adaptive-path/plans/:plan_id` | Get plan |
| GET | `/v1/adaptive-path/entities/:entity_id/plan` | Get plan by entity |
| POST | `/v1/adaptive-path/plans/:plan_id/update` | Update plan (idempotent) |
| GET | `/v1/adaptive-path/plans/:plan_id/events` | List plan events |
| POST | `/v1/adaptive-path/plans/:plan_id/suggestions` | Propose suggestion (idempotent) |
| GET | `/v1/adaptive-path/plans/:plan_id/suggestions` | List suggestions |
| POST | `/v1/adaptive-path/suggestions/:suggestion_id/accept` | Accept suggestion (idempotent) |
| POST | `/v1/adaptive-path/suggestions/:suggestion_id/reject` | Reject suggestion (idempotent) |

### Vaults

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/vaults` | Create vault draft (idempotent) |
| GET | `/v1/vaults` | List vaults |
| GET | `/v1/vaults/:vault_entry_id` | Get vault entry |
| DELETE | `/v1/vaults/:vault_entry_id` | Delete vault draft (idempotent) |
| POST | `/v1/vaults/:vault_entry_id/update` | Update vault (idempotent) |
| POST | `/v1/vaults/:vault_entry_id/seal` | Seal vault (idempotent) |
| POST | `/v1/vaults/:vault_entry_id/publish` | Publish vault (idempotent) |
| POST | `/v1/vaults/:vault_entry_id/revoke` | Revoke vault (idempotent) |
| POST | `/v1/vaults/:vault_entry_id/expire` | Expire vault (idempotent) |
| GET | `/v1/vaults/:vault_entry_id/timeline` | Vault timeline |
| GET | `/v1/vaults/:vault_entry_id/trustees` | List trustees |
| POST | `/v1/vaults/:vault_entry_id/trustees` | Add trustee (idempotent) |
| DELETE | `/v1/vaults/:vault_entry_id/trustees/:wali_id` | Remove trustee (idempotent) |

### Moderation

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/moderations` | Apply moderation action (idempotent) |
| GET | `/v1/moderations/review-queue` | List review queue |
| GET | `/v1/moderations/:content_id` | Get moderation view |

### Discovery (Feed / Search / Notifications) — HOT PATH

| Method | Path | Purpose |
|---|---|---|
| GET | `/v1/feed` | List discovery feed (cursor pagination) |
| GET | `/v1/search` | Search discovery feed |
| GET | `/v1/notifications` | List notifications (cursor pagination) |
| POST | `/v1/notifications/:notification_id/read` | Mark notification read (idempotent) |
| GET | `/v1/notifications/unread-count` | Unread count |
| GET | `/v1/notifications/weekly-digest` | Weekly digest summary |

### Ontology (Triples / enrichment layer)

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/ontology/concepts` | Upsert concept |
| GET | `/v1/ontology/concepts/:qid` | Get concept by Wikidata QID |
| POST | `/v1/ontology/concepts/:concept_id/broader/:broader_id` | Add broader edge |
| GET | `/v1/ontology/concepts/:concept_id/hierarchy` | List hierarchy |
| POST | `/v1/ontology/feed` | Create ontology note (idempotent); public notes are also ingested into discovery feed |
| POST | `/v1/ontology/notes/:note_id/vouches` | Vouch a note |
| POST | `/v1/ontology/notes/:note_id/challenges` | Challenge a note |
| GET | `/v1/ontology/notes/:note_id/feedback` | Feedback |
| GET | `/v1/ontology/notes/:note_id/ranked` | Ranking |

### Tandang proxy (server-to-server reads)

| Method | Path | Purpose |
|---|---|---|
| GET | `/v1/tandang/me/profile` | Profile snapshot |
| GET | `/v1/tandang/cv-hidup/qr` | CV hidup QR |
| POST | `/v1/tandang/cv-hidup/export` | CV hidup export |
| GET | `/v1/tandang/cv-hidup/verify/:export_id` | Verify export |
| GET | `/v1/tandang/skills/search` | Skill search |
| GET | `/v1/tandang/skills/nodes/:node_id` | Skill node |
| GET | `/v1/tandang/skills/nodes/:node_id/labels` | Skill labels |
| GET | `/v1/tandang/skills/nodes/:node_id/relations` | Skill relations |
| GET | `/v1/tandang/skills/:skill_id/parent` | Skill parent |
| GET | `/v1/tandang/por/requirements/:task_type` | PoR requirements |
| GET | `/v1/tandang/por/status/:evidence_id` | PoR status |
| GET | `/v1/tandang/por/triad-requirements/:track/:transition` | PoR triad requirements |
| GET | `/v1/tandang/reputation/leaderboard` | Reputation leaderboard |
| GET | `/v1/tandang/reputation/distribution` | Tier distribution |
| GET | `/v1/tandang/users/:user_id/vouch-budget` | Vouch budget |
| GET | `/v1/tandang/decay/warnings/:user_id` | Decay warnings |
| GET | `/v1/tandang/community/pulse/overview` | Community pulse |
| GET | `/v1/tandang/community/pulse/insights` | Community pulse |
| GET | `/v1/tandang/community/pulse/trends` | Community pulse |
| GET | `/v1/tandang/hero/leaderboard` | Hero leaderboard |
| GET | `/v1/tandang/hero/:user_id` | Hero status |
| GET | `/v1/tandang/slash/gdf` | GDF weather |

### Admin (Webhook outbox)

| Method | Path | Purpose |
|---|---|---|
| GET | `/v1/admin/webhooks/outbox` | List outbox events |
| GET | `/v1/admin/webhooks/outbox/:event_id` | Get outbox event |
| GET | `/v1/admin/webhooks/outbox/:event_id/logs` | Delivery logs |

### Chat — HOT PATH

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/chat/threads` | Create thread (idempotent) |
| GET | `/v1/chat/threads` | List threads |
| GET | `/v1/chat/threads/:thread_id/members` | List members |
| POST | `/v1/chat/threads/:thread_id/join` | Join thread (idempotent) |
| POST | `/v1/chat/threads/:thread_id/leave` | Leave thread (idempotent) |
| GET | `/v1/chat/threads/:thread_id/messages` | List messages (catch-up) |
| GET | `/v1/chat/threads/:thread_id/messages/poll` | Poll messages (same semantics) |
| POST | `/v1/chat/threads/:thread_id/messages/send` | Send message (idempotent) |
| GET | `/v1/chat/threads/:thread_id/messages/stream` | SSE message stream |
| GET | `/v1/chat/threads/:thread_id/messages/ws` | WebSocket message stream |
| GET | `/v1/chat/threads/:thread_id/read-cursor` | Get read cursor |
| POST | `/v1/chat/threads/:thread_id/read-cursor` | Mark read cursor (idempotent) |

### EdgePod AI (duplicate routes)

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/edge-pod/ai/03/duplicate-detection` | AI-03 |
| POST | `/v1/edge-pod/ai/05/gaming-risk` | AI-05 |
| POST | `/v1/edge-pod/ai/08/sensitive-media` | AI-08 |
| POST | `/v1/edge-pod/ai/09/credit-recommendation` | AI-09 |
| POST | `/v1/edge-pod/ai/siaga/evaluate` | SIAGA evaluate |
| POST | `/api/v1/edge-pod/ai/03/duplicate-detection` | AI-03 (alt prefix) |
| POST | `/api/v1/edge-pod/ai/05/gaming-risk` | AI-05 (alt prefix) |
| POST | `/api/v1/edge-pod/ai/08/sensitive-media` | AI-08 (alt prefix) |
| POST | `/api/v1/edge-pod/ai/09/credit-recommendation` | AI-09 (alt prefix) |
| POST | `/api/v1/edge-pod/ai/siaga/evaluate` | SIAGA evaluate (alt prefix) |

---

## 2) Hot-Path API Shapes (Freeze)

This section is the contract we should treat as “frozen” while designing/benchmarking SurrealDB query plans and indexes.

### 2.1 Feed — `GET /v1/feed`

**Query params** (current):
- `cursor?: string` (format: `<occurred_at_ms>:<feed_id>`)
- `limit?: number` (default 20, max 50)
- `scope_id?: string`
- `privacy_level?: string`
- `from_ms?: number`
- `to_ms?: number`
- `involvement_only?: bool` (default false)

**Response**: `PagedFeed { items: FeedItem[], next_cursor?: string }`

**Ordering rule**:
- Descending by `(occurred_at_ms, feed_id)`. Cursor is the last item’s `(occurred_at_ms, feed_id)`.

**Visibility rule** (domain-level):
- Items must be visible to the requesting actor (privacy + participant checks).

**Read-model requirement**:
- Must be satisfiable by a single-table time-ordered query with optional filters and stable cursor pagination.

**Known `source_type` values** (not exhaustive):
- `contribution` (from `POST /v1/contributions`)
- `vouch` (from `POST /v1/vouches`)
- `ontology_note` (from `POST /v1/ontology/feed`, when `rahasia_level == 0`)

**`ontology_note` payload contract** (best-effort enrichment, non-hot-path):
- `payload.note`: the created `OntologyNote` (public notes only).
- `payload.enrichment`: enrichment block (may be partial; worker refreshes asynchronously on sync-enrichment failure).
  - `payload.enrichment.tags`: `{ concept_qids: string[], action_types: string[], place_ids: string[] }`
  - `payload.enrichment.labels`: `{ concepts: OntologyConcept[], actions: OntologyActionRef[], places: OntologyPlaceRef[] }`
  - `payload.enrichment.feedback`: `{ vouch_count: number, challenge_count: number, score: number }`
  - `payload.enrichment.status`: `pending|computed`
  - `payload.enrichment.enriched_at_ms`: epoch milliseconds (initial aggregate marker)
  - `payload.enrichment.tags_enriched_at_ms`: epoch milliseconds (tag/label refresh marker)
  - `payload.enrichment.feedback_enriched_at_ms`: epoch milliseconds (feedback refresh marker)
  - On `status=pending`, API still emits empty `tags/labels` plus feedback snapshot; worker later fills tag/label fields.

**Privacy gating (current)**:
- Ontology notes are only ingested into discovery feed when `rahasia_level == 0` (public).
- Notes with `rahasia_level > 0` remain queryable only via the ontology endpoints (until scope-based visibility rules exist for feed).
- TTL cleanup marks expired ontology feed items with `payload.lifecycle.hidden=true` and feed/search visibility filters drop hidden items.

### 2.2 Notifications — `GET /v1/notifications`

**Query params** (current):
- `cursor?: string` (format: `<created_at_ms>:<notification_id>`)
- `limit?: number` (default 20, max 50)
- `include_read?: bool` (default false)

**Response**: `PagedNotifications { items: InAppNotification[], next_cursor?: string }`

**Ordering rule**:
- Descending by `(created_at_ms, notification_id)`.

### 2.3 Notifications — unread count + mark read

- `GET /v1/notifications/unread-count` → `{ unread_count: number }`
- `POST /v1/notifications/:notification_id/read` → returns the updated `InAppNotification`

### 2.4 Chat threads — list/create/join/leave

**Create thread**: `POST /v1/chat/threads`  
Request body:
```json
{ "scope_id": "string", "privacy_level": "public|private" }
```
Response: `ChatThread`

Idempotency:
- Keyed by `(operation="chat_thread_create", actor_id, x-request-id)`.
- Clients must reuse `x-request-id` on retry to avoid duplicate thread creation.

**List threads**: `GET /v1/chat/threads?scope_id=...`  
- If `scope_id` is present: list threads in a scope visible to actor.
- Else: list threads for the actor (membership).

**Join/leave**: `POST /v1/chat/threads/:thread_id/join|leave`  
Response: `ChatMember`  
Idempotency:
- Keyed by `(operation, actor_id:thread_id, x-request-id)`.

### 2.5 Chat messages — catch-up list + send

**Catch-up list**: `GET /v1/chat/threads/:thread_id/messages` (and `/poll`)  
Query params:
- `since_created_at_ms?: number`
- `since_message_id?: string`
- `limit?: number` (defaults to 50, clamped 1..=200)

Rule:
- If `since_message_id` is provided, `since_created_at_ms` is required.
- If neither `since_created_at_ms` nor `since_message_id` is provided, return the **latest** `limit` messages (still ordered ascending in the response).

Response:
- `ChatMessage[]` ordered ascending by `(created_at_ms, message_id)`.

**Send**: `POST /v1/chat/threads/:thread_id/messages/send`  
Request:
```json
{ "body": "string", "attachments": [] }
```
Response: `ChatMessage`

Idempotency:
- Keyed by `(operation="chat_message_send", actor_id:thread_id, x-request-id)`.

### 2.6 Chat streaming — SSE + WebSocket

- `GET /v1/chat/threads/:thread_id/messages/stream` (SSE)
- `GET /v1/chat/threads/:thread_id/messages/ws` (WS)

Both accept the same catch-up query params to seed a backlog, then stream new messages.

### 2.7 Tandang trust reads (hot-adjacent contract)

These reads are not the chat/feed DB hot path, but they are render-critical for trust UI and must stay predictable:

- `GET /v1/tandang/me/profile`
- `GET /v1/tandang/reputation/leaderboard`
- `GET /v1/tandang/users/:user_id/vouch-budget`
- `GET /v1/tandang/decay/warnings/:user_id`

Contract stance:
- Use cached/stale-while-revalidate patterns for non-blocking UI hydration.
- Never block chat send/receive path on these reads.
- If upstream is degraded, keep prior snapshot + stale marker.

### 2.8 Latency SLO Contract (Hard Gates)

Targets below are authoritative for benchmark/release gates:

| Operation | Route / Flow | p50 | p95 | p99 |
|---|---|---:|---:|---:|
| Chat send ACK | `POST /v1/chat/threads/:thread_id/messages/send` | ≤ 50 ms | ≤ 120 ms | ≤ 250 ms |
| Chat delivery | Server push to subscribed WS/SSE clients | ≤ 80 ms | ≤ 200 ms | ≤ 400 ms |
| Feed list (20 items) | `GET /v1/feed?limit=20` | ≤ 70 ms | ≤ 180 ms | ≤ 350 ms |
| Notifications list (20 items) | `GET /v1/notifications?limit=20` | ≤ 60 ms | ≤ 150 ms | ≤ 300 ms |

Measurement notes:
- Measure service-side latency at API boundary (exclude client network variance).
- Validate under worst-credible load envelopes, not single-user local runs only.
- Failing p95 gates blocks rollout for affected path.

### 2.9 Delivery Priority (Execution Order)

To reduce integration risk, delivery sequence is fixed:
1. Chat fast path (`send`, `catch-up`, `stream`) until SLO-stable.
2. Feed + trust surfaces (relevance + Tandang trust widgets).
3. Notifications polish (read-state UX + digest tuning).

---

## 3) SurrealDB Read-Model-First Mapping (Current + Required)

This section ties the frozen API shapes to concrete SurrealDB tables/indexes and (representative) query patterns.

### 3.1 Chat read model

Surreal schema + indexes:
- `database/migrations/0001_initial_schema.surql`
- `database/migrations/0002_chat_indexes.surql`
- Permissions: `database/migrations/0019_record_permissions.surql`

Tables:
- `chat_thread`
- `chat_member`
- `chat_message`
- `chat_read_cursor`
- `chat_delivery_event`

Critical indexes (must remain valid under benchmarks):
- `idx_message_order` on `(thread_id, created_at, message_id)` (deterministic catch-up)
- `idx_member_lookup` on `(user_id, thread_id)` (membership checks)
- `idx_read_cursor_lookup` on `(user_id, thread_id)`
- `uniq_message_request` on `(thread_id, request_id)` (idempotent send)

Representative SurrealQL patterns (from `crates/infra/src/repositories/impls.rs`):
- Catch-up:
  - `WHERE thread_id = $thread_id AND (created_at > $t OR (created_at = $t AND message_id > $id)) ORDER BY created_at ASC, message_id ASC LIMIT $limit`
- Idempotent send read:
  - `WHERE thread_id = $thread_id AND request_id = $request_id LIMIT 1`

### 3.2 Feed read model

Surreal schema:
- `database/migrations/0008_discovery_schema.surql`
- Permissions: `database/migrations/0019_record_permissions.surql`

Tables:
- `discovery_feed_item`

Indexes (current):
- `idx_feed_scope` on `(scope_id, occurred_at, feed_id)`
- `idx_feed_time` on `(occurred_at, feed_id)`
- `uniq_feed_source_request` on `(source_type, source_id, request_id)` (idempotent ingestion)

Representative query pattern (from `crates/infra/src/repositories/impls.rs`):
- `ORDER BY occurred_at DESC, feed_id DESC LIMIT $limit`
- Cursor:
  - `(occurred_at < $cursor OR (occurred_at = $cursor AND feed_id < $feed_id))`

### 3.3 Notifications read model

Surreal schema:
- `database/migrations/0008_discovery_schema.surql`

Tables:
- `discovery_notification`

Indexes:
- `idx_notification_user` on `(user_id, created_at, notification_id)`
- `uniq_notification_dedupe` on `(user_id, dedupe_key)` (idempotent notification ingestion)

### 3.4 Outbox (robustness lane)

Surreal schema:
- `database/migrations/0011_webhook_outbox_schema.surql`

Tables:
- `webhook_outbox_event`
- `webhook_delivery_log`

Indexes:
- `idx_outbox_status` on `(status, created_at, event_id)`
- `uniq_outbox_request` on `(request_id)` (idempotent enqueue)

### 3.5 Idempotency (hot write safety)

Current implementation:
- Idempotency storage is backed by Redis (`RedisIdempotencyStore`), not Surreal.

Design stance (freeze):
- Keep idempotency durable and fast; DB choice is allowed to remain Redis as long as:
  - keys are stable and bounded,
  - eviction/TTL is policy-driven,
  - replays are deterministic.

---

## 4) Triples/Relations Layer (Enrichment + Audit)

Ontology/triples schema exists:
- `database/migrations/0013_ontology_schema.surql`

Stance:
- Use triples/relations for enrichment (tags, broader/narrower, located-at) and audit signals (vouch/challenge on notes).
- Do **not** depend on multi-hop traversals to render hot lists (feed/chat) on the request path; instead, denormalize enrichment fields into read models asynchronously.

---

## 5) Known Risks / Fix-Next Candidates (for tracking)

These are intentionally documented as “next candidates” rather than fixed here.

### Feed `involvement_only` pagination correctness

Status: **fixed** (2026-02-24)

`DiscoveryService::list_feed()` relies on the repository returning `limit+1` raw rows to determine whether to continue paginating.

If a repository applies `involvement_only` filtering *after* the DB query (reducing result length), it can prematurely signal “end of feed” even when older matching rows exist.

Fix:
- Surreal feed/search repositories now apply `involvement_only` as a DB-level predicate (no post-query retain), so `rows.len()` remains meaningful for pagination.
- Pack C participant-edge lane is active for `involvement_only=true` (edge-first with legacy fallback + shadow warning on fallback):
  - `docs/database/hot-path-pack-c-feed-participant-edge-design-v1.md`

### Search scaling

Current search is effectively “scan a time-ordered window, then match text in application code”.

Action (future):
- Decide if/when to introduce SurrealDB FULLTEXT indexes for discovery content (not chat).

---

## 6) References

- Canonical hot-path query/SLO matrix: `docs/database/hot-path-query-shape-slo-matrix.md`
- Hot-path read-model design packs: `docs/database/hot-path-read-model-design-v1.md`
- Pack C feed participant-edge design: `docs/database/hot-path-pack-c-feed-participant-edge-design-v1.md`
- SurrealDB v3 feature audit + benchmark matrix: `docs/database/surrealdb-v3-feature-audit-benchmark-matrix.md`
- Chat schema requirements: `docs/database/schema-requirements.md`
- Surreal read models: `database/migrations/0001_initial_schema.surql`, `database/migrations/0002_chat_indexes.surql`, `database/migrations/0008_discovery_schema.surql`, `database/migrations/0011_webhook_outbox_schema.surql`
- Ontology triples layer: `database/migrations/0013_ontology_schema.surql`
