# Tandang Read Client Cache Policy

## Scope

This document defines the server-to-server read behavior from Gotong Royong to Tandang (Markov Engine) through `MarkovReadClient`.

## Endpoint Classes

| Class | Endpoints | TTL | Stale-While-Revalidate Window |
| --- | --- | --- | --- |
| Profile | `users/{id}/reputation`, `users/{uuid}/tier`, `users/{uuid}/activity`, `cv-hidup/{uuid}` | `MARKOV_CACHE_PROFILE_TTL_MS` (default `300000`) | `MARKOV_CACHE_PROFILE_STALE_WHILE_REVALIDATE_MS` (default `1200000`) |
| Gameplay | `skills/search`, `por/requirements/{task_type}`, `por/triad-requirements/{track}/{transition}`, `reputation/leaderboard`, `reputation/distribution` | `MARKOV_CACHE_GAMEPLAY_TTL_MS` (default `45000`) | `MARKOV_CACHE_GAMEPLAY_STALE_WHILE_REVALIDATE_MS` (default `180000`) |

## Request and Auth

- Base URL: `MARKOV_READ_BASE_URL` (default `http://127.0.0.1:3000/api/v1`)
- Header for platform read auth: `X-Platform-Token: <token>`
- Token config: `MARKOV_READ_PLATFORM_TOKEN`
- Required-token endpoints: all Gotong server-to-server Markov read endpoints (`users/{id}/reputation`, `users/{uuid}/tier`, `users/{uuid}/activity`, `cv-hidup/{uuid}`, `skills/search`, `por/requirements/{task_type}`, `por/triad-requirements/{track}/{transition}`, `reputation/leaderboard`, `reputation/distribution`)

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
- `GET /v1/tandang/skills/search`
- `GET /v1/tandang/por/requirements/:task_type`
- `GET /v1/tandang/por/triad-requirements/:track/:transition`
- `GET /v1/tandang/reputation/leaderboard`
- `GET /v1/tandang/reputation/distribution`
