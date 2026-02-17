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

### Core Endpoints

- Reputation: `GET /api/v1/users/{id}/reputation`
- Tier: `GET /api/v1/users/{id}/tier`
- Activity: `GET /api/v1/users/{id}/activity`
- CV hidup: `GET /api/v1/cv-hidup/{user_id}`
- CV export/QR: `GET /api/v1/cv-hidup/{user_id}/export`, `GET /api/v1/cv-hidup/{user_id}/qr`
- Skills: `GET /api/v1/skills/search`, `GET /api/v1/skills/nodes`
- PoR requirements: `GET /api/v1/por/requirements`, `GET /api/v1/por/triad-requirements`
- PoR status: `GET /api/v1/por/status?user_id=...`
- Leaderboards: `GET /api/v1/reputation/leaderboard`
- Distribution: `GET /api/v1/reputation/distribution`

### Suggested Gotong UX Mapping

| Gotong UX Surface | Tandang Endpoint(s) |
| --- | --- |
| Profile header (reputation/tier) | `/api/v1/users/{id}/reputation`, `/api/v1/users/{id}/tier` |
| Activity feed | `/api/v1/users/{id}/activity` |
| Live CV panel | `/api/v1/cv-hidup/{user_id}` |
| Share CV | `/api/v1/cv-hidup/{user_id}/export`, `/api/v1/cv-hidup/{user_id}/qr` |
| Skills badge/suggestions | `/api/v1/skills/search` |
| PoR guidance | `/api/v1/por/requirements`, `/api/v1/por/triad-requirements` |
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
