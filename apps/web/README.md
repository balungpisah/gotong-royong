# Gotong Web

Frontend application for Gotong Royong using SvelteKit 2 + Svelte 5.

## Foundation Guide

- Contributor guide: `../../docs/development/frontend-foundation.md`
- Environment template: `.env.example`

## Local Commands

```sh
bun install
bun run dev
bun run check
bun run lint
bun run test:unit
bun run test:e2e
bun run build
```

## Service Toggles

- `PUBLIC_GR_USE_API_NOTIFICATIONS=true` enables real notifications from `/v1/notifications`.
- `PUBLIC_GR_USE_API_FEED=true` enables real feed items from `/v1/feed`.
- `PUBLIC_GR_USE_API_CHAT=true` enables witness chat send/poll via `/v1/chat/threads/*`.
- `PUBLIC_GR_USE_API_USER=true` enables user profile reads via `/v1/auth/me` + `/v1/tandang/me/profile` + `/v1/tandang/users/:user_id/profile`.
- `PUBLIC_GR_USE_API_TRIAGE=true` enables triage session flows via `/v1/triage/sessions*`.
- `PUBLIC_GR_USE_API_SIGNAL=true` enables witness signal flows via `/v1/witnesses/:witness_id/signals*`.
- `PUBLIC_GR_USE_API_GROUP=true` enables group lifecycle flows via `/v1/groups*`.
- `GR_API_PROXY_TARGET=http://127.0.0.1:3000` proxies browser `/v1/*` calls to local backend during `bun dev` (recommended for live API wiring).
- Default (unset) is API-first; set any of these to `false` only for local dev/test mock runs.
- Production guard: runtime rejects `PUBLIC_GR_USE_API_*=false` and disables API-service mock fallback paths.
- Cutover tracker and remaining debt: `../../docs/research/frontend-service-api-cutover-backlog.md`.

## Live API Proxy Smoke

Run this when you want proof that frontend hot paths are hitting real backend APIs via Vite proxy:

```sh
GR_API_PROXY_TARGET=http://127.0.0.1:3100 \
bun run test:e2e:live-api
```

Notes:

- Requires `gotong-api` running at `GR_API_PROXY_TARGET`.
- Test bootstraps a real user via `POST /v1/auth/signup`, then validates a broader live matrix:
  - hot-path reads: `/v1/auth/me`, `/v1/feed`, `/v1/notifications`
  - triage flow: `POST /v1/triage/sessions`, `POST /v1/triage/sessions/:session_id/messages`
  - witness flow: `POST /v1/witnesses` + permalink load at `/saksi/:witness_id`
  - signal flow: `/v1/witnesses/:witness_id/signals*`
  - group flow: `/v1/groups*`
  - profile route/deep-link: `/v1/tandang/me/profile` or `/v1/tandang/users/:user_id/profile`

For deployed frontend hosts (no local Vite dev server), run:

```sh
PLAYWRIGHT_EXTERNAL_BASE_URL=https://<frontend-host> \
bun run test:e2e:live-api:external
```

## Baseline Structure

```text
src/
  routes/          # Route entries and layouts
  lib/
    api/           # API client and transport wrappers
    components/    # Shared UI components
    stores/        # Svelte stores and state modules
    types/         # Shared TypeScript contracts
    utils/         # Utility helpers
```

This structure is intentionally lightweight and will be expanded during the frontend foundation sprint.
