# Feed Involvement Cutover Alert Thresholds (Pack C)

Last updated: 2026-02-25

Purpose:
- Define one-page Grafana + Alertmanager thresholds for the Pack C fallback-off rollout.
- Keep thresholds aligned with `docs/deployment/feed-involvement-fallback-removal-runbook.md`.

References:
- `docs/deployment/feed-involvement-fallback-removal-runbook.md`
- `docs/database/hot-path-pack-c-feed-participant-edge-design-v1.md`
- `docs/database/hot-path-query-shape-slo-matrix.md`
- `deploy/monitoring/README.md`

## 1) Canonical PromQL signals

```promql
# Total involvement traffic (feed + search)
sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m]))
```

```promql
# Edge error ratio
(sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search",lane="edge_error"}[5m])) or vector(0))
/
clamp_min((sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m])) or vector(0)), 1)
```

```promql
# Edge partial ratio
(sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search",lane="edge_partial"}[5m])) or vector(0))
/
clamp_min((sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m])) or vector(0)), 1)
```

```promql
# Shadow mismatch growth (fallback-on stages only)
sum(increase(gotong_api_feed_involvement_shadow_mismatch_total[30m])) or vector(0)
```

```promql
# Feed p95 latency (seconds)
max(gotong_api_http_request_duration_seconds{route="/v1/feed",method="GET",quantile="0.95"})
```

```promql
# Feed 5xx ratio
(sum(rate(gotong_api_http_errors_total{route="/v1/feed",method="GET"}[5m])) or vector(0))
/
clamp_min((sum(rate(gotong_api_http_requests_total{route="/v1/feed",method="GET"}[5m])) or vector(0)), 1)
```

```promql
# Fallback lane presence (must be zero after full cutover)
sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search",lane=~"fallback|legacy"}[5m]))
```

## 2) Stage thresholds

| Signal | Stage A (fallback ON) | Stage B (canary OFF subset) | Stage C (full OFF) |
| --- | --- | --- | --- |
| Edge error ratio | warn > `0.001` for `10m`, crit > `0.005` for `5m` | warn > `0.0005` for `10m`, crit > `0.002` for `5m` | warn > `0.0001` for `10m`, crit > `0.0005` for `5m` |
| Edge partial ratio | warn > `0.003` for `10m`, crit > `0.01` for `5m` | warn > `0.001` for `10m`, crit > `0.005` for `5m` | warn > `0.0005` for `10m`, crit > `0.002` for `5m` |
| Shadow mismatch `increase(...[30m])` | warn `> 3`, crit `> 10` | warn `> 1`, crit `> 5` | not applicable |
| Feed p95 latency | warn > `0.18s` for `15m`, crit > `0.25s` for `10m` | same as Stage A | same as Stage A |
| Feed 5xx ratio | warn > `0.01` for `10m`, crit > `0.02` for `5m` | same as Stage A | same as Stage A |
| Fallback/legacy lane RPS | allowed; trend should be flat or decreasing | expected to decrease as OFF share grows | must be `0` (any sustained non-zero is no-go) |

Use these as rollout gates:
- Do not advance stage when any critical threshold is firing.
- Roll back to `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true` if Stage B/C critical alerts persist for two consecutive evaluation windows.

## 3) Alertmanager rule template

```yaml
groups:
  - name: gotong-pack-c-cutover
    rules:
      - alert: GotongFeedInvolvementEdgeErrorRatioHigh
        expr: |
          (
            (sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search",lane="edge_error"}[5m])) or vector(0))
            /
            clamp_min((sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m])) or vector(0)), 1)
          ) > 0.0005
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Pack C edge_error ratio is above threshold"

      - alert: GotongFeedInvolvementEdgePartialRatioHigh
        expr: |
          (
            (sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search",lane="edge_partial"}[5m])) or vector(0))
            /
            clamp_min((sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m])) or vector(0)), 1)
          ) > 0.002
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Pack C edge_partial ratio is above threshold"

      - alert: GotongFeedP95LatencySLOBreach
        expr: |
          max(gotong_api_http_request_duration_seconds{route="/v1/feed",method="GET",quantile="0.95"}) > 0.18
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "GET /v1/feed p95 is above 180ms"

      - alert: GotongFeed5xxRatioHigh
        expr: |
          (
            (sum(rate(gotong_api_http_errors_total{route="/v1/feed",method="GET"}[5m])) or vector(0))
            /
            clamp_min((sum(rate(gotong_api_http_requests_total{route="/v1/feed",method="GET"}[5m])) or vector(0)), 1)
          ) > 0.01
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "GET /v1/feed 5xx ratio is above 1%"
```

Operational note:
- Add a stage label (for example `pack_c_stage`) in deployment metadata so Stage A/B/C thresholds can be selected cleanly in Grafana and rule routing.
- For full cutover, add an explicit alert for fallback/legacy lane RPS `> 0` for `15m`.
