# Engagement Backend Contract v0.1

> **Version 0.1** | **Living Document** | **Status: DRAFT**
> Last updated: 2026-02-19

<!-- STATUS: DRAFT — no fields are backend-implemented yet -->

---

## Purpose

This document bridges the frontend engagement features (Phase 1 "Feel Alive" and beyond) to backend implementation. It records what data sources, APIs, and schemas are needed to power each engagement field on `FeedItem`.

**Companion documents:**
- [ENGAGEMENT-STRATEGY-v0.1.md](./ENGAGEMENT-STRATEGY-v0.1.md) — Feature proposals, Octalysis mapping, ethical guardrails
- [FEED-API-CONTRACT-v0.1.md](./FEED-API-CONTRACT-v0.1.md) — Feed endpoint contracts (`GET /api/feed`)
- [FEED-SYSTEM-SPEC-v0.1.md](./FEED-SYSTEM-SPEC-v0.1.md) — Feed architecture, card anatomy, layers
- [Feature-contract map](../../research/feature-contract-map.md) — Master feature-to-backend mapping

**Frontend type source:** `apps/web/src/lib/types/feed.ts` (`FeedItem` interface)

---

## 1. Field Registry

Master table of all engagement-related fields on `FeedItem`. Every field that requires backend data to replace mock/hardcoded values is listed here.

<!-- STATUS: FIELD_REGISTRY — 11 fields tracked -->

| Field | Type | Data Source | Update Frequency | Phase | Frontend | Backend | Status |
|---|---|---|---|---|---|---|---|
| `hook_line` | `string?` | EP-00 triage output (extended) | on-triage | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |
| `pull_quote` | `string?` | EP-07 summary output (extended) | per-event (on new messages) | Phase 2 | Done | Not started | 🟢 FE / ⚪ BE |
| `sentiment` | `enum?` | EP-00 triage output (extended) | on-triage | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |
| `intensity` | `number?` (1-5) | EP-00 triage output (extended) | on-triage, re-scored on milestones | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |
| `cover_url` | `string?` | Evidence attachments / seed media | per-event (on new evidence) | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |
| `body` | `string?` | EP-07 summary output (extended) | per-event (periodic re-summarize) | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |
| `active_now` | `number?` | Presence service (new) | real-time (~30s window) | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |
| `deadline` | `string?` (ISO) | Track state machine (UI-03) | per-event (on stage transition) | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |
| `deadline_label` | `string?` | Track state machine (UI-03) | per-event (on stage transition) | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |
| `quorum_target` | `number?` | Track governance config (UI-03/UI-17) | per-event (on vote/phase open) | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |
| `quorum_current` | `number?` | Computed from vote/consent records | real-time (on each vote/consent) | Phase 1 | Done | Not started | 🟢 FE / ⚪ BE |

**Legend:**
- 🟢 Done
- 🔵 Spec'd (contract written, not yet implemented)
- 🟡 Needs Discussion
- ⚪ Not Started
- 🔴 Blocked

---

## 2. Data Source Contracts

### 2A. Presence Service (`active_now`)

<!-- STATUS: 2A_PRESENCE — NEEDS_DISCUSSION -->

**Purpose:** Provide anonymous count of users currently viewing or interacting with a witness. Powers the pulse glow animation and "N aktif" live badge.

**Architecture options:**

| Option | Mechanism | Latency | Complexity | Recommended |
|---|---|---|---|---|
| A1: SurrealDB LIVE SELECT | Write heartbeat rows, LIVE SELECT count | ~1-2s | Low | Yes (Phase 1) |
| A2: Dedicated WebSocket | Standalone presence channel per witness | <500ms | High | No (Phase 2+) |
| A3: Client polling | `GET /api/witnesses/{id}/presence` every 30s | 30s | Very Low | Fallback only |

**Proposed contract (Option A1):**

The client receives `active_now` as part of the feed response. For real-time updates on the detail view, a SurrealDB live subscription provides count changes.

**Heartbeat write (client -> backend):**

```
POST /api/presence/heartbeat
```

```json
{
  "witness_id": "witness-001",
  "session_id": "sess_abc123"
}
```

- Client sends heartbeat every 30 seconds while witness is in viewport or detail is open.
- Backend upserts a row in `witness_presence` with TTL.
- Rows older than 60 seconds are considered stale (2x heartbeat interval).

**Feed query enrichment:**

The `GET /api/feed` response includes `active_now` per item, computed as:

```sql
SELECT count() FROM witness_presence
WHERE witness_id = $witness_id
  AND last_seen > time::now() - 60s
GROUP BY witness_id;
```

**SurrealDB schema:**

```sql
DEFINE TABLE witness_presence SCHEMAFULL;
DEFINE FIELD witness_id ON TABLE witness_presence TYPE string;
DEFINE FIELD session_id ON TABLE witness_presence TYPE string;
DEFINE FIELD last_seen  ON TABLE witness_presence TYPE datetime;

DEFINE INDEX idx_presence_witness
  ON TABLE witness_presence FIELDS witness_id;

DEFINE INDEX uniq_presence_session
  ON TABLE witness_presence FIELDS witness_id, session_id UNIQUE;
```

**Privacy rules:**
- Counts only. No individual user IDs exposed to other clients.
- `session_id` is an opaque token, not mappable to `user_id` by other users.
- Stale rows are garbage-collected on a 5-minute cron cycle.
- Presence data is never persisted beyond the 60-second TTL window.

**TODO:** Decide whether `active_now` in the feed response is sufficient for Phase 1 or if real-time push (LIVE SELECT) is needed from day one.

---

### 2B. Deadline & Quorum Service (`deadline`, `deadline_label`, `quorum_target`, `quorum_current`)

<!-- STATUS: 2B_DEADLINE_QUORUM — PARTIAL -->

**Purpose:** Surface real deadlines and quorum progress from the track state machine onto feed cards. Powers countdown timers and quorum progress bars.

**Source:** Track governance workflow (UI-03 contract, `track_state_transition` table).

Deadlines and quorum thresholds are set when a governance action opens (vote, consent window, phase activation). These values are already part of the track state machine -- they need to be projected onto feed items.

**How data flows to the feed:**

```
track_state_transition (new vote_opened / consent_opened)
    |
    v
witness record updated with:
  - deadline (from governance duration table)
  - deadline_label (from transition type)
  - quorum_target (from governance config)
    |
    v
GET /api/feed reads these fields from witness record
    |
    v
quorum_current computed at query time from vote/consent count
```

**Governance duration mapping (from UI-UX-SPEC 13-transitions-and-quorum):**

| Transition Type | Duration | `deadline_label` |
|---|---|---|
| Checkpoint open -> completed (consent) | 24h | "Batas persetujuan" |
| Phase activation (simple, consent) | 24h | "Fase berakhir" |
| Phase activation (complex, vote) | 48h | "Voting ditutup" |
| Galang-related checkpoint (vote) | 48h | "Voting ditutup" |
| Checkpoint requiring evidence (vote) | 72h | "Voting ditutup" |
| Plan completion (vote + challenge) | 72h | "Voting ditutup" |
| Emergency fast-track | Immediate | "Darurat aktif" |

**Proposed witness record additions:**

```sql
DEFINE FIELD deadline       ON TABLE witness TYPE option<datetime>;
DEFINE FIELD deadline_label  ON TABLE witness TYPE option<string>;
DEFINE FIELD quorum_target   ON TABLE witness TYPE option<int>;
```

**`quorum_current` computation:**

Not stored. Computed at query time from existing vote/consent records:

```sql
-- For vote-type governance:
SELECT count() FROM governance_votes
WHERE witness_id = $witness_id
  AND governance_action_id = $active_action_id
GROUP ALL;

-- For consent-type governance:
SELECT count() FROM governance_consents
WHERE witness_id = $witness_id
  AND governance_action_id = $active_action_id
  AND status = 'approved'
GROUP ALL;
```

**Feed enrichment:** `quorum_current` is computed per-item during feed assembly. For feeds with many active governance items, consider caching the count on the witness record and updating it via a trigger on vote/consent insert.

**DISCUSSION NEEDED:** Should `quorum_current` be cached on the witness record for performance, or always computed at query time? Caching trades consistency for speed. With SurrealDB events/triggers, a write-through cache is feasible.

---

### 2C. LLM Enrichment (`hook_line`, `pull_quote`, `sentiment`, `intensity`, `body`)

<!-- STATUS: 2C_LLM_ENRICHMENT — NOT_STARTED -->

**Purpose:** AI-generated editorial fields that make feed cards emotionally resonant. Powers hook headlines, sentiment-colored shadows, and narrative bodies.

**Data flow:**

| Field | Generated By | When | Stored On |
|---|---|---|---|
| `hook_line` | EP-00 triage | During initial triage | `witness` record |
| `sentiment` | EP-00 triage | During initial triage | `witness` record |
| `intensity` | EP-00 triage | During initial triage, re-scored on milestones | `witness` record |
| `pull_quote` | EP-07 summary | After sufficient conversation (5+ messages) | `witness` record |
| `body` | EP-07 summary | After sufficient conversation, re-generated periodically | `witness` record |

**EP-00 triage output schema extension:**

The current EP-00 output includes `entry_flow`, `track`, `seed_type`, `context_bar_state`. Extend with engagement fields:

```json
{
  "output": {
    "entry_flow": "community",
    "track": "tuntaskan",
    "seed_type": "masalah",
    "context_bar_state": "...",
    "hook_line": "Sudah 2 minggu, gelap total.",
    "sentiment": "angry",
    "intensity": 4
  }
}
```

**New fields in EP-00 output:**

| Field | Type | Required | Description |
|---|---|---|---|
| `hook_line` | `string` | no | Punchy 1-liner (max 80 chars) for the feed card headline |
| `sentiment` | `enum` | no | One of: `angry`, `hopeful`, `urgent`, `celebratory`, `sad`, `curious`, `fun` |
| `intensity` | `integer` | no | Heat level 1-5 (1 = calm, 5 = intense) |

**EP-07 summary output schema extension:**

The current EP-07 output includes `summary`, `key_points`, `sentiment`, `action_items`. Extend with engagement fields:

```json
{
  "output": {
    "summary": "...",
    "key_points": ["..."],
    "pull_quote": "Anak saya harus lewat gang gelap setiap pulang les.",
    "body": "Warga Gang Melati mengeluh lampu jalan padam total sejak dua minggu lalu. Ibu-ibu takut pulang malam dari pasar.",
    "sentiment": "positive"
  }
}
```

**New fields in EP-07 output:**

| Field | Type | Required | Description |
|---|---|---|---|
| `pull_quote` | `string` | no | Most emotionally resonant sentence (max 140 chars) |
| `body` | `string` | no | AI-summarized narrative, 2-4 sentences, civility-filtered |

**Sentiment mapping note:** EP-00 uses a 7-value engagement sentiment enum (`angry`, `hopeful`, etc.) while EP-07 uses a 4-value summary sentiment (`positive`, `neutral`, `negative`, `mixed`). These serve different purposes:
- EP-00 engagement sentiment drives visual styling (shadow color, card mood).
- EP-07 summary sentiment is analytical.
- Both are stored. The frontend uses the EP-00 engagement sentiment for display.

**Processing flow:**

1. User submits seed via Bagikan flow.
2. EP-00 triage runs. Output includes `hook_line`, `sentiment`, `intensity`.
3. Fields are persisted on the `witness` record.
4. After 5+ messages accumulate in the witness conversation, EP-07 summary runs.
5. Output includes `pull_quote` and `body`.
6. Fields are persisted on the `witness` record (upsert).
7. EP-07 re-runs periodically (every 10 new messages or 24 hours, whichever comes first) to keep `body` and `pull_quote` current.

**Witness record additions:**

```sql
DEFINE FIELD hook_line  ON TABLE witness TYPE option<string>;
DEFINE FIELD pull_quote ON TABLE witness TYPE option<string>;
DEFINE FIELD sentiment  ON TABLE witness TYPE option<string>;
DEFINE FIELD intensity  ON TABLE witness TYPE option<int>;
DEFINE FIELD body       ON TABLE witness TYPE option<string>;
```

**TODO:** Define the prompt additions for EP-00 and EP-07 to generate these fields. Consider whether `intensity` should be re-scored when the witness crosses milestone thresholds (e.g., 10+ participants, evidence submitted, vote opened).

---

### 2D. Cover Image Resolution (`cover_url`)

<!-- STATUS: 2D_COVER_IMAGE — NEEDS_DISCUSSION -->

**Purpose:** Provide a representative image for the feed card. Powers the edge-to-edge cover photo at the top of rich media cards.

**Image source priority (highest to lowest):**

| Priority | Source | When Available |
|---|---|---|
| 1 | Most recent evidence photo | After evidence submission (event_type `evidence`) |
| 2 | Seed media attachment | During initial triage (if user attached photo) |
| 3 | Location-based photo | If witness has geo-coordinates (OSM/map tile) |
| 4 | None | Card renders without cover image |

**Selection logic:**

```
IF witness has evidence attachments with image MIME type:
    cover_url = most_recent_evidence_image.cdn_url
ELSE IF seed has media_urls with image MIME type:
    cover_url = seed.media_urls[0].cdn_url
ELSE:
    cover_url = null
```

**Storage:**

`cover_url` is stored on the `witness` record and updated when new evidence images are submitted. It is a denormalized field for feed performance -- the canonical evidence data lives in the evidence/attachment records.

```sql
DEFINE FIELD cover_url ON TABLE witness TYPE option<string>;
```

**CDN considerations:**
- All user-uploaded images pass through EP-08 sensitive media scan before being served.
- `cover_url` points to the CDN-served, moderation-cleared variant.
- Thumbnail generation: 600px wide, quality 80, WebP format for feed cards.
- Original resolution preserved in evidence records for detail view.

**DISCUSSION NEEDED:** Should we support auto-generated cover images from location data (map tiles, street view) when no user photos exist? This would increase visual density in the feed but adds complexity.

---

## 3. Feed API Enrichment

<!-- STATUS: 3_FEED_ENRICHMENT — NOT_STARTED -->

The existing `GET /api/feed` endpoint (defined in [FEED-API-CONTRACT-v0.1.md](./FEED-API-CONTRACT-v0.1.md)) returns `FeedItem[]`. The engagement fields must be added to the response shape.

### 3.1 Response Shape Changes

The `FeedItem` response object gains these optional fields:

```typescript
// ── LLM-enriched (cached on witness record) ──────────────
hook_line?: string;
pull_quote?: string;
sentiment?: 'angry' | 'hopeful' | 'urgent' | 'celebratory' | 'sad' | 'curious' | 'fun';
intensity?: number;      // 1-5

// ── Rich media ───────────────────────────────────────────
cover_url?: string;
body?: string;

// ── Engagement: Pulse & Urgency ──────────────────────────
active_now?: number;
deadline?: string;        // ISO 8601
deadline_label?: string;
quorum_target?: number;
quorum_current?: number;
```

### 3.2 Feed Assembly Changes

The feed query must be extended to join/compute engagement fields:

| Field | Assembly Strategy |
|---|---|
| `hook_line`, `pull_quote`, `sentiment`, `intensity`, `body`, `cover_url` | Direct read from `witness` record (pre-cached by EP-00/EP-07) |
| `deadline`, `deadline_label`, `quorum_target` | Direct read from `witness` record (set by track state machine) |
| `active_now` | Subquery against `witness_presence` table |
| `quorum_current` | Subquery against `governance_votes` / `governance_consents` |

### 3.3 Performance Considerations

| Concern | Mitigation |
|---|---|
| `active_now` subquery per feed item | Batch query: fetch all presence counts for the feed item set in one query |
| `quorum_current` computation | Only compute for items where `quorum_target IS NOT NULL` |
| Feed response size increase | ~200 bytes per item for engagement fields; negligible at `limit=20` |

### 3.4 Caching Strategy

| Data | Cache TTL | Invalidation |
|---|---|---|
| LLM fields (`hook_line`, `body`, etc.) | Long (until next EP-00/EP-07 run) | On triage re-run or summary update |
| `active_now` | 30 seconds | Per heartbeat cycle |
| `deadline` / `quorum_*` | Until governance action closes | On `track_state_transition` event |
| `cover_url` | Until new evidence submitted | On evidence attachment event |

---

## 4. Feature-Contract Map Additions

New entries for the [feature-contract-map.md](../../research/feature-contract-map.md), following its exact format.

<!-- STATUS: 4_FEATURE_MAP — PROPOSED -->

| Feature ID | Backend capability | Contract source(s) | Request/Output contract | Trigger | Ownership | Status |
|---|---|---|---|---|---|---|
| UI-24 | Presence heartbeat and active_now count | `ENGAGEMENT-BACKEND-CONTRACT-v0.1.md` 2A | `POST /api/presence/heartbeat` with `witness_id`, `session_id`; feed response `active_now` integer | Client viewport intersection / detail open; 30s heartbeat interval | app-only | UNKNOWN |
| UI-25 | Deadline and quorum projection onto feed items | `ENGAGEMENT-BACKEND-CONTRACT-v0.1.md` 2B, `13-transitions-and-quorum.md` | `deadline` (ISO), `deadline_label` (string), `quorum_target` (int) written on `track_state_transition`; `quorum_current` computed at query time | Governance action open (vote/consent) | app-only | PARTIAL |
| UI-26 | LLM engagement field extraction (hook_line, sentiment, intensity) | `ENGAGEMENT-BACKEND-CONTRACT-v0.1.md` 2C, `edgepod-endpoint-contracts.md` EP-00 | EP-00 output extended with `hook_line`, `sentiment`, `intensity`; persisted on `witness` record | EP-00 triage execution | shared (EP-00 + app) | UNKNOWN |
| UI-27 | LLM narrative generation (pull_quote, body) | `ENGAGEMENT-BACKEND-CONTRACT-v0.1.md` 2C, `edgepod-endpoint-contracts.md` EP-07 | EP-07 output extended with `pull_quote`, `body`; persisted on `witness` record | EP-07 summary execution (5+ messages threshold) | shared (EP-07 + app) | UNKNOWN |
| UI-28 | Cover image resolution and CDN serving | `ENGAGEMENT-BACKEND-CONTRACT-v0.1.md` 2D | `cover_url` on `witness` record; updated on evidence submission; CDN thumbnail 600px WebP | Evidence attachment, seed media upload | app-only | UNKNOWN |

---

## 5. SurrealDB Schema Additions

All schema additions for engagement features, consolidated.

<!-- STATUS: 5_SCHEMA — PROPOSED -->

### 5.1 New Table: `witness_presence`

```sql
-- Tracks active users per witness for pulse glow and "N aktif" badge.
-- Rows expire after 60s of no heartbeat. GC runs every 5 minutes.

DEFINE TABLE witness_presence SCHEMAFULL;

DEFINE FIELD witness_id ON TABLE witness_presence TYPE string;
DEFINE FIELD session_id ON TABLE witness_presence TYPE string;
DEFINE FIELD last_seen  ON TABLE witness_presence TYPE datetime;

DEFINE INDEX idx_presence_witness
  ON TABLE witness_presence FIELDS witness_id;

DEFINE INDEX uniq_presence_session
  ON TABLE witness_presence FIELDS witness_id, session_id UNIQUE;
```

### 5.2 New Fields on `witness` Table

```sql
-- LLM engagement fields (populated by EP-00 triage)
DEFINE FIELD hook_line  ON TABLE witness TYPE option<string>;
DEFINE FIELD sentiment  ON TABLE witness TYPE option<string>;
DEFINE FIELD intensity  ON TABLE witness TYPE option<int>;

-- LLM narrative fields (populated by EP-07 summary)
DEFINE FIELD pull_quote ON TABLE witness TYPE option<string>;
DEFINE FIELD body       ON TABLE witness TYPE option<string>;

-- Rich media
DEFINE FIELD cover_url  ON TABLE witness TYPE option<string>;

-- Deadline & quorum (populated by track state machine on governance actions)
DEFINE FIELD deadline       ON TABLE witness TYPE option<datetime>;
DEFINE FIELD deadline_label  ON TABLE witness TYPE option<string>;
DEFINE FIELD quorum_target   ON TABLE witness TYPE option<int>;
```

**Note:** `quorum_current` is not stored. It is computed at query time from `governance_votes` / `governance_consents` tables.

### 5.3 Index Additions

```sql
-- Speed up feed assembly: filter witnesses with active deadlines
DEFINE INDEX idx_witness_deadline
  ON TABLE witness FIELDS deadline
  WHERE deadline IS NOT NULL;

-- Speed up presence count batch query during feed assembly
DEFINE INDEX idx_presence_active
  ON TABLE witness_presence FIELDS witness_id, last_seen;
```

---

## 6. Architecture Decisions Log

| ID | Decision | Date | Status | Notes |
|---|---|---|---|---|
| ENG-D01 | Use SurrealDB heartbeat rows for presence (Option A1) over dedicated WebSocket | 2026-02-19 | proposed | Simplest path for Phase 1. Revisit if latency requirements tighten. |
| ENG-D02 | `quorum_current` computed at query time, not cached | 2026-02-19 | proposed | Consistency over speed. Cache with write-through trigger if feed assembly latency exceeds 200ms. |
| ENG-D03 | Extend EP-00 output schema with `hook_line`, `sentiment`, `intensity` | 2026-02-19 | proposed | Requires Edge-Pod prompt update. No breaking change -- new optional fields. |
| ENG-D04 | Extend EP-07 output schema with `pull_quote`, `body` | 2026-02-19 | proposed | Requires Edge-Pod prompt update. No breaking change -- new optional fields. |
| ENG-D05 | EP-07 re-run trigger: every 10 messages or 24 hours | 2026-02-19 | proposed | Balances freshness vs. LLM cost. TODO: confirm cost model. |
| ENG-D06 | Cover image: most-recent evidence photo wins, no auto-generated map tiles | 2026-02-19 | proposed | Keep it simple for Phase 1. Map tile covers can be a Phase 2+ enhancement. |
| ENG-D07 | Presence data never persisted beyond 60s TTL | 2026-02-19 | proposed | Privacy-first. No historical presence analytics. |
| ENG-D08 | Engagement sentiment (7-value) is separate from summary sentiment (4-value) | 2026-02-19 | proposed | Different purposes: visual styling vs. analytical summary. Both stored, frontend uses engagement sentiment. |
| ENG-D09 | Feed response adds ~200 bytes/item for engagement fields | 2026-02-19 | accepted | Negligible impact at `limit=20` (4KB total). No pagination changes needed. |
| ENG-D10 | `active_now` in feed response refreshed per request, not pushed | 2026-02-19 | proposed | Phase 1: poll on feed refresh. Phase 2+: LIVE SELECT push for detail view. |

---

## 7. Phase Mapping

How this contract maps to the engagement strategy phases.

| Phase | Strategy Feature | Contract Sections | Backend Work |
|---|---|---|---|
| **Phase 1: Feel Alive** | 4.2 Pulse Ring + 4.4 Countdown | 2A, 2B, 2C (EP-00 fields), 2D, 3, 5 | Presence service, deadline projection, EP-00 extension, cover_url, feed enrichment |
| **Phase 2: Quick Dopamine** | 4.3 Quick Strike | (future contract needed) | Reaction storage, inline vote API |
| **Phase 3: Show Life** | 4.1 Story Peek | 2C (EP-07 fields) | EP-07 extension, message preview subscription |
| **Phase 4: Ownership** | 4.5 Streak & Impact | (future contract needed) | `user_daily_activity` tracking, streak computation |
| **Phase 5: The Shift** | 4.6 River Mode | (no backend changes) | Frontend-only view mode |

---

## 8. Open Questions

Tracked items that need resolution before implementation.

| # | Question | Relevant Section | Owner | Status |
|---|---|---|---|---|
| Q1 | Should `active_now` update in real-time via LIVE SELECT on the feed, or only on feed refresh? | 2A | Backend | DISCUSSION NEEDED |
| Q2 | Should `quorum_current` be cached on witness record with trigger-based update? | 2B | Backend | DISCUSSION NEEDED |
| Q3 | What is the cost model for EP-07 re-runs (every 10 messages)? Is this sustainable at scale? | 2C | AI/Platform | DISCUSSION NEEDED |
| Q4 | Should auto-generated cover images (map tiles) be supported when no user photos exist? | 2D | Product | DISCUSSION NEEDED |
| Q5 | How should `intensity` re-scoring work? On which milestone events? | 2C | AI/Product | DISCUSSION NEEDED |
| Q6 | What is the privacy review status for the presence heartbeat design? | 2A | Privacy | TODO |
| Q7 | Should the `witness` table be the canonical store for engagement fields, or a separate `witness_engagement` table? | 5.2 | Backend | DISCUSSION NEEDED |

---

## Changelog

| Version | Date | Change |
|---|---|---|
| 0.1 | 2026-02-19 | Initial draft. Field registry, 4 data source contracts, feed enrichment spec, feature-contract map additions, SurrealDB schema, 10 architecture decisions. |
