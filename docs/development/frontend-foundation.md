# Frontend Foundation Contributor Guide

Last updated: `2026-02-17`  
Status: `active`

## Purpose

This guide is the contributor baseline for `apps/web` during the frontend foundation sprint. It documents local setup, required environment variables, coding conventions, and the frontend foundation definition-of-done.

## Canonical References

- `docs/research/frontend-foundation-implementation-plan.md`
- `docs/research/FRONTEND-TECHNOLOGY-RESEARCH-REPORT.md`
- `docs/design/specs/UI-GUIDELINE-v1.0.md`
- `docs/design/specs/DESIGN-DNA-v0.1.md`

## Scope and Runtime

- Frontend lives at `apps/web`.
- Framework baseline: SvelteKit 2 + Svelte 5 + TypeScript.
- Local tooling baseline: Bun.
- Current repo config uses `svelte-adapter-bun` in `apps/web/svelte.config.js`.
- Production runtime target is Bun via `svelte-adapter-bun`.

## Prerequisites

- Bun `1.3.6+`
- Node.js `22+` (optional fallback for ecosystem tooling)

## Setup

```sh
cd apps/web
bun install
cp .env.example .env # optional but recommended
bunx playwright install chromium
```

## Required Environment Variables

| Variable | Required | Default | Used by | Notes |
| --- | --- | --- | --- | --- |
| `JWT_SECRET` | Yes for authenticated flows | none | `src/lib/auth/server.ts` | If unset, all requests are treated as unauthenticated. Use a 32+ character secret. |
| `GR_SESSION_COOKIE_NAME` | No | `gr_session` | `src/lib/auth/server.ts` | Override only if cookie naming must differ by environment. |
| `GR_API_PROXY_TARGET` | No | unset | `vite.config.ts` | When set (e.g. `http://127.0.0.1:3000`), `bun dev` proxies browser `/v1/*` calls to backend for live API wiring. |
| `GR_AUTH_DEV_BYPASS_ENABLED` | No | `false` | `src/hooks.server.ts` | Dev-only auth bypass for protected-route redirects; only effective when `NODE_ENV=development`. |
| `GR_AUTH_DEV_BYPASS_USER_ID` | No | `dev-user` | `src/hooks.server.ts` | Optional synthetic user id used when `GR_AUTH_DEV_BYPASS_ENABLED=true`. |
| `PUBLIC_GR_USE_API_NOTIFICATIONS` | No | `true` | `src/lib/services/index.ts` | Set `false` only for local dev/test mock runs; production runtime rejects false values. |
| `PUBLIC_GR_USE_API_FEED` | No | `true` | `src/lib/services/index.ts` | Set `false` only for local dev/test mock runs; production runtime rejects false values. |
| `PUBLIC_GR_USE_API_CHAT` | No | `true` | `src/lib/services/index.ts` | Set `false` only for local dev/test mock runs; production runtime rejects false values. |
| `PUBLIC_GR_USE_API_USER` | No | `true` | `src/lib/services/index.ts` | Set `false` only for local dev/test mock runs; production runtime rejects false values. |
| `PUBLIC_GR_USE_API_TRIAGE` | No | `true` | `src/lib/services/index.ts` | Set `false` only for local dev/test mock runs; production runtime rejects false values. |
| `PUBLIC_GR_USE_API_SIGNAL` | No | `true` | `src/lib/services/index.ts` | Set `false` only for local dev/test mock runs; production runtime rejects false values. |
| `PUBLIC_GR_USE_API_GROUP` | No | `true` | `src/lib/services/index.ts` | Set `false` only for local dev/test mock runs; production runtime rejects false values. |
| `PLAYWRIGHT_EXTERNAL_BASE_URL` | No | unset | `playwright.config.ts` | When set, Playwright targets an already deployed frontend host and skips starting local Vite dev server. |

## Daily Commands

```sh
cd apps/web
bun run dev
bun run lint
bun run check
bun run test:unit
bun run test:e2e
bun run build
```

## CI Quality Gates (PR)

Workflow: `.github/workflows/ci.yml` (`web` job)

- `bun run lint`
- `bun run check`
- `bun run test:unit`
- `bun run test:e2e`
- `bun run build`

Notes:
- App runtime defaults hot-path services to API-first (`PUBLIC_GR_USE_API_*` default `true`).
- Production runtime enforces API-only service selection and disables API-service mock fallback paths.
- Playwright config forces those toggles to `false` for deterministic frontend-only E2E in CI.

## Coding Conventions

- Use Paraglide message keys for all UI strings. Do not hardcode user-facing text in components.
  - Message catalog: `apps/web/messages/id.json`
- Keep Tanah design DNA tokens as source of truth (`apps/web/src/app.css`); avoid ad-hoc raw color systems.
- Keep route authorization in `apps/web/src/hooks.server.ts` and auth helpers in `apps/web/src/lib/auth/*`.
- Keep API access inside `apps/web/src/lib/api/index.ts` (typed client); avoid ad-hoc `fetch` wrappers per feature.
- Keep realtime integration through `apps/web/src/lib/api/realtime.ts` to preserve fallback behavior.
- Treat sensitive routes (`/masuk`, protected app routes, role-guarded paths) as `no-store`.
- Preserve mobile-first shell behavior in `apps/web/src/routes/+layout.svelte`.

## Foundation Definition of Done

Use this checklist before marking frontend-foundation work done:

- [ ] Clean clone can install and run `lint`, `check`, `test:unit`, `test:e2e`, and `build`.
- [ ] CI (`web` job) passes on pull request.
- [ ] Protected-route redirect and session behavior are validated by smoke tests.
- [ ] Sensitive routes return `Cache-Control: no-store` and are excluded from PWA navigation fallback.
- [ ] PWA build emits service worker + manifest without cache-policy regressions.
- [ ] UI strings are sourced from Paraglide message keys.
- [ ] Changes keep performance guardrails from the implementation plan:
  - Framework runtime `< 10 KB` gzipped
  - Initial route JS `< 50 KB` gzipped
  - Total lazy-loaded JS `< 200 KB` gzipped
  - LCP `< 2.5s`, TTI `< 3.5s` (4G median target)
- [ ] Contributor-facing docs stay updated when commands, env vars, or conventions change.
