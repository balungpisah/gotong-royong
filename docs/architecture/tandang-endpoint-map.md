# Gotong ↔ Tandang (Markov Engine) — Endpoint Map (Authoritative)

Purpose: a single reference for **how Gotong Royong should use Tandang endpoints** (read + write), and where contracts are currently mismatched.

Sources of truth:
- Gotong read client: `crates/infra/src/markov_client.rs`
- Gotong proxy routes: `crates/api/src/routes/mod.rs` (`/v1/tandang/*`)
- Tandang router/middleware: `tandang/markov-engine/crates/api/src/routes/mod.rs` (`/api/v1/*`)
- OpenAPI inventory (generated): `docs/architecture/tandang-endpoints-openapi-inventory.md`
- Refresh OpenAPI inventory: `scripts/docs/generate_tandang_openapi_inventory.py`

---

## Base URLs and Auth

### Tandang API base

- Base: `{MARKOV_READ_BASE_URL}` (default `http://127.0.0.1:3000/api/v1`)

### Gotong proxy base (frontend-facing)

- Base: `https://<gotong-host>/v1/tandang/*`

### Auth schemes

- **Platform service token** (server-to-server): `X-Platform-Token: <token>`
  - Used by `MarkovReadClient` for Gotong reads.
- **User JWT**: `Authorization: Bearer <jwt>`
  - Some Tandang endpoints are **JWT-only** today.
- **Webhook signature**: `X-GR-Signature: sha256=<hex>`
  - Used for Gotong → Tandang writes (webhook ingestion).

### Identity rule (critical)

Gotong’s canonical identity in Tandang is:

- `gotong_royong:{gotong_user_id}` (see `MarkovReadClient::platform_identity`)

This identity is passed into Tandang `GET /api/v1/users/{id}/reputation` as `{id}` (where `{id}` may be a Markov UUID or a platform-scoped identity string).

### ID formats (important)

Tandang endpoints use two different user identifier shapes:

- **Markov user ID**: a UUID string (example: `6d2f6d2d-6a3a-4c47-9e4e-3f4d7c6f9d2b`)
- **Platform-scoped identity**: `{platform_id}:{platform_user_id}` (example: `gotong_royong:user123`)

For Gotong’s user-keyed trusted reads (`/api/v1/users/{id}/vouch-budget`, `/api/v1/users/{id}/decay/warnings`, `/api/v1/cv-hidup/{user_id}/qr`, `/api/v1/cv-hidup/{user_id}/export`), Tandang accepts **either** form and resolves platform identities via account links.

### Phase-2 query handshake (read-side readiness)

For explicit rollout visibility, Gotong can append these query params on reputation-family reads:
- `view_scope=platform`
- `platform_id=gotong_royong`

This handshake is optional and should stay disabled until Tandang read handlers accept it.

---

## A) Gotong → Tandang (Write Paths)

| Domain | Endpoint | Auth | Gotong usage proposal | Notes |
|---|---|---|---|---|
| Platforms | `POST /api/v1/platforms/gotong_royong/webhook` | `X-GR-Signature` | Primary ingestion lane for reputation/skills/PoR signals; must be idempotent by `event_id`. | Spec: `docs/api/webhook-spec.md` + `docs/api/event-payloads.md`. |
| Platforms | `POST /api/v1/platforms/gotong_royong/ingest` | JWT (per OpenAPI) | Operator/admin-only backfill lane; use for DLQ replay or historical import. | If we want “Gotong system backfill” without human JWTs, add a platform-token write lane (proposed). |
| Platforms | `POST /api/v1/platforms/gotong_royong/poll-ingest` | `X-Bot-Token` (per OpenAPI) | Likely unnecessary for Native integration; keep as emergency fallback for reconciliation. | Verify if needed for Gotong at all. |

---

## B) Gotong ← Tandang (Read Paths)

These are the endpoints Gotong either calls directly via `MarkovReadClient` or exposes to clients via `/v1/tandang/*` proxy routes.

### B.1 Profile snapshot (Gotong-composed)

| Domain | Gotong endpoint | Upstream Tandang calls (current Gotong behavior) | Auth | Cache class | Gotong usage proposal |
|---|---|---|---|---|---|
| Users/CV | `GET /v1/tandang/me/profile` | 1) `GET /api/v1/users/{gotong_royong:user_id}/reputation` → derive `markov_user_id` from payload<br/>2) `GET /api/v1/users/{markov_user_id}/tier`<br/>3) `GET /api/v1/users/{markov_user_id}/activity`<br/>4) `GET /api/v1/cv-hidup/{markov_user_id}` | Platform token | Profile | **Primary “profile header” read**: tier badge, I/C/J axes, trend line, and the activity timeline slice. Keep UI resilient: show stale marker if cache is stale. |

Game-theory note:
- This is the core “trust surface”. Optimize it for **explainability and anti-gaming**: show *why* credit was earned (activity), not just totals.

### B.2 Reputation + tier + activity (primitive reads)

| Domain | Tandang endpoint | Auth in Tandang router | Cache class | Gotong usage proposal |
|---|---|---|---|---|
| Users | `GET /api/v1/users/{id}/reputation` | JWT **or** platform token | Profile | Gate high-impact actions (rate-limits, eligibility, “Pilih sendiri” unlock), compute reward multipliers, show trust breakdown. |
| Users | `GET /api/v1/users/{id}/tier` | JWT **or** platform token | Profile | Drive feature unlocks (vouching, verification roles, governance). |
| Users | `GET /api/v1/users/{id}/activity` | JWT **or** platform token | Profile | “Proof of contribution” timeline; anti-sybil UX: recent actions, streaks, witness confirmations. |

### B.3 Skills

| Domain | Tandang endpoint | Auth in Tandang router | Cache class | Gotong usage proposal |
|---|---|---|---|---|
| Skills | `GET /api/v1/skills/search` | JWT **or** platform token | Gameplay | Autocomplete for skills; task-to-skill matching; onboarding “follow skills” suggestions. |
| Skills | `GET /api/v1/skills/nodes/{id}` | JWT **or** platform token | Gameplay | Skill detail (label, hierarchy node). |
| Skills | `GET /api/v1/skills/nodes/{id}/labels` | JWT **or** platform token | Gameplay | Localization for UI display + ranking. |
| Skills | `GET /api/v1/skills/nodes/{id}/relations` | JWT **or** platform token | Gameplay | Enrichment only (background or detail pages), not feed assembly. |
| Skills | `GET /api/v1/skills/{id}/parent` | JWT **or** platform token | Gameplay | Breadcrumbs in skill UI + clustering. |

### B.4 PoR (Proof of Reality)

| Domain | Tandang endpoint | Auth in Tandang router | Cache class | Gotong usage proposal | Status |
|---|---|---|---|---|---|
| PoR | `GET /api/v1/por/requirements/{task_type}` | JWT **or** platform token | Gameplay | Drive PoR checklist UI for each task type (anti-fraud friction that still feels helpful). | OK |
| PoR | `GET /api/v1/por/triad-requirements/{track}/{transition}` | JWT **or** platform token | Gameplay | Gate phase transitions deterministically (no “soft” bypass), with clear “missing items” UX. | OK |
| PoR | `GET /api/v1/por/status/{evidence_id}` | JWT **or** platform token | Gameplay | Needed to finalize rewards and show verification state. | OK |

### B.5 Reputation aggregates (leaderboards/distribution)

| Domain | Tandang endpoint | Auth in Tandang router | Cache class | Gotong usage proposal |
|---|---|---|---|---|
| Reputation | `GET /api/v1/reputation/leaderboard` | JWT **or** platform token | Gameplay | Use carefully: prefer *local/community leaderboards* and “personal progress” views to avoid pure vanity loops. |
| Reputation | `GET /api/v1/reputation/distribution` | JWT **or** platform token | Gameplay | Calibrate expectations (“where am I in the curve”), and detect anomalies (anti-gaming monitoring). |

### B.6 Slash / GDF weather

| Domain | Tandang endpoint | Auth in Tandang router | Cache class | Gotong usage proposal |
|---|---|---|---|---|
| Slash | `GET /api/v1/slash/gdf` | Public (optional auth) | Gameplay | “Community difficulty floor” signal to adjust incentives: discourage farming when community is under stress; inform prioritization. |

### B.7 Community pulse and heroes (optional surfaces)

| Domain | Tandang endpoint | Auth in Tandang router | Cache class | Gotong usage proposal |
|---|---|---|---|---|
| Community Pulse | `GET /api/v1/community/pulse/overview` | Public (optional auth) | Gameplay | Community health dashboard (aggregate only; no user-level leakage). |
| Community Pulse | `GET /api/v1/community/pulse/insights` | Public (optional auth) | Gameplay | “What’s changing” summaries; helps keep feed relevant without heavy computation on Gotong. |
| Community Pulse | `GET /api/v1/community/pulse/trends` | Public (optional auth) | Gameplay | Trend lines to drive gentle nudges (not punishments). |
| Weather Heroes | `GET /api/v1/hero/{user_id}` | Public (optional auth) | Gameplay | “Hero status” widget; can reward consistent help without tying directly to raw post volume. |
| Weather Heroes | `GET /api/v1/hero/leaderboard` | Public (optional auth) | Gameplay | Optional leaderboard; show with anti-toxicity guardrails (bands, local scope, cooldown). |

### B.8 CV Hidup sharing

| Domain | Tandang endpoint | Auth in Tandang router | Gotong usage proposal | Status |
|---|---|---|---|---|
| CV Hidup | `POST /api/v1/cv-hidup/{user_id}/export` | JWT **or** platform token (platform token requires `write` scope) | Generate export artifacts for share sheet (C6) without user-JWT bridging. | OK |
| CV Hidup | `GET /api/v1/cv-hidup/{user_id}/qr` | JWT **or** platform token | QR for the user’s live CV share flow (server-to-server). | OK |
| CV Hidup | `POST /api/v1/cv-hidup/export` | JWT-only | Legacy/direct-Markov export (not used by Gotong). | OK (legacy) |
| CV Hidup | `GET /api/v1/cv-hidup/qr` | JWT-only | Legacy/direct-Markov QR (not used by Gotong). | OK (legacy) |
| CV Hidup | `GET /api/v1/cv-hidup/verify/{export_id}` | Public | Verification landing page for shared CV links. | OK |
| CV Hidup | `GET /api/v1/cv-hidup/qr/{export_id}` | Public | QR for a specific exported CV. | OK |

---

## C) Contract Mismatches (Resolved)

These were places where Gotong’s read client and/or proxy endpoints didn’t match Tandang’s router behavior. They are now resolved with trusted (platform-token) reads and user-keyed endpoints where needed.

| Area | Gotong expects (today) | Tandang actually exposes (today) | Impact | Proposed direction |
|---|---|---|---|---|
| PoR status | `GET /api/v1/por/status/{evidence_id}` via platform token | `GET /api/v1/por/status/{evidence_id}` now accepts JWT **or** platform token (platform-scoped). | Reward finalization + verification UI can rely on Tandang as authority. | Use the same endpoint with platform token. |
| CV sharing | `POST /api/v1/cv-hidup/export` + `GET /api/v1/cv-hidup/qr` via platform token | Added user-keyed variants that accept JWT **or** platform token. | Gotong can generate share artifacts without user-JWT bridging. | Use `POST /api/v1/cv-hidup/{user_id}/export` and `GET /api/v1/cv-hidup/{user_id}/qr` for platform-token calls. |
| Vouch budget | `GET /api/v1/users/{id}/vouch-budget` | Added `GET /api/v1/users/{id}/vouch-budget` (JWT self-only or platform-scoped). | Gotong can show vouch budget and gate vouch actions deterministically. | Use `GET /api/v1/users/{id}/vouch-budget`. |
| Decay warnings | `GET /api/v1/users/{id}/decay/warnings` | Added `GET /api/v1/users/{id}/decay/warnings` (JWT self-only or platform-scoped). | Gotong can surface decay risk without user-JWT bridging. | Use `GET /api/v1/users/{id}/decay/warnings`. |

Tracking checklist: `docs/architecture/tandang-integration-gap-tracker.md`

---

## D) Endpoint Additions (Implemented)

These were added to make incentives enforceable and reads fast without requiring Gotong to “act as the user” with a Markov JWT.

| Endpoint (Tandang) | Why it helps | Expected auth |
|---|---|---|
| `GET /api/v1/users/{id}/vouch-budget` | Enables deterministic vouch gating in Gotong UI (prevents vouch spam/farming) while keeping Tandang as the authority for budget policy. | Platform token (trusted read scoped to platform-linked users) |
| `GET /api/v1/users/{id}/decay/warnings` | Enables “decay risk” UX (retention + anti-gaming) without requiring user JWT bridging. | Platform token (trusted read scoped to platform-linked users) |
| `GET /api/v1/por/status/{evidence_id}` | Needed for reward finalization and verification UI when PoR authority is Tandang. | Platform token (trusted read scoped to platform-linked submitters) |
| `POST /api/v1/cv-hidup/{id}/export` | Lets Gotong create share artifacts (export IDs) while keeping Tandang’s signature/verification semantics. | Platform token (trusted write scoped to platform-linked users) |
| `GET /api/v1/cv-hidup/{id}/qr` | Lets Gotong fetch a live-CV QR artifact without user JWT. | Platform token (trusted read scoped to platform-linked users) |
| `GET /api/v1/users/snapshot` (batch by `{id}` list) | Cuts roundtrips for “member list / profile chips” UIs (tier + key rep stats), without giving Gotong extra authority. | Platform token (trusted read; rate-limited) |
