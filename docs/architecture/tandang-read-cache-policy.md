# Tandang Read Client Cache Policy

## Scope

This document defines the server-to-server read behavior from Gotong Royong to Tandang (Markov Engine) through `MarkovReadClient`.

## Endpoint Classes

| Class | Endpoints | TTL | Stale-While-Revalidate Window |
| --- | --- | --- | --- |
| Profile | `users/{id}/reputation`, `users/{uuid}/tier`, `users/{uuid}/activity`, `cv-hidup/{uuid}` | `MARKOV_CACHE_PROFILE_TTL_MS` (default `300000`) | `MARKOV_CACHE_PROFILE_STALE_WHILE_REVALIDATE_MS` (default `1200000`) |
| Gameplay | `skills/*` read endpoints, `por/requirements/{task_type}`, `por/triad-requirements/{track}/{transition}`, `reputation/leaderboard`, `reputation/distribution`, `slash/gdf`, `community/pulse/*`, `hero/*` | `MARKOV_CACHE_GAMEPLAY_TTL_MS` (default `45000`) | `MARKOV_CACHE_GAMEPLAY_STALE_WHILE_REVALIDATE_MS` (default `180000`) |

## Request and Auth

- Base URL: `MARKOV_READ_BASE_URL` (default `http://127.0.0.1:3000/api/v1`)
- Header for platform read auth: `X-Platform-Token: <token>`
- Token config: `MARKOV_READ_PLATFORM_TOKEN`

### Platform-token supported endpoints (expected)

These are the endpoints Gotong expects to read with `X-Platform-Token`:

- `users/{id}/reputation` (`{id}` can be `gotong_royong:{user_id}` or a Markov UUID)
- `users/{uuid}/tier`
- `users/{uuid}/activity`
- `cv-hidup/{uuid}`
- `cv-hidup/{id}/qr` (`{id}` can be `gotong_royong:{user_id}` or a Markov UUID)
- `cv-hidup/{id}/export` (requires platform token `write` scope; `{id}` can be `gotong_royong:{user_id}` or a Markov UUID)
- `skills/search`
- `skills/nodes/{id}`
- `skills/nodes/{id}/labels`
- `skills/nodes/{id}/relations`
- `skills/{id}/parent`
- `por/requirements/{task_type}`
- `por/triad-requirements/{track}/{transition}`
- `por/status/{evidence_id}`
- `reputation/leaderboard`
- `reputation/distribution`
- `slash/gdf`
- `users/{id}/vouch-budget` (`{id}` can be `gotong_royong:{user_id}` or a Markov UUID)
- `users/{id}/decay/warnings` (`{id}` can be `gotong_royong:{user_id}` or a Markov UUID)
- `community/pulse/overview`
- `community/pulse/insights`
- `community/pulse/trends`
- `hero/leaderboard`
- `hero/{user_id}`

### Previously-known mismatches (resolved)

These used to be called by Gotong with `X-Platform-Token` but were JWT-only in Tandang. They have been resolved via platform-token support and/or user-keyed endpoints:

- `por/status/{evidence_id}` (now supports platform token, scoped)
- `cv-hidup/export` + `cv-hidup/qr` (superseded by `cv-hidup/{id}/export` + `cv-hidup/{id}/qr`)
- `decay/warnings` (superseded by `users/{id}/decay/warnings`)
- `vouches/budget` (superseded by `users/{id}/vouch-budget`)

Authoritative mismatch tracking:
- `docs/architecture/tandang-integration-gap-tracker.md`
- `docs/architecture/tandang-endpoint-map.md`

## Reliability Controls

- Timeout: `MARKOV_READ_TIMEOUT_MS` (default `2500`)
- Retry attempts: `MARKOV_READ_RETRY_MAX_ATTEMPTS` (default `3`)
- Backoff: exponential between `MARKOV_READ_RETRY_BACKOFF_BASE_MS` and `MARKOV_READ_RETRY_BACKOFF_MAX_MS`
- Circuit breaker:
  - Open threshold: `MARKOV_READ_CIRCUIT_FAIL_THRESHOLD` transient failures
  - Open duration: `MARKOV_READ_CIRCUIT_OPEN_MS`

## Response Metadata Surface

Every cached Tandang payload returned by Gotong includes:

- `cache.status`: `hit`, `miss`, or `stale`
- `cache.stale`: boolean
- `cache.age_ms`: age of cached value
- `cache.cached_at_epoch_ms`: cache write timestamp

This metadata is exposed on:

- `GET /v1/tandang/me/profile`
- `GET /v1/tandang/users/:user_id/profile`
- `GET /v1/tandang/cv-hidup/qr`
- `POST /v1/tandang/cv-hidup/export`
- `GET /v1/tandang/cv-hidup/verify/:export_id`
- `GET /v1/tandang/skills/search`
- `GET /v1/tandang/skills/nodes/:node_id`
- `GET /v1/tandang/skills/nodes/:node_id/labels`
- `GET /v1/tandang/skills/nodes/:node_id/relations`
- `GET /v1/tandang/skills/:skill_id/parent`
- `GET /v1/tandang/por/requirements/:task_type`
- `GET /v1/tandang/por/status/:evidence_id`
- `GET /v1/tandang/por/triad-requirements/:track/:transition`
- `GET /v1/tandang/reputation/leaderboard`
- `GET /v1/tandang/reputation/distribution`
- `GET /v1/tandang/slash/gdf`
- `GET /v1/tandang/users/:user_id/vouch-budget`
- `GET /v1/tandang/decay/warnings/:user_id`
- `GET /v1/tandang/community/pulse/overview`
- `GET /v1/tandang/community/pulse/insights`
- `GET /v1/tandang/community/pulse/trends`
- `GET /v1/tandang/hero/leaderboard`
- `GET /v1/tandang/hero/:user_id`
