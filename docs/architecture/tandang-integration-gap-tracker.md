# Tandang Integration Gap Tracker (Gotong)

Purpose: a checklist for closing **contract mismatches** between Gotong and Tandang before implementation hardening.

Primary reference:
- Endpoint map: `docs/architecture/tandang-endpoint-map.md`

Related research:
- Gap log: `docs/research/logs/tandang-gap-log.md`

---

## 1) Read API mismatches (blocking)

### PoR status (reward finalization)

- [x] **Confirm desired contract**: Gotong needs to read PoR status by `evidence_id` using **platform token**.
- [x] **Tandang change**: allow platform token on `GET /api/v1/por/status/{evidence_id}` (move to `auth_or_platform_service_middleware`).
- [x] **OpenAPI alignment**: update security for `/api/v1/por/status/{evidence_id}` to include `platform_service_token`.
- [ ] **Gotong alignment**: keep `/v1/tandang/por/status/:evidence_id` stable; verify it works against live Tandang.

### CV Hidup share artifacts

- [x] Decide: **Tandang-owned exports** vs **Gotong-owned exports**.
  - Option A (Tandang-owned): add trusted platform-token endpoint to create exports (e.g., `POST /api/v1/cv-hidup/{user_id}/export`) + possibly `GET /api/v1/cv-hidup/{user_id}/qr`.
  - Option B (Gotong-owned): Gotong generates QR + share payloads from `GET /api/v1/cv-hidup/{user_id}`; Tandang only provides CV content.
- [x] If Option A: add/verify non-leaky access controls (self-or-admin equivalent via platform identity resolution).
- [x] If Option B: remove/adjust Gotong proxy endpoints `/v1/tandang/cv-hidup/export` and `/v1/tandang/cv-hidup/qr` accordingly.

### Vouch budget (gameplay gating)

- [x] Confirm desired contract: does Gotong need budgets for **self only**, or for **viewing other users**, or for **admin tooling**?
- [x] Tandang currently exposes: `GET /api/v1/vouches/budget` (JWT-only, self).
- [x] Choose one:
  - [x] Add trusted read endpoint keyed by `{id}` (Markov UUID or `platform:user_id`).
  - [ ] Keep self-only in Tandang, and have Gotong compute budget locally (less preferred if Tandang is the authority).
- [x] Align Gotong: `MarkovReadClient::get_vouch_budget` path and `/v1/tandang/users/:user_id/vouch-budget`.

### Decay warnings (anti-gaming + retention)

- [x] Confirm desired contract: Gotong needs decay warnings for a given user using platform token.
- [x] Tandang currently exposes: `GET /api/v1/decay/warnings` (JWT-only, self).
- [x] Choose one:
  - [x] Add trusted endpoint keyed by `{id}` and allow platform token.
  - [ ] Or keep self-only and remove the user_id param from Gotong proxy (requires user JWT bridging, likely not wanted).

---

## 2) OpenAPI contract drift (documentation + tooling)

OpenAPI currently diverges from the router for several trusted-read endpoints (auth and sometimes presence).

- [ ] Ensure OpenAPI matches Tandang router middleware for the Gotong-relevant endpoints:
  - `/api/v1/users/{id}/reputation|tier|activity`
  - `/api/v1/cv-hidup/{user_id}`
  - `/api/v1/skills/*` read endpoints
  - `/api/v1/por/requirements/*` and `/api/v1/por/triad-requirements/*`
  - `/api/v1/reputation/distribution|leaderboard`
- [ ] Ensure OpenAPI includes or intentionally omits “trusted platform read” semantics (documented).

Generated snapshot for reference:
- `docs/architecture/tandang-endpoints-openapi-inventory.md`

---

## 3) Gotong proxy surface decisions (product/API design)

Decide which Tandang reads remain exposed to clients through Gotong:

- [ ] Keep `/v1/tandang/me/profile` as the canonical profile snapshot.
- [ ] Decide if other `/v1/tandang/*` endpoints are:
  - [ ] client-exposed (stable product API), or
  - [ ] internal-only (backend integration convenience), or
  - [ ] removed in favor of Gotong-native read models.
