# FRONTEND-LIVE-CUTOVER-001

Date: 2026-02-25
Status: `AWAITING_STAGING_PRODUCTION_HOSTS`

## Objective

Move frontend hot paths to live backend in deployed environments with explicit gate evidence (no mock runtime path in production).

## Gate Workflow

1. Static preflight:
   - `just frontend-live-cutover-gate`
2. Per environment live host validation:
   - `just frontend-live-cutover-gate-live https://<frontend-host>`
   - or orchestrated rollout wrapper:
     - `just frontend-live-cutover-rollout staging https://<staging-frontend-host>`
     - `just frontend-live-cutover-rollout production https://<production-frontend-host>`
3. Archive evidence:
   - `docs/research/frontend-live-cutover-gate-latest.md`
   - `docs/research/frontend-live-api-proxy-smoke-latest.md`
   - `docs/research/frontend-live-cutover-staging-latest.md`
   - `docs/research/frontend-live-cutover-production-latest.md`
   - `docs/research/frontend-live-cutover-gate-staging-latest.md`
   - `docs/research/frontend-live-cutover-gate-production-latest.md`

## Notes

- Live host gate runs Playwright smoke against deployed frontend host and verifies authenticated `200` responses for `/v1/auth/me`, `/v1/feed`, `/v1/notifications`.
- The gate uses `apps/web/tests/e2e/live-api-proxy.spec.ts` and external mode in `apps/web/playwright.config.ts`.
- Local end-to-end proof completed on 2026-02-25 via rollout wrapper (`just frontend-live-cutover-rollout staging ...`); reports updated at `docs/research/frontend-live-cutover-staging-latest.md` and `docs/research/frontend-live-cutover-gate-staging-latest.md`.
