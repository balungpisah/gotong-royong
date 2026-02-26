# Frontend Live Cutover Gate Report

Date: 2026-02-26T07:50:53Z
Mode: `dry-run`
Frontend URL: `n/a`
Result: `PASS`

## Command Status

| Check | Status | Duration |
|---|---|---|
| Production guard enforces API-only services | PASS | 0s |
| External live smoke entrypoints exist | PASS | 0s |
| Playwright live smoke via deployed frontend host | SKIPPED | 0s |

## Commands Executed

- `rg -q "assertApiEnabledInProduction" apps/web/src/lib/services/index.ts &&    rg -q "PUBLIC_GR_USE_API_NOTIFICATIONS" apps/web/src/lib/services/index.ts &&    rg -q "PUBLIC_GR_USE_API_FEED" apps/web/src/lib/services/index.ts &&    rg -q "PUBLIC_GR_USE_API_CHAT" apps/web/src/lib/services/index.ts &&    rg -q "PUBLIC_GR_USE_API_USER" apps/web/src/lib/services/index.ts &&    rg -q "PUBLIC_GR_USE_API_TRIAGE" apps/web/src/lib/services/index.ts &&    rg -q "PUBLIC_GR_USE_API_SIGNAL" apps/web/src/lib/services/index.ts &&    rg -q "PUBLIC_GR_USE_API_GROUP" apps/web/src/lib/services/index.ts`
- `rg -q "test:e2e:live-api:external" apps/web/package.json &&    rg -q "web-test-e2e-live-api-external" justfile &&    test -f apps/web/tests/e2e/live-api-proxy.spec.ts`
- `skipped: cd apps/web && PLAYWRIGHT_EXTERNAL_BASE_URL=<frontend-url> bun run test:e2e:live-api:external`

## Context

- Related backlog: `docs/research/frontend-service-api-cutover-backlog.md`
- Related debt tracker: `docs/research/frontend-hot-path-integration-debt.md`
- Related smoke spec: `apps/web/tests/e2e/live-api-proxy.spec.ts`
