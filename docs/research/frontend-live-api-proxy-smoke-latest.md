# Frontend Live API Proxy Smoke (Local, Docker DB)

Date: 2026-02-25  
Environment: local development (`gotong-royong` + `tandang/markov-engine`), Docker Desktop

## Goal

Verify frontend dev server can hit real backend APIs through Vite proxy (`GR_API_PROXY_TARGET`) against live SurrealDB and live Tandang read-path.

## Automation Update (2026-02-25)

- Added opt-in Playwright live proxy smoke: `apps/web/tests/e2e/live-api-proxy.spec.ts`.
- Runnable via:
  - `GR_API_PROXY_TARGET=http://127.0.0.1:<api-port> bun run test:e2e:live-api`
  - `just web-test-e2e-live-api target=http://127.0.0.1:<api-port>`
  - `PLAYWRIGHT_EXTERNAL_BASE_URL=https://<frontend-host> bun run test:e2e:live-api:external`
  - `just web-test-e2e-live-api-external https://<frontend-host>`
- Current assertion matrix bootstraps a real authenticated user via `POST /v1/auth/signup`, then covers live frontend host/proxy calls for hot-path reads (`/v1/auth/me`, `/v1/feed`, `/v1/notifications`), triage flow (`/v1/triage/sessions*`), witness create + permalink (`/v1/witnesses`, `/saksi/:witness_id`), signal flow (`/v1/witnesses/:witness_id/signals*`), group flow (`/v1/groups*`), and profile route/deep-link request verification (`/v1/tandang/me/profile` or `/v1/tandang/users/:user_id/profile`).
- Latest revalidation (2026-02-25): `just dev-db-up` + local `gotong-api` (`DATA_BACKEND=surrealdb`) on `:3210` + `GR_API_PROXY_TARGET=http://127.0.0.1:3210 bun run test:e2e:live-api` passed (`3 passed`).

## Commands Run

```bash
just dev-db-up

# gotong backend
APP_ENV=smoke PORT=3100 DATA_BACKEND=surrealdb \
  SURREAL_ENDPOINT=ws://127.0.0.1:8000 SURREAL_NS=gotong SURREAL_DB=chat \
  SURREAL_USER=root SURREAL_PASS=root REDIS_URL=redis://127.0.0.1:6379 \
  WEBHOOK_ENABLED=true \
  WEBHOOK_MARKOV_URL=http://127.0.0.1:3000/api/v1/platforms/gotong_royong/webhook \
  WEBHOOK_SECRET=gotong-webhook-secret \
  WEBHOOK_SOURCE_PLATFORM_ID=gotong_royong \
  MARKOV_READ_BASE_URL=http://127.0.0.1:3000/api/v1 \
  MARKOV_READ_PLATFORM_TOKEN=<platform_read_token> \
  MARKOV_READ_PLATFORM_ID=gotong_royong \
  CHAT_REALTIME_TRANSPORT=local LOG_LEVEL=warn \
  target/debug/gotong-api

# gotong worker
APP_ENV=smoke DATA_BACKEND=surrealdb \
  SURREAL_ENDPOINT=ws://127.0.0.1:8000 SURREAL_NS=gotong SURREAL_DB=chat \
  SURREAL_USER=root SURREAL_PASS=root REDIS_URL=redis://127.0.0.1:6379 \
  WEBHOOK_ENABLED=true \
  WEBHOOK_MARKOV_URL=http://127.0.0.1:3000/api/v1/platforms/gotong_royong/webhook \
  WEBHOOK_SECRET=gotong-webhook-secret \
  WEBHOOK_SOURCE_PLATFORM_ID=gotong_royong \
  MARKOV_READ_BASE_URL=http://127.0.0.1:3000/api/v1 \
  MARKOV_READ_PLATFORM_TOKEN=<platform_read_token> \
  MARKOV_READ_PLATFORM_ID=gotong_royong \
  LOG_LEVEL=warn \
  target/debug/gotong-worker

# tandang api (source)
ENVIRONMENT=development \
DATABASE_URL=postgresql://markov:***@127.0.0.1:54321/markov_engine \
REDIS_URL=redis://:***@127.0.0.1:63791 \
JWT_SECRET=change-this-secret-in-production-min-32-chars \
GOTONG_ROYONG_WEBHOOK_SECRET=gotong-webhook-secret \
PLATFORM_SERVICE_SECRET=gotong-read-secret \
PLATFORM_SERVICE_ALLOWED_PLATFORMS=gotong_royong \
TRUSTED_PLATFORM_AUTO_LINK=true \
TRUSTED_PLATFORM_IDS=gotong_royong \
target/debug/markov-api

# frontend
cd apps/web
GR_API_PROXY_TARGET=http://127.0.0.1:3100 bun run dev -- --host 127.0.0.1 --port 4173
```

## Smoke Result (via frontend host `http://127.0.0.1:4173`)

| Endpoint | Result |
|---|---|
| `GET /v1/auth/me` | ✅ `200` |
| `GET /v1/feed` | ✅ `200` |
| `GET /v1/notifications` | ✅ `200` |
| `GET /v1/chat/threads` | ✅ `200` |
| `GET /v1/chat/threads/:thread_id/messages` | ✅ `200` |
| `GET /v1/tandang/me/profile` | ✅ `200` |
| `GET /v1/tandang/users/:user_id/profile` | ✅ `200` |

## Additional Findings During Live Run

1. SurrealDB v3 compatibility fixes were required in Gotong repositories:
   - replaced deprecated `type::thing(...)` with `type::record(...)`
   - fixed contribution/evidence/vouch create queries to persist/read `datetime` fields correctly on v3

2. Tandang trusted auto-link/read compatibility was fixed:
   - fixed ordering bug in account auto-link (`users` row now created before `user_identity_hashes` persistence)
   - normalized ingestion platform IDs so new links are stored as raw `platform_user_id` (without duplicate `platform_id:` prefix)
   - added read-side compatibility for legacy prefixed links in `resolve_user_reputation_id`
   - validated with live user flow: `GET /v1/tandang/me/profile` moved from `404` before webhook to `200` after first valid webhook for fresh unlinked user
   - validation evidence: webhook response `status=success` for fresh user with valid contribution payload (`description >= 20 chars`, non-empty valid `skill_ids`, e.g. `G2.5`), and `account_links.platform_user_id` stored as raw ID

3. Markov PostgreSQL migration drift existed in local container.
   - applied missing SQL migrations from `../tandang/markov-engine/migrations`

4. Gotong webhook outbox persistence gap is fixed on local SurrealDB v3:
   - patched webhook outbox repository writes/reads to use explicit datetime coercion and nullable `next_attempt_at` projection compatible with Surreal v3
   - added migration/check pair: `database/migrations/0030_webhook_payload_flexible.surql` + `database/checks/0030_webhook_payload_flexible_check.surql` so webhook payload object remains schema-flexible for nested keys
   - live validation (`create_contribution` + `submit_vouch`) now persists outbox rows for both request IDs without `created_at/updated_at` coercion warnings

5. Worker retry payload gap is fixed:
   - root cause: retry enqueue reused the same `job_id` and then acknowledged the processing entry; `ack(job_id)` removed payload for the queued retry entry
   - fix: worker retry path now generates a new retry queue ID (`next_retry_job_id`) before enqueue+ack, while preserving webhook `event_id` in payload
   - live validation with isolated queue prefix (`WORKER_QUEUE_PREFIX=gotong:jobs:smoke:<id>`) shows retry lane health:
     - outbox row advances attempts (`attempts >= 1`, `status=retrying`)
     - Redis queue keeps retry payload (`delayed=1`, `payload=1`)
     - worker log has no `missing payload for job_id` errors
