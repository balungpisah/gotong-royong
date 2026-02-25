# Full Gotong-Tandang Integration (Transparent)

## Goals

- Make Tandang the reputation, CV hidup, skills, and PoR engine for Gotong.
- Keep Tandang usable by other platforms without Gotong-specific coupling.
- Keep the user experience fully transparent (no manual linking).
- Preserve strong security and operational reliability.

## System Responsibilities

- Gotong Royong (source of truth):
  - Tasks, contributions, evidence storage, and gameplay state.
  - Emission of trusted webhook events to Tandang.
- Tandang (source of truth):
  - Reputation, tiering, activity timeline, CV hidup, skills graph, PoR status.
  - Leaderboards and reputation distributions.

## Identity and Linking (Transparent)

- Identity format: `gotong_royong:{user_id}`.
- Gotong is configured as a trusted platform in Tandang.
- On first webhook for a new user, Tandang auto-creates:
  - Markov user identity.
  - Verified account link for `gotong_royong:{user_id}`.
- No user action is required in Gotong to link accounts.
- Other platforms continue to use verification-based account linking.

## Authentication Model

- Webhook writes: HMAC-SHA256 signature with shared secret.
- Read APIs: platform service token (server-to-server).
- End-user JWT access remains supported for direct user-facing Tandang apps.

## Write Path (Gotong → Tandang)

### Event Types

- `contribution_created`
- `vouch_submitted`
- `por_evidence`

### Required Fields

- `event_id` (idempotency)
- `schema_version` (required contract version; positive integer or numeric string)
- `request_id` (required trace ID; must match `X-Request-ID`)
- `source_platform_id` (required for platform-partitioned aggregation; default `gotong_royong`)
- `event_type`
- `actor.user_id`
- `subject` (event-specific)

### Required Headers

- `X-GR-Signature: sha256=<hex>`
- `X-Request-ID: <request_id>` (must equal payload `request_id`)

### Delivery Guarantees

- Outbox + retries with exponential backoff.
- Idempotent ingestion in Tandang by `event_id`.
- Duplicate `event_id` is handled as `200` with ignored/no-op result.
- Dead-letter queue for manual replay.

## Read Path (Gotong ← Tandang)

Canonical endpoint mapping lives in:
- `docs/architecture/tandang-endpoint-map.md`

### Core Endpoints (summary)

- Reputation: `GET /api/v1/users/{id}/reputation`
- Tier: `GET /api/v1/users/{id}/tier`
- Activity: `GET /api/v1/users/{id}/activity`
- CV hidup: `GET /api/v1/cv-hidup/{user_id}`
- Skills: `GET /api/v1/skills/search`, `GET /api/v1/skills/nodes/{id}` (+ labels/relations/parent)
- PoR requirements: `GET /api/v1/por/requirements/{task_type}`, `GET /api/v1/por/triad-requirements/{track}/{transition}`
- PoR status: `GET /api/v1/por/status/{evidence_id}`
- Leaderboards: `GET /api/v1/reputation/leaderboard`
- Distribution: `GET /api/v1/reputation/distribution`

Notes:
- Gotong should treat `/api/v1/users/{id}/...` `{id}` as either:
  - Markov UUID, or
  - platform identity string like `gotong_royong:{gotong_user_id}` (preferred when calling on behalf of a Gotong user).
- Trusted platform-token reads/writes needed by Gotong now exist as user-keyed variants (see `docs/architecture/tandang-endpoint-map.md`), so Gotong does not need to “bridge” user JWTs for core UX.

### Suggested Gotong UX Mapping

| Gotong UX Surface | Tandang Endpoint(s) |
| --- | --- |
| Profile header (reputation/tier) | `/api/v1/users/{id}/reputation`, `/api/v1/users/{id}/tier` |
| Activity feed | `/api/v1/users/{id}/activity` |
| Live CV panel | `/api/v1/cv-hidup/{user_id}` |
| Share CV | `POST /api/v1/cv-hidup/{user_id}/export` + `GET /api/v1/cv-hidup/{user_id}/qr` (platform-token supported) |
| Skills badge/suggestions | `/api/v1/skills/search` |
| PoR guidance | `/api/v1/por/requirements/{task_type}`, `/api/v1/por/triad-requirements/{track}/{transition}` |
| Leaderboards | `/api/v1/reputation/leaderboard` |

Gameplay gates, rewards, and badge rules are specified in:
- `docs/architecture/tandang-gameplay-rules.md`

## Backfill and Replay

- Use `POST /api/v1/platforms/gotong_royong/ingest` for historical data.
- Use `POST /api/v1/platforms/gotong_royong/poll-ingest` for periodic polling.
- Maintain replay tooling for DLQ and backfills.

## Failure Handling

- Webhook failures: retry with backoff, then DLQ.
- Read failures: fall back to cached data and mark stale.
- Tandang outages: queue events locally for later delivery.

## Compatibility With Other Platforms

- Trusted-platform auto-linking is opt-in per platform.
- All other platforms retain verification-based linking.
- Gotong-specific changes must be implemented as generic primitives.

## Security and Privacy

- Rotate webhook secrets and platform tokens without downtime.
- Audit log platform reads and failed auth.
- Limit platform tokens to `gotong_royong` scope.

## Cross-Platform Identity + Reputation Scoping (Roadmap)

Tandang should support a hybrid identity model:
- Same person → one canonical Markov UUID → multiple verified links (platform identities).
- Default: platform-scoped reputation *views* for platform service tokens.
- Optional: unified cross-platform reputation view (consent + policy gated).

### Phase 1 (Do now): Auth scoping — block cross-platform queries

Threat model: a valid `gotong_royong` platform token must not be able to read arbitrary Markov UUIDs that are not linked to Gotong.

Required behavior in Tandang (for all platform-token user reads):
- Resolve `{id}` → canonical Markov UUID.
- If auth is a platform token: require the resolved user to be linked to `claims.platform_id` (e.g. `gotong_royong`) or return `403`.

This is defense-in-depth on top of Gotong’s own proxy restrictions.

### Phase 2 (Before adding a 2nd platform): Data scoping — partition by source platform

Goal: prevent “reputation parasitism” where a weak platform’s token can see (or benefit from) contributions from a stronger platform by default.

Required behavior:
- Every stored reputation signal/event carries `source_platform_id`.
- Precompute and serve per-platform aggregates by default:
  - Platform token → platform-scoped aggregates only.
  - User JWT (self) + consent → unified aggregates.
  - Admin/system → full canonical view.
- Optional explicit handshake (recommended for rollout clarity):
  - Gotong can send `view_scope=platform` and `platform_id=gotong_royong` on reputation-family reads.
  - Keep this disabled until Tandang handlers accept/query these parameters, then enable for cutover validation.

### Phase 3 (Product decision): Consent + policy gates for unified view

Unified cross-platform aggregation is allowed only when:
- User explicitly opts in (consent flag), and
- Platform policy allows cross-platform reads, and
- Any additional governance rules (cooldowns, review, thresholds) pass.
