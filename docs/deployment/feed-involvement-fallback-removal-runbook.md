# Feed Involvement Fallback Removal Runbook (Pack C)

Last updated: 2026-02-25

Purpose:
- Remove legacy OR-query fallback for `involvement_only=true` safely.
- Use the runtime switch `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED` to run canary phases before full cutover.

References:
- Design/status: `docs/database/hot-path-pack-c-feed-participant-edge-design-v1.md`
- SLO matrix: `docs/database/hot-path-query-shape-slo-matrix.md`
- Backfill runbook: `docs/deployment/feed-participant-edge-backfill.md`
- Alert thresholds: `docs/deployment/feed-involvement-fallback-alert-thresholds.md`
- Deployable Prometheus rules: `deploy/monitoring/README.md`
- Benchmark artifact: `docs/research/surrealdb-feed-involvement-bench-latest.md`

## 1) Preconditions

- `feed_participant_edge` schema and indexes are migrated (`0027_*`).
- Historical edge backfill has been executed and verified in target environment.
- Runtime supports the switch:
  - `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true` → edge-first + legacy fallback.
  - `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=false` → edge-only (no fallback).

## 2) Mandatory pre-cutover checks

From repo root:

```bash
just pack-c-cutover-readiness
```

Equivalent explicit commands:

```bash
just pack-c-slice-gate
just feed-participant-edge-backfill --dry-run --page-size 1000 --progress-every 1000
just smoke-feed-involvement-edge-cutover-live
just feed-involvement-bench-surreal
```

Expected:
- Pack C slice gate passes (`just pack-c-slice-gate`).
- Backfill dry-run completes without unexpected failures.
- Cutover smoke passes both modes (fallback-on and edge-only).
- Benchmark still shows edge lane avoiding participant-membership scan behavior under noisy datasets.

## 3) Canary rollout sequence

Use staged rollout for API pods/instances.

### Stage A — Baseline (fallback ON)

- Optional kickoff automation (recommended):
  - `just pack-c-stage-a-kickoff`

- Keep `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true` on all replicas.
- Apply Stage A alert rules:
  - `just pack-c-alerts-stage-a`
- Observe for at least 24h:
  - lane usage
  - mismatch counter
  - feed/search latency

### Stage B — Canary (fallback OFF on a subset)

- Optional kickoff automation (recommended):
  - `just pack-c-stage-b-kickoff`

- Set `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=false` on 5–10% of replicas.
- Replace alert rules with Stage B profile:
  - `just pack-c-alerts-stage-b`
- Run for 2–4h minimum, then increase to 25%, then 50% if stable.

### Stage C — Full cutover

- Optional kickoff automation (recommended):
  - `just pack-c-stage-c-kickoff`

- Set `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=false` on all replicas.
- Replace alert rules with Stage C profile:
  - `just pack-c-alerts-stage-c`
- Keep enhanced monitoring for at least 24h.

## 4) Metrics to watch

Threshold sheet:
- `docs/deployment/feed-involvement-fallback-alert-thresholds.md`
- `deploy/monitoring/grafana-pack-c-cutover-dashboard.json`

### Lane distribution

```promql
sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint="feed"}[5m])) by (lane)
sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint="search"}[5m])) by (lane)
```

Goal after full cutover:
- `lane="edge"` dominates.
- `lane="fallback"` and `lane="legacy"` remain `0`.
- `lane="edge_error"` and `lane="edge_partial"` remain `0` (or extremely rare and investigated immediately).

### Shadow mismatch (while fallback ON stages)

```promql
increase(gotong_api_feed_involvement_shadow_mismatch_total[30m])
```

Goal:
- No sustained growth during baseline/canary with fallback enabled.

### Feed latency SLO

```promql
histogram_quantile(
  0.95,
  sum(rate(gotong_api_http_request_duration_seconds_bucket{route="/v1/feed",method="GET"}[5m])) by (le)
)
```

SLO gate:
- `GET /v1/feed` p95 `<= 180ms` (limit=20 contract).

## 5) Go / No-Go criteria

Go to next stage only if all are true:
- No sustained `edge_error` spikes in canary.
- No sustained `edge_partial` during normal traffic.
- Feed p95 latency stays within SLO (or no regression beyond agreed guard band).
- No material increase in feed/search 5xx.

No-Go immediately if any is true:
- Repeated `edge_error` for the same endpoint over consecutive intervals.
- Significant `edge_partial` growth after traffic stabilizes.
- SLO breach tied to `involvement_only` feed/search path.
- User-visible missing feed items attributable to edge coverage.

## 6) Rollback

If No-Go condition is hit:

1. Set `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true` for all replicas.
2. Roll API deployment to the last stable revision if needed.
3. Re-run:
   - `just smoke-feed-involvement-edge-cutover-live`
   - targeted feed query checks in staging
4. Open incident note with:
   - timeframe
   - lane counters
   - sample request IDs/correlation IDs
   - exact rollback timestamp

## 7) Exit criteria for code cleanup

Only remove fallback logic in code when:
- Full cutover has run stable for a defined window (recommended: >= 7 days),
- `fallback` / `legacy` lanes are effectively zero,
- no unresolved edge coverage incidents remain.
