# Frontend Live Cutover Runbook

Last updated: 2026-02-25

Objective:
- Verify deployed frontend hosts use live backend APIs on hot paths (`/v1/auth/me`, `/v1/feed`, `/v1/notifications`) before broad traffic cutover.

## Prerequisites

- Frontend host is reachable from operator machine.
- Host is configured for API-backed service toggles (production defaults).
- Operator has local Playwright/Bun toolchain (`apps/web`).

## Commands

Dry-run preflight:

```bash
just frontend-live-cutover-gate
just frontend-live-cutover-rollout-dry-run staging
```

Staging rollout:

```bash
just frontend-live-cutover-rollout staging https://<staging-frontend-host>
```

Production rollout (only after staging evidence review):

```bash
just frontend-live-cutover-rollout production https://<production-frontend-host>
```

## Evidence Artifacts

- `docs/research/frontend-live-cutover-staging-latest.md`
- `docs/research/frontend-live-cutover-gate-staging-latest.md`
- `docs/research/frontend-live-cutover-production-latest.md`
- `docs/research/frontend-live-cutover-gate-production-latest.md`

## Exit Criteria

- Staging and production rollout reports are `PASS`.
- Gate reports for both environments show successful live Playwright smoke.
- Slice tracker `docs/research/frontend-live-cutover-001-latest.md` is updated to done.
