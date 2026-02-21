# Frontend Foundation Implementation Plan

Last updated: `2026-02-17`  
Status: `READY FOR EXECUTION`

## Purpose

Create a frontend foundation sprint that sets up project structure, design system plumbing, auth/realtime baselines, and engineering tooling so feature work can start with low rework risk.

Canonical references:
- `docs/research/FRONTEND-TECHNOLOGY-RESEARCH-REPORT.md`
- `docs/research/ui-feature-inventory.md`
- `docs/design/specs/UI-GUIDELINE-v1.0.md`
- `docs/design/specs/DESIGN-DNA-v0.1.md`

## Scope

In scope:
- Greenfield SvelteKit 2 + Svelte 5 app scaffold
- Core boilerplate (routing shell, API client, state/store conventions)
- Foundation auth path (direct JWT session handling in `hooks.server.ts`)
- Realtime baseline (WebSocket primary, SSE/polling fallback strategy)
- Tooling and quality gates (lint, format, tests, CI)
- PWA baseline and performance budget guardrails

Out of scope:
- Full implementation of UI-01..UI-23 features
- Final visual polish across all screens
- Production hardening beyond foundation-level readiness
- Framework re-evaluation (already covered by frontend technology report)

## Non-Negotiable Constraints

- Runtime profile: Bun for dev workflows, Node.js (`@sveltejs/adapter-node`) for production.
- Auth default: direct JWT session handling first; introduce `@auth/sveltekit` only if multi-provider OAuth becomes a requirement.
- Realtime model: WebSocket primary with SSE and polling fallback compatibility.
- Performance guardrails: framework runtime < 10 KB gzipped, initial route JS < 50 KB gzipped, total JS (lazy-loaded) < 200 KB gzipped, LCP < 2.5s, TTI < 3.5s on 4G median (derived from `docs/research/FRONTEND-TECHNOLOGY-RESEARCH-REPORT.md` Section 4.2).
- Privacy baseline: server-side data minimization is primary; client-side hiding is secondary and non-authoritative.

## Action Items

- [ ] Initialize frontend in monorepo path `apps/web` with TypeScript, ESLint, and Prettier.
- [ ] Set up baseline project structure: `src/routes`, `src/lib/components`, `src/lib/api`, `src/lib/stores`, `src/lib/utils`, `src/lib/types`.
- [ ] Wire Tailwind and port Design DNA tokens (colors, typography, spacing, shadows) into theme config and global styles.
- [ ] Install and configure shadcn-svelte (ADR-03): initialize with project theme, verify Svelte 5 compatibility, add initial base components (Button, Card, Dialog, Input, Badge).
- [ ] Install and configure Paraglide JS 2 (ADR-04): set up Vite plugin, create `messages/id.json` for Bahasa Indonesia strings, wire `<ParaglideJS>` provider in root layout. All UI strings must use message keys from day one.
- [ ] Add core app shell skeleton (root layout + mobile-first navigation scaffold + placeholder route groups).
- [ ] Implement a typed API client wrapper with standard error envelope handling, auth header/cookie behavior, and retry policy boundaries.
- [ ] Implement direct JWT session handling in `hooks.server.ts` plus protected-route guard helpers and role-check utilities.
- [ ] Build realtime transport manager abstraction (WebSocket first, SSE fallback path, degraded polling mode hook).
- [ ] Add PWA baseline (`@vite-pwa/sveltekit`) with conservative cache policy and explicit `no-store` handling for sensitive routes.
- [ ] Configure tests and CI gates: unit (`vitest`), E2E smoke (`playwright`), lint/typecheck/build checks on pull requests.
- [ ] Add foundation docs for contributors: local run commands, env vars, coding conventions, and definition-of-done checklist.

## Validation Gates

- Build and typecheck pass on clean clone.
- Lint and format checks pass in CI.
- Minimal E2E smoke verifies app shell render, auth guard redirect, and a mocked realtime connection lifecycle.
- Lighthouse or equivalent baseline run confirms LCP < 2.5s and TTI < 3.5s on simulated 4G.
- Bundle size CI check (`bundlesize` or `vite-bundle-analyzer`): framework runtime < 10 KB gzipped, initial route JS < 50 KB gzipped.
- Security sanity check confirms no sensitive payloads are cached by service worker for protected endpoints.

## Exit Criteria (Foundation Complete)

- New contributors can install dependencies, run dev server, and run tests from documented commands.
- Frontend shell and shared primitives exist and are ready for UI feature integration work.
- Auth and realtime baselines are integrated behind stable utility interfaces.
- CI blocks regressions for lint/type/test/build from day one.

## Open Questions

- ~~Should frontend live inside this repo as `apps/web`, or as a sibling `gotong-web` repo?~~ **Recommended: `apps/web` in this repo.** The Rust backend is already a Cargo workspace at repo root (`crates/`). Adding `apps/web` keeps the monorepo pattern, avoids cross-repo CI/versioning overhead, and simplifies shared tooling (linting, formatting, deployment scripts). Revisit only if repo size becomes a concern.
- Which deployment target is primary for first staging rollout (self-hosted Node, container platform, or other)?
- Do we require visual regression tooling in foundation scope, or defer to Phase 2?
