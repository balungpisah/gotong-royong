# Frontend Service API Cutover Backlog (Beyond Hot Path)

Last updated: 2026-02-25

Purpose:
- Track frontend domains that are still mock-backed.
- Freeze the next backend contracts so frontend can move to API-first safely.
- Keep cutover scope aligned with existing hot-path freeze (`chat`, `feed`, `notifications`).

## 1) Current Domain Status

| Domain | Frontend Service | Runtime Status | Backend Endpoint Status | Notes |
|---|---|---|---|---|
| Chat / Witness | `WitnessService` | API-first (`PUBLIC_GR_USE_API_CHAT`) | Implemented | Includes attachments + poll/stream + author snapshot. |
| Feed | `FeedService` | API-first (`PUBLIC_GR_USE_API_FEED`) | Implemented | Includes suggestions endpoint. |
| Notifications | `NotificationService` | API-first (`PUBLIC_GR_USE_API_NOTIFICATIONS`) | Implemented | Unread count + mark read flows wired. |
| User Profile | `UserService` | API-first (`PUBLIC_GR_USE_API_USER`) | Implemented (initial) | Uses `/v1/auth/me`, `/v1/tandang/me/profile`, and `/v1/tandang/users/:user_id/profile` for non-self reads; profile route `/profil/:user_id` is wired and deep-linked from feed/chat/witness avatar interactions with e2e coverage. For local live wiring, set `GR_API_PROXY_TARGET` so `/v1/*` calls from `bun dev` reach backend. Note: Tandang profile endpoints require `MARKOV_READ_PLATFORM_TOKEN` (+ `MARKOV_READ_BASE_URL`) on API runtime; otherwise local live calls return `500` (`docs/research/frontend-live-api-proxy-smoke-latest.md`). |
| Triage | `TriageService` | API-first (`PUBLIC_GR_USE_API_TRIAGE`); production fail-fast, dev/test fallback only | Implemented (initial) | Uses `POST /v1/triage/sessions` + `POST /v1/triage/sessions/:session_id/messages`; Playwright triage e2e auth/navigation helper hardened to reduce parallel-run flake during shell readiness. |
| Signal Resolution | `SignalService` | API-first (`PUBLIC_GR_USE_API_SIGNAL`); production fail-fast, dev/test fallback only | Implemented (initial) | Uses `/v1/witnesses/:witness_id/signals*` endpoints. |
| Group / Kelompok | `GroupService` | API-first (`PUBLIC_GR_USE_API_GROUP`); production fail-fast, dev/test fallback only | Implemented (initial) | Uses `/v1/groups*` lifecycle endpoints. |

## 2) Frozen Proposed Contracts (Next Slices)

### TRIAGE-API-001

Goal: replace `MockTriageService` with API-backed conversational flow.

Proposed endpoints:
- `POST /v1/triage/sessions`
  - request: `{ content: string, attachments?: [...] }`
  - response: `{ session_id: string, result: TriageResult }`
- `POST /v1/triage/sessions/:session_id/messages`
  - request: `{ answer: string, attachments?: [...] }`
  - response: `{ result: TriageResult }`

Backend notes:
- Keep AI provider/Edge-Pod internals hidden behind this contract.
- Enforce idempotency keys (`x-request-id`) for writes.
- Status: implemented (initial) on 2026-02-25.

### SIGNAL-API-001

Goal: replace `MockSignalService` for witness signal chips and resolution history.

Proposed endpoints:
- `POST /v1/witnesses/:witness_id/signals`
  - request: `{ signal_type: "saksi" | "perlu_dicek" }`
  - response: `ContentSignal`
- `DELETE /v1/witnesses/:witness_id/signals/:signal_type`
- `GET /v1/witnesses/:witness_id/signals/my-relation`
  - response: `MyRelation`
- `GET /v1/witnesses/:witness_id/signals/counts`
  - response: `SignalCounts`
- `GET /v1/witnesses/:witness_id/signals/resolutions`
  - response: `ContentSignal[]`

Backend notes:
- Persist as append-only signal events + read-model aggregate for counts.
- Keep resolution computation deterministic on witness terminal transition.
- Status: implemented (initial) on 2026-02-25.

### GROUP-API-001

Goal: replace `MockGroupService` and enable real kelompok/lembaga lifecycle.

Proposed endpoints:
- `POST /v1/groups`
- `GET /v1/groups`
- `GET /v1/groups/me`
- `GET /v1/groups/:group_id`
- `PATCH /v1/groups/:group_id`
- `POST /v1/groups/:group_id/join`
- `POST /v1/groups/:group_id/requests`
- `POST /v1/groups/:group_id/requests/:request_id/approve`
- `POST /v1/groups/:group_id/requests/:request_id/reject`
- `POST /v1/groups/:group_id/invite`
- `POST /v1/groups/:group_id/leave`
- `POST /v1/groups/:group_id/members/:user_id/remove`
- `POST /v1/groups/:group_id/members/:user_id/role`

Backend notes:
- Read-model first: `group_summary`, `group_member`, `group_join_request` with pagination indexes.
- Preserve feed integration by maintaining stable `entity_tag.entity_id == group_id`.
- Status: implemented (initial) on 2026-02-25; persistence wired via `GroupRepository` with in-memory + Surreal implementations (migration `0029_group_read_model_schema.surql`) and validated against live dev DB with `just db-migrate`, `just db-check`, `just release-gates-surreal` (`docs/research/release-gates-surreal-latest.md`).

## 3) Recommended Execution Order

1. Contract hardening + persistence migration (replace in-memory group registry with Surreal read models and migration path).

Each slice should include:
- backend contract + handler tests,
- frontend API service + unit tests,
- toggle in `apps/web/src/lib/services/index.ts`,
- debt tracker update in `docs/research/frontend-hot-path-integration-debt.md` (or closeout note).

## 4) Contract Reliability Gate (2026-02-25)

Completed for current live domains:
- Production runtime guard in `apps/web/src/lib/services/index.ts` blocks `PUBLIC_GR_USE_API_*=false` and disables API-service mock fallback paths.
- Backend contract tests now assert standard error envelope (`error.code`, `error.message`) for auth/forbidden/validation on representative hot-path routes.
- Frontend API client tests now assert backend error envelope parsing and fallback behavior for non-standard/opaque upstream errors.
- Playwright now has opt-in live mode (`PLAYWRIGHT_LIVE_API=true`) and a dedicated proxy smoke spec (`apps/web/tests/e2e/live-api-proxy.spec.ts`) that bootstraps a real auth token (`POST /v1/auth/signup`) and verifies `/v1/auth/me`, `/v1/feed`, and `/v1/notifications` return strict `200` responses via `GR_API_PROXY_TARGET`.
