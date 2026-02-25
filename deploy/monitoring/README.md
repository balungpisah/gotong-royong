# Monitoring Manifests

This directory contains deployable Prometheus Operator rule manifests for Pack C feed involvement fallback cutover.

## Files

- `prometheusrule-pack-c-stage-a.yaml` — baseline stage (`DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true`)
- `prometheusrule-pack-c-stage-b.yaml` — canary stage (subset with fallback disabled)
- `prometheusrule-pack-c-stage-c.yaml` — full cutover stage (`DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=false` on all replicas)
- `grafana-pack-c-cutover-dashboard.json` — Grafana dashboard for Pack C lane/latency/error cutover watch

## Prerequisites

- Prometheus Operator CRDs are installed (`monitoring.coreos.com/v1`).
- `gotong-api` metrics are scraped into Prometheus.
- Namespace `monitoring` exists, or manifests are adjusted to your namespace.

## Rollout commands

Stage A:

```bash
scripts/deploy/pack_c_prometheus_rules.sh --stage stage-a --namespace monitoring
```

Stage B:

```bash
scripts/deploy/pack_c_prometheus_rules.sh --stage stage-b --namespace monitoring
```

Stage C:

```bash
scripts/deploy/pack_c_prometheus_rules.sh --stage stage-c --namespace monitoring
```

Dry-run plan:

```bash
scripts/deploy/pack_c_prometheus_rules.sh --stage stage-b --namespace monitoring --dry-run
```

Equivalent `just` shortcuts:

```bash
just pack-c-alerts-stage-a
just pack-c-alerts-stage-b
just pack-c-alerts-stage-c
just pack-c-alerts-plan stage=stage-c
```

## Validation

```bash
kubectl get prometheusrule -n monitoring | rg gotong-pack-c-cutover
kubectl describe prometheusrule gotong-pack-c-cutover-stage-c -n monitoring
```

Grafana import:

- Dashboard UID: `gotong-pack-c-cutover`
- Import file: `deploy/monitoring/grafana-pack-c-cutover-dashboard.json`
- Datasource placeholder in JSON: `${DS_PROMETHEUS}`

Runbook and threshold references:
- `docs/deployment/feed-involvement-fallback-removal-runbook.md`
- `docs/deployment/feed-involvement-fallback-alert-thresholds.md`
