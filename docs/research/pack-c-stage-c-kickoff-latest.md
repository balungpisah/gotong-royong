# Pack C Stage C Kickoff Report

Date: 2026-02-25T09:26:35Z
Namespace: `monitoring`
Observation window target: 24h
Stage summary: Full rollout with fallback OFF on all replicas.

## Kickoff Summary

| Item | Result |
|---|---|
| Readiness gate | skipped |
| Kubernetes cluster status | unreachable |
| Stage rule action mode | dry-run |
| Stage rule action result | dry_run_only |

## Commands Executed

1. `not run`
2. `scripts/deploy/pack_c_prometheus_rules.sh --stage stage-c --namespace monitoring --dry-run`

## Stage Checklist (24h)

- Set `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=false` on all replicas.
- Keep enhanced monitoring for at least the target window before considering fallback code removal.
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
