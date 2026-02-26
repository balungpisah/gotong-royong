# Feature Completeness Report

Date: 2026-02-26  
Scope: `apps/web` frontend against current Gotong API + Tandang proxy contracts

## Summary
- Overall assessment: **partially complete** (hot-path reads are mostly API-backed; several core user journeys still break or degrade in production semantics).
- Top gaps:
  - Witness creation path is not API-backed and fails under production fail-fast settings.
  - Canonical witness permalink route is missing (`/saksi/:id` is referenced but not implemented).
  - Community dashboard remains mock-backed despite existing backend pulse endpoints.
  - Feed personalization writes (follow/monitor) are client-local only and not persisted.

## Assumptions and inputs
- Docs/requirements used:
  - `docs/architecture/hot-path-api-shapes.md`
  - `docs/research/frontend-service-api-cutover-backlog.md`
  - `docs/research/frontend-hot-path-integration-debt.md`
  - `docs/research/contracts/feature-contract-map.md`
- Code surfaces audited:
  - Routes: `apps/web/src/routes/*`
  - Service factory and API services: `apps/web/src/lib/services/*`
  - Stores: `apps/web/src/lib/stores/*`
  - E2E coverage: `apps/web/tests/e2e/*`, `apps/web/playwright.config.ts`
- Assumptions made due missing requirements:
  - Canonical deep-link for a witness should be first-class (share + notification should resolve to a real route).
  - "API-first in production" means no user-facing feature can depend on mock fallback.

## Endpoint Coverage (Grouped by Domain)

| Domain | Frontend calls | Status | Evidence |
|---|---|---|---|
| Auth/session | `POST /v1/auth/signin`, `GET /v1/auth/me` | Live | `apps/web/src/routes/masuk/+page.server.ts`, `apps/web/src/lib/services/api/user-service.ts` |
| Feed read | `GET /v1/feed`, `GET /v1/feed/suggestions` | Live (read) | `apps/web/src/lib/services/api/feed-service.ts` |
| Notifications | `GET /v1/notifications`, `POST /v1/notifications/:id/read`, `GET /v1/notifications/unread-count` | Live API, partial UX linkage | `apps/web/src/lib/services/api/notification-service.ts`, `apps/web/src/routes/notifikasi/+page.svelte` |
| Chat thread/message | `GET/POST /v1/chat/threads*`, `POST /v1/chat/attachments/upload` | Live | `apps/web/src/lib/services/api/witness-service.ts` |
| Witness lifecycle (entity create/permalink) | **No dedicated create endpoint used**; create path uses mock fallback | **Gap** | `apps/web/src/lib/services/api/witness-service.ts`, `crates/api/src/routes/mod.rs` |
| Triage | `POST /v1/triage/sessions`, `POST /v1/triage/sessions/:session_id/messages` | Live with dev/test fallback | `apps/web/src/lib/services/api/triage-service.ts` |
| Signal resolution | `/v1/witnesses/:witness_id/signals*` | Live with dev/test fallback | `apps/web/src/lib/services/api/signal-service.ts` |
| Group lifecycle | `/v1/groups*` | Live with dev/test fallback | `apps/web/src/lib/services/api/group-service.ts` |
| Profile/Tandang | `/v1/tandang/me/profile`, `/v1/tandang/users/:id/profile`, `/v1/tandang/users/:id/vouch-budget`, `/v1/tandang/decay/warnings/:id` | Live (read), dependent on Markov token config | `apps/web/src/lib/services/api/user-service.ts` |
| Community pulse | **No backend call from frontend** (store uses fixtures) | **Gap** | `apps/web/src/lib/stores/community-store.svelte.ts`, `apps/web/src/routes/komunitas/+page.svelte` |

## Gap list (actionable)
- ID: `FEAT-AUDIT-001`
- Gap: Witness creation is mock-only under API mode.
- Evidence: `ApiWitnessService.create()` delegates directly to fallback (`apps/web/src/lib/services/api/witness-service.ts`), while production disables mock fallback (`apps/web/src/lib/services/index.ts`).
- User impact: New witness flow from triage can fail in production-like runtime.
- Priority: `P0`
- Suggested fix: Add API-backed witness create contract (e.g., `POST /v1/witnesses`), implement in `ApiWitnessService.create()`, then remove mock dependency for create.

- ID: `FEAT-AUDIT-002`
- Gap: Witness permalink path is inconsistent and currently non-existent.
- Evidence: Notification click and share URL both target `/saksi/:id` (`apps/web/src/routes/notifikasi/+page.svelte`, `apps/web/src/lib/utils/share.ts`), but no `/saksi` route exists in `apps/web/src/routes`.
- User impact: Notification and shared links can lead to dead-end navigation.
- Priority: `P0`
- Suggested fix: Freeze one canonical witness route (`/witness/:id` or `/saksi/:id`), implement the route, and update all deep-link emitters.

- ID: `FEAT-AUDIT-003`
- Gap: Notification API mapping drops witness context.
- Evidence: `ApiNotificationService` does not map `source_id`/payload to `AppNotification.witness_id` (`apps/web/src/lib/services/api/notification-service.ts`), but route handler expects optional `witness_id` for navigation (`apps/web/src/routes/notifikasi/+page.svelte`).
- User impact: Notification cards often cannot deep-link users to the relevant witness.
- Priority: `P1`
- Suggested fix: Extend API mapping to persist target entity context and align notification click contract.

- ID: `FEAT-AUDIT-004`
- Gap: Community dashboard is still fixture-backed.
- Evidence: `CommunityStore` explicitly marked mock-backed with TODOs (`apps/web/src/lib/stores/community-store.svelte.ts`), while komunitas route consumes it (`apps/web/src/routes/komunitas/+page.svelte`).
- User impact: Community metrics are not trustworthy and diverge from backend reality.
- Priority: `P1`
- Suggested fix: Introduce `CommunityService` using `/v1/tandang/community/pulse/*` (and Gotong-owned overlays if needed), inject via service factory.

- ID: `FEAT-AUDIT-005`
- Gap: Feed follow/monitor actions are client-local only.
- Evidence: `toggleMonitor`, `toggleFollow`, and `followAllSuggested` mutate local store only; comments note missing write endpoints (`apps/web/src/lib/stores/feed-store.svelte.ts`).
- User impact: Preferences reset on reload/device switch; ranking personalization becomes inconsistent.
- Priority: `P1`
- Suggested fix: Add feed preference write endpoints and wire service methods for optimistic+confirmed updates.

- ID: `FEAT-AUDIT-006`
- Gap: "Stempel" action is simulated in frontend.
- Evidence: `handleStempel()` injects synthetic AI/system messages after `setTimeout` (`apps/web/src/routes/+page.svelte`), triggered from chat panel (`apps/web/src/lib/components/pulse/witness-chat-panel.svelte`).
- User impact: Perceived workflow completion can diverge from backend state/audit trail.
- Priority: `P1`
- Suggested fix: Replace with backend-backed checkpoint evaluation endpoint, or hide until backend is ready.

- ID: `FEAT-AUDIT-007`
- Gap: E2E coverage for API-first mode is narrow.
- Evidence: Playwright defaults most service toggles to false unless `PLAYWRIGHT_LIVE_API=true` (`apps/web/playwright.config.ts`); live smoke currently validates only auth/feed/notifications (`apps/web/tests/e2e/live-api-proxy.spec.ts`).
- User impact: Regressions in triage/signal/group/witness creation can pass CI unnoticed.
- Priority: `P2`
- Suggested fix: Add live-mode E2E matrix for triage, signal, group, and profile deep-link flows.

## Consistency issues
- Concept mismatch: Witness entity naming differs across surfaces (`witness`, `saksi`) without a canonical permalink route.
- Surfaces affected: `apps/web/src/lib/utils/share.ts`, `apps/web/src/routes/notifikasi/+page.svelte`, route tree under `apps/web/src/routes`.
- Suggested alignment: Freeze one route + naming contract and make it the only deep-link path emitted by share/notification/feed/chat.

- Concept mismatch: "API-first production" policy conflicts with mock-only create path.
- Surfaces affected: `apps/web/src/lib/services/index.ts`, `apps/web/src/lib/services/api/witness-service.ts`, `apps/web/src/lib/components/shell/chat-input.svelte`.
- Suggested alignment: No production-exposed user action should rely on mock fallback.

## Next steps
1. Close `FEAT-AUDIT-001` and `FEAT-AUDIT-002` together as one slice (witness create + permalink route + navigation emitters).
2. Implement `CommunityService` (`FEAT-AUDIT-004`) using existing community pulse endpoints.
3. Implement persistent feed preference writes (`FEAT-AUDIT-005`) and remove local-only semantics.
4. Replace/harden stempel behavior (`FEAT-AUDIT-006`) with backend contract.
5. Expand live E2E gate matrix (`FEAT-AUDIT-007`) before frontend live wiring sign-off.
