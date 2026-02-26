# Frontend Service API Cutover Backlog (Beyond Hot Path)

Last updated: 2026-02-25

Purpose:
- Track frontend domains that are still mock-backed.
- Freeze the next backend contracts so frontend can move to API-first safely.
- Keep cutover scope aligned with existing hot-path freeze (`chat`, `feed`, `notifications`).

## 1) Current Domain Status

| Domain | Frontend Service | Runtime Status | Backend Endpoint Status | Notes |
|---|---|---|---|---|
| Chat / Witness | `WitnessService` | API-first (`PUBLIC_GR_USE_API_CHAT`) | Implemented (extended) | Includes attachments + poll/stream + author snapshot. Added witness-create endpoint (`POST /v1/witnesses`) and canonical witness permalink route (`/saksi/:witness_id`) wiring on 2026-02-26. |
| Feed | `FeedService` | API-first (`PUBLIC_GR_USE_API_FEED`) | Implemented | Includes suggestions endpoint. |
| Notifications | `NotificationService` | API-first (`PUBLIC_GR_USE_API_NOTIFICATIONS`) | Implemented | Unread count + mark read flows wired. |
| User Profile | `UserService` | API-first (`PUBLIC_GR_USE_API_USER`) | Implemented (initial) | Uses `/v1/auth/me`, `/v1/tandang/me/profile`, and `/v1/tandang/users/:user_id/profile` for non-self reads; profile route `/profil/:user_id` is wired and deep-linked from feed/chat/witness avatar interactions with e2e coverage. For local live wiring, set `GR_API_PROXY_TARGET` so `/v1/*` calls from `bun dev` reach backend. Note: Tandang profile endpoints require `MARKOV_READ_PLATFORM_TOKEN` (+ `MARKOV_READ_BASE_URL`) on API runtime; otherwise local live calls return `500` (`docs/research/frontend-live-api-proxy-smoke-latest.md`). |
| Triage | `TriageService` | API-first (`PUBLIC_GR_USE_API_TRIAGE`); production fail-fast, dev/test fallback only | Implemented (initial) | Uses `POST /v1/triage/sessions` + `POST /v1/triage/sessions/:session_id/messages`; Playwright triage e2e auth/navigation helper hardened to reduce parallel-run flake during shell readiness. |
| Signal Resolution | `SignalService` | API-first (`PUBLIC_GR_USE_API_SIGNAL`); production fail-fast, dev/test fallback only | Implemented (initial) | Uses `/v1/witnesses/:witness_id/signals*` endpoints. |
| Group / Kelompok | `GroupService` | API-first (`PUBLIC_GR_USE_API_GROUP`); production fail-fast, dev/test fallback only | Implemented (initial) | Uses `/v1/groups*` lifecycle endpoints. |
| Community Pulse | `CommunityService` | API-first (`PUBLIC_GR_USE_API_COMMUNITY`) | Implemented (initial) | Uses `/v1/tandang/community/pulse/overview|insights|trends` to map Komunitas dashboard model (`CommunityStore.loadDashboard()`). |

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
- Playwright now has opt-in live mode (`PLAYWRIGHT_LIVE_API=true`) and a dedicated proxy smoke spec (`apps/web/tests/e2e/live-api-proxy.spec.ts`) that bootstraps a real auth token (`POST /v1/auth/signup`) and verifies `/v1/auth/me`, `/v1/feed`, and `/v1/notifications` return successful `200` responses via `GR_API_PROXY_TARGET`.

## 5) Next Slice: `FRONTEND-LIVE-CUTOVER-001`

Goal: prove deployed frontend hosts are API-backed on hot paths before enabling live user traffic.

Gate commands:
- Dry-run/static gate: `just frontend-live-cutover-gate`
- Live host gate: `just frontend-live-cutover-gate-live https://<frontend-host>`
- Direct smoke (same test, explicit): `cd apps/web && PLAYWRIGHT_EXTERNAL_BASE_URL=https://<frontend-host> bun run test:e2e:live-api:external`
- Env rollout wrapper: `just frontend-live-cutover-rollout <staging|production> https://<frontend-host>`

Artifacts:
- `docs/research/frontend-live-cutover-gate-latest.md`
- `docs/research/frontend-live-api-proxy-smoke-latest.md`
- `docs/research/frontend-live-cutover-<env>-latest.md`
- `docs/research/frontend-live-cutover-gate-<env>-latest.md`

## 6) Feature Completeness Follow-up Slices (2026-02-26)

Source: `docs/research/frontend-feature-completeness-audit-latest.md`

| Slice ID | Scope | Why it matters | Proposed Contract Direction |
|---|---|---|---|
| `WITNESS-API-002` | Witness create + canonical permalink route | Current triage → witness flow is mock-dependent and permalink emitters target a missing route. | Added `POST /v1/witnesses`, wired `ApiWitnessService.create()` to API-first create, implemented canonical witness detail route `apps/web/src/routes/saksi/[witness_id]/+page.svelte`, and aligned existing share/notification deep-link emitters to a live route. Status: implemented (initial) on 2026-02-26. |
| `NOTIF-API-002` | Notification target-link mapping | Notification cards cannot reliably deep-link due dropped witness/entity context in API mapping. | Implemented (initial) on 2026-02-26: `ApiNotificationService` now maps `witness_id` + sanitized `target_path` from payload/source fields and notifications route navigation now resolves canonical targets (`target_path` first, witness permalink fallback). |
| `COMMUNITY-API-001` | Komunitas dashboard API cutover | Community dashboard still fixture-backed despite backend pulse endpoints. | Implemented (initial) on 2026-02-26: added `ApiCommunityService` (`/v1/tandang/community/pulse/overview|insights|trends`), `MockCommunityService`, service-factory toggle (`PUBLIC_GR_USE_API_COMMUNITY`), and wired `CommunityStore.loadDashboard()` to API service. |
| `FEED-PREF-001` | Feed follow/monitor persistence | Current follow/pantau actions are local-only and reset across sessions/devices. | Implemented (initial) on 2026-02-26: added backend write endpoints (`POST /v1/feed/preferences/monitor/:witness_id`, `POST /v1/feed/preferences/follow/:entity_id`) + Surreal preference tables (`0031_feed_preference_schema`), overlaid preferences in feed/suggestion reads, and wired `FeedStore` optimistic toggles to persist via `ApiFeedService`. |
| `WITNESS-UX-001` | Stempel backend alignment | Stempel injected simulated frontend messages and could diverge from backend truth. | Implemented safe interim on 2026-02-25: removed synthetic stempel injection from `apps/web/src/routes/+page.svelte` and `apps/web/src/routes/dev/components/tandang/+page.svelte`; chat UI now hides the Stempel action when no backend handler/contract is wired (`apps/web/src/lib/components/pulse/witness-chat-panel.svelte`). Keep follow-up endpoint slice for checkpoint evaluation contract before re-enabling manual stempel. |
| `E2E-LIVE-002` | Live API E2E expansion | Current live smoke only validates auth/feed/notifications. | Implemented (initial) on 2026-02-25: expanded `apps/web/tests/e2e/live-api-proxy.spec.ts` to cover live triage session start/continue, witness-create + `/saksi/:witness_id` permalink load, signal endpoint round-trip (`POST/GET/DELETE /v1/witnesses/:witness_id/signals*`), group list/create/detail (`/v1/groups*`), and profile route/deep-link request verification (`/v1/tandang/me/profile` or `/v1/tandang/users/:user_id/profile`). |
