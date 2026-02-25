# Pack C Stage B Kickoff Report

Date: 2026-02-25T18:05:26Z
Namespace: `monitoring`
Observation window target: 4h
Stage summary: Canary rollout with fallback OFF on a subset of replicas.

## Kickoff Summary

| Item | Result |
|---|---|
| Readiness gate | skipped |
| Kubernetes cluster status | unreachable |
| Stage rule action mode | dry-run |
| Stage rule action result | dry_run_only |
| Go/no-go gate | pass |
| Go/no-go mode | dry-run |
| Go/no-go window | 4h |
| Go/no-go report | docs/research/pack-c-stage-b-go-no-go-latest.md |

## Commands Executed

1. `not run`
2. `scripts/deploy/pack_c_prometheus_rules.sh --stage stage-b --namespace monitoring --dry-run`
3. `scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-b --prom-url http://127.0.0.1:9090 --window 4h --step 60s --output docs/research/pack-c-stage-b-go-no-go-latest.md --dry-run`

## Stage Checklist (4h)

- Set `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=false` on 5–10% of replicas.
- Increase to 25% then 50% only if no critical alerts and SLO remains stable.
- Watch lane distribution:
  - `sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m])) by (lane)`
- Watch shadow mismatch:
  - `sum(increase(gotong_api_feed_involvement_shadow_mismatch_total[30m]))`
- Watch feed SLO:
  - `max(gotong_api_http_request_duration_seconds{route="/v1/feed",method="GET",quantile="0.95"})`

References:
- `docs/deployment/feed-involvement-fallback-removal-runbook.md`
- `docs/deployment/feed-involvement-fallback-alert-thresholds.md`
- `docs/research/pack-c-cutover-readiness-latest.md`
- `docs/research/pack-c-stage-b-go-no-go-latest.md`
