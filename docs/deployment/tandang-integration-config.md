# Gotong ↔ Tandang Deployment Config Matrix

Last updated: 2026-02-25

This document is the deployment source of truth for integration-specific env vars, toggles, and safe defaults.

## 1. Gotong Royong (sender + reader) env vars

| Variable | Required | Safe default (code) | Production guidance |
|---|---|---|---|
| `WEBHOOK_ENABLED` | Yes | `false` | Set `true` in prod once Tandang endpoint is reachable. |
| `WEBHOOK_MARKOV_URL` | Yes when `WEBHOOK_ENABLED=true` | `https://api.markov.local/v1/platforms/gotong_royong/webhook` | Set to real Tandang URL (`/api/v1/platforms/gotong_royong/webhook`). |
| `WEBHOOK_SECRET` | Yes when `WEBHOOK_ENABLED=true` | `dev_webhook_secret_32_chars_minimum` | 32+ random bytes from secrets manager. |
| `WEBHOOK_MAX_ATTEMPTS` | Yes | `5` | Keep `5` unless SLO tuning requires change. |
| `WEBHOOK_SOURCE_PLATFORM_ID` | Yes | `gotong_royong` | Keep stable across environments; emitted on every webhook payload as `source_platform_id` for Phase-2 platform-partitioned reputation. |
| `MARKOV_READ_BASE_URL` | Yes | `http://127.0.0.1:3000/api/v1` | Set real Tandang API base URL. |
| `MARKOV_READ_PLATFORM_TOKEN` | Yes when Markov read APIs are enabled | empty string | Required for Gotong server-to-server read routes. If Gotong uses `POST /api/v1/cv-hidup/{user_id}/export`, this token must include `write` scope too. |
| `MARKOV_READ_PLATFORM_ID` | Yes | `gotong_royong` | Platform ID used by explicit read-scope handshake query params. Keep aligned with trusted platform registration in Tandang. |
| `MARKOV_READ_EXPLICIT_SCOPE_QUERY_ENABLED` | No | `false` | Keep `false` until Tandang read handlers accept explicit scope query params (`view_scope=platform&platform_id=...`). Turn on during Phase-2 cutover. |
| `MARKOV_READ_TIMEOUT_MS` | Yes | `2500` | Keep `2000-4000` based on latency profile. |
| `MARKOV_READ_RETRY_MAX_ATTEMPTS` | Yes | `3` | Keep low to avoid fan-out storms. |
| `MARKOV_READ_RETRY_BACKOFF_BASE_MS` | Yes | `200` | Keep exponential retry base small. |
| `MARKOV_READ_RETRY_BACKOFF_MAX_MS` | Yes | `2000` | Keep bounded. |
| `MARKOV_READ_CIRCUIT_FAIL_THRESHOLD` | Yes | `5` | Tune with alerting and traffic level. |
| `MARKOV_READ_CIRCUIT_OPEN_MS` | Yes | `15000` | Keep short to recover quickly. |
| `MARKOV_CACHE_PROFILE_TTL_MS` | Yes | `300000` | 5m is safe for profile surfaces. |
| `MARKOV_CACHE_PROFILE_STALE_WHILE_REVALIDATE_MS` | Yes | `1200000` | 20m stale window for resilience. |
| `MARKOV_CACHE_GAMEPLAY_TTL_MS` | Yes | `45000` | Keep short for gameplay freshness. |
| `MARKOV_CACHE_GAMEPLAY_STALE_WHILE_REVALIDATE_MS` | Yes | `180000` | 3m stale window for gameplay endpoints. |

## 2. Tandang (receiver + scoped reader) env vars

| Variable | Required | Safe default (code) | Production guidance |
|---|---|---|---|
| `GOTONG_ROYONG_WEBHOOK_SECRET` | Yes | none guaranteed | Must match Gotong `WEBHOOK_SECRET`. |
| `TRUSTED_PLATFORM_AUTO_LINK` | Yes | `true` | Keep `true` for transparent linking. |
| `TRUSTED_PLATFORM_IDS` | Yes | `gotong_royong` | Include `gotong_royong`; comma-separated list. |
| `PLATFORM_SERVICE_KEYS` | Recommended | none | Preferred multi-key format for rotation (`kid=secret,...`). |
| `PLATFORM_SERVICE_SECRET` | Fallback only | none | Use only when single-key setup is unavoidable. |
| `PLATFORM_SERVICE_ACTIVE_KID` | Yes (with multi-key) | `default` | Set active signing key ID. |
| `PLATFORM_SERVICE_ALLOWED_PLATFORMS` | Yes | `gotong_royong` | Keep strict allowlist. |
| `PLATFORM_SERVICE_DEFAULT_SCOPE` | Yes | `read` | Keep least-privilege scope list (issue per-platform tokens with `write` only when needed). |
| `PLATFORM_SERVICE_TOKEN_TTL_SECS` | Yes | `3600` | 1h default; shorten if security posture requires. |

Notes:
- Gotong webhook dedupe retention/cap are currently internal constants in Tandang (`7 days`, `200000` keys) and not env-configurable.
- `TANDANG_WEBHOOK_SECRET` is unrelated to Gotong webhook ingestion; it protects the separate `@tandangIndex` webhook path.

## 3. Deployment sequence (no-downtime baseline)

1. Provision secrets in secret manager:
   - `WEBHOOK_SECRET` (Gotong) and matching `GOTONG_ROYONG_WEBHOOK_SECRET` (Tandang).
   - `PLATFORM_SERVICE_KEYS` with at least one active key.
2. Deploy Tandang with trusted-platform toggles enabled:
   - `TRUSTED_PLATFORM_AUTO_LINK=true`
   - `TRUSTED_PLATFORM_IDS=gotong_royong`
3. Deploy Gotong with webhook/read integration toggles:
   - `WEBHOOK_ENABLED=true`
   - `WEBHOOK_SOURCE_PLATFORM_ID=gotong_royong`
   - `MARKOV_READ_PLATFORM_TOKEN=<issued token>`
   - Keep `MARKOV_READ_EXPLICIT_SCOPE_QUERY_ENABLED=false` unless Tandang Phase-2 query contract is deployed.
4. Validate:
   - webhook delivery success > 99%
   - no signature/auth failures
   - read APIs returning scoped reputation surfaces

## 4. Rotation checklist

1. Add new key in `PLATFORM_SERVICE_KEYS` (keep old key present).
2. Set `PLATFORM_SERVICE_ACTIVE_KID` to new key.
3. Re-issue Gotong read token using new key.
4. Verify reads still succeed.
5. Remove old key after drain window.

For webhook secret rotation:
1. Deploy Tandang to accept new secret.
2. Deploy Gotong to sign with new secret.
3. Verify signature success metrics.
4. Remove old secret material.
