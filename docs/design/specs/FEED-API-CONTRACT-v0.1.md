# Gotong Royong — Feed API Contract v0.1

## Status
Draft: 2026-02-19 | Author: Design Session
Purpose: Backend API contract for the Pulse feed system. Frontend types are defined in `apps/web/src/lib/types/feed.ts`.

---

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>`.
The `user_id` is derived from the token — never passed as a body parameter.

---

## Endpoints

### 1. `GET /api/feed`

Fetch the user's feed, merged from 3 layers (Ikutan, Terlibat, Sekitar).

**Query parameters:**

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `filter` | `semua \| ikutan \| terlibat \| sekitar` | `semua` | Filter by feed layer |
| `cursor` | `string` | — | Opaque cursor for pagination (from previous response) |
| `limit` | `number` | `20` | Items per page (max 50) |

**Response: `200 OK`**

```json
{
  "items": FeedItem[],
  "next_cursor": "string | null"
}
```

Each `FeedItem` matches the TypeScript interface in `feed.ts`:

```typescript
{
  witness_id: string;
  title: string;
  track_hint?: string;
  status: WitnessStatus;          // "draft" | "open" | "active" | "resolved" | "closed"
  rahasia_level: RahasiaLevel;    // "L0" | "L1" | "L2" | "L3"
  latest_event: {
    event_id: string;
    event_type: FeedEventType;    // "created" | "joined" | "checkpoint" | ...
    actor_name: string;
    actor_avatar?: string;
    actor_role?: WitnessMemberRole;
    timestamp: string;            // ISO 8601
    verb: string;
    snippet?: string;
  };
  collapsed_count: number;
  member_count: number;
  members_preview: FeedMemberPreview[];  // max 5
  entity_tags: EntityTag[];
  urgency?: UrgencyBadge;         // "baru" | "voting" | "selesai" | "ramai"
  source: FeedSource;             // "ikutan" | "terlibat" | "sekitar"
  repost?: RepostFrame;
}
```

**Backend logic notes:**

- Feed assembly must merge 3 layers:
  - **Ikutan**: Witnesses tagged with entities the user follows (via `user_follows` table)
  - **Terlibat**: Witnesses where the user is a member (via `witness_members` table)
  - **Sekitar**: Trending witnesses in the user's geographic area (proximity + popularity algorithm)
- Deduplicate: a witness appearing in multiple layers should appear once, with `source` set to the highest-priority layer (terlibat > ikutan > sekitar)
- Sort by `latest_event.timestamp` DESC
- Urgency badges are computed server-side based on:
  - `baru`: witness created < 1 hour ago
  - `voting`: has an open vote block with a deadline
  - `selesai`: resolved in the last 24 hours
  - `ramai`: > 10 events in the past 24 hours
- Repost frames: when showing a witness that appears in the user's Ikutan because someone they follow participated, include the `repost` field with the follower's role context
- L2+ Rahasia witnesses are NEVER shown in Sekitar layer and NEVER include repost frames

---

### 2. `GET /api/feed/suggestions`

Fetch suggested entities for onboarding (shown when user follows < 3 entities).

**Response: `200 OK`**

```json
{
  "entities": FollowableEntity[]
}
```

Each `FollowableEntity`:

```typescript
{
  entity_id: string;
  entity_type: EntityType;        // "lingkungan" | "topik" | "kelompok" | "lembaga" | "warga"
  label: string;
  followed: boolean;              // always false in suggestions
  description?: string;
  witness_count: number;
  follower_count: number;
}
```

**Backend logic notes:**

- Suggestions are based on the user's registered location (kelurahan, kecamatan)
- Return the top 4–6 entities sorted by `witness_count` DESC
- Exclude entities the user already follows
- Only return entities of type `lingkungan` and `topik` (not `warga`)

---

### 3. `POST /api/entities/{entity_id}/follow`

Follow an entity. Idempotent — following an already-followed entity returns success.

**Response: `200 OK`**

```json
{
  "followed": true,
  "entity_id": "ent-001"
}
```

---

### 4. `DELETE /api/entities/{entity_id}/follow`

Unfollow an entity. Idempotent — unfollowing a not-followed entity returns success.

**Response: `200 OK`**

```json
{
  "followed": false,
  "entity_id": "ent-001"
}
```

---

### 5. `GET /api/entities/{entity_id}`

Get full detail for a single entity.

**Response: `200 OK`**

```json
{
  "entity_id": "ent-001",
  "entity_type": "lingkungan",
  "label": "RT 05 Menteng",
  "followed": true,
  "description": "Komunitas warga RT 05 Kelurahan Menteng",
  "witness_count": 23,
  "follower_count": 45
}
```

---

### 6. `PATCH /api/witnesses/{witness_id}/repost`

Toggle repost visibility for a witness the user participates in.

**Request body:**

```json
{
  "enabled": true
}
```

**Response: `200 OK`**

```json
{
  "repost_enabled": true
}
```

**Backend logic notes:**

- Only works if the user is a member of the witness
- Returns `403` if the witness is Rahasia L2+ (repost never allowed)
- Default repost state depends on the user's role and event type (see FEED-SYSTEM-SPEC §5.1)

---

## Error Responses

All endpoints use standard error format:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Entity not found"
  }
}
```

| HTTP Status | Code | When |
|-------------|------|------|
| `400` | `BAD_REQUEST` | Invalid query params or body |
| `401` | `UNAUTHORIZED` | Missing or invalid token |
| `403` | `FORBIDDEN` | Rahasia level restriction, not a member |
| `404` | `NOT_FOUND` | Entity or witness not found |
| `429` | `RATE_LIMITED` | Too many requests |

---

## Database Tables (Suggested Schema)

These are suggestions — adapt to your ORM/database of choice.

```sql
-- Entity follow graph
CREATE TABLE user_follows (
  user_id     UUID NOT NULL,
  entity_id   TEXT NOT NULL,
  followed_at TIMESTAMPTZ DEFAULT now(),
  PRIMARY KEY (user_id, entity_id)
);

-- Repost preferences per witness participation
CREATE TABLE witness_repost_prefs (
  user_id     UUID NOT NULL,
  witness_id  TEXT NOT NULL,
  enabled     BOOLEAN DEFAULT TRUE,
  PRIMARY KEY (user_id, witness_id)
);

-- Feed events (denormalized for fast feed assembly)
CREATE TABLE feed_events (
  event_id    TEXT PRIMARY KEY,
  witness_id  TEXT NOT NULL,
  event_type  TEXT NOT NULL,
  actor_id    UUID,
  actor_name  TEXT NOT NULL,
  actor_role  TEXT,
  timestamp   TIMESTAMPTZ NOT NULL,
  verb        TEXT NOT NULL,
  snippet     TEXT,
  created_at  TIMESTAMPTZ DEFAULT now()
);

-- Witness-entity tagging (populated by AI-00)
CREATE TABLE witness_entity_tags (
  witness_id  TEXT NOT NULL,
  entity_id   TEXT NOT NULL,
  PRIMARY KEY (witness_id, entity_id)
);

-- Entities (knowledge graph nodes)
CREATE TABLE entities (
  entity_id     TEXT PRIMARY KEY,
  entity_type   TEXT NOT NULL,
  label         TEXT NOT NULL,
  description   TEXT,
  witness_count INT DEFAULT 0,
  follower_count INT DEFAULT 0
);
```

---

## Implementation Priority

1. **`GET /api/feed`** — Core endpoint, start with Terlibat layer only (simplest: just user's witnesses)
2. **`GET /api/feed/suggestions`** — Needed for onboarding UX
3. **`POST/DELETE /api/entities/{entity_id}/follow`** — Enables Ikutan layer
4. **`GET /api/feed`** with Ikutan layer — Merge followed entities into feed
5. **`PATCH /api/witnesses/{witness_id}/repost`** — Can be deferred until social features are prioritized
6. **Sekitar layer** — Algorithm-based, can be deferred until user base grows
