# Pack C Stage A Kickoff Report

Date: 2026-02-25T09:16:46Z
Namespace: `monitoring`
Observation window target: 24h

## Kickoff Summary

| Item | Result |
|---|---|
| Readiness gate | pass |
| Kubernetes cluster status | unreachable |
| Stage A rule action mode | dry-run |
| Stage A rule action result | dry_run_only |

## Commands Executed

1. `scripts/deploy/pack_c_cutover_readiness.sh --namespace monitoring`
2. `scripts/deploy/pack_c_prometheus_rules.sh --stage stage-a --namespace monitoring --dry-run`

## Stage A Observation Checklist (24h)

- Keep `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true` on all replicas.
- Watch lane distribution:
  - `sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m])) by (lane)`
- Watch shadow mismatch:
  - `increase(gotong_api_feed_involvement_shadow_mismatch_total[30m])`
- Watch feed SLO:
  - `histogram_quantile(0.95, sum(rate(gotong_api_http_request_duration_seconds_bucket{route="/v1/feed",method="GET"}[5m])) by (le))`

References:
- `docs/deployment/feed-involvement-fallback-removal-runbook.md`
- `docs/deployment/feed-involvement-fallback-alert-thresholds.md`
- `docs/research/pack-c-cutover-readiness-latest.md`
