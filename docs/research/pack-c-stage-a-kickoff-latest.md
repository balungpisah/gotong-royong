# Pack C Stage A Kickoff Report

Date: 2026-02-25T09:52:49Z
Namespace: `monitoring`
Observation window target: 24h
Stage summary: Baseline rollout with fallback ON across all replicas.

## Kickoff Summary

| Item | Result |
|---|---|
| Readiness gate | pass |
| Kubernetes cluster status | unreachable |
| Stage rule action mode | dry-run |
| Stage rule action result | dry_run_only |
| Go/no-go gate | pass |
| Go/no-go mode | dry-run |
| Go/no-go window | 24h |
| Go/no-go report | docs/research/pack-c-stage-a-go-no-go-latest.md |

## Commands Executed

1. `scripts/deploy/pack_c_cutover_readiness.sh --namespace monitoring`
2. `scripts/deploy/pack_c_prometheus_rules.sh --stage stage-a --namespace monitoring --dry-run`
3. `scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-a --prom-url http://127.0.0.1:9090 --window 24h --step 60s --output docs/research/pack-c-stage-a-go-no-go-latest.md --dry-run`

## Stage Checklist (24h)

- Keep `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true` on all replicas.
- Observe lane usage, shadow mismatch, feed/search latency for at least the target window.
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
- `docs/research/pack-c-stage-a-go-no-go-latest.md`
