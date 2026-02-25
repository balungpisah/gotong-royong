# Monitoring Manifests

This directory contains deployable Prometheus Operator rule manifests for Pack C feed involvement fallback cutover.

## Files

- `prometheusrule-pack-c-stage-a.yaml` — baseline stage (`DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true`)
- `prometheusrule-pack-c-stage-b.yaml` — canary stage (subset with fallback disabled)
- `prometheusrule-pack-c-stage-c.yaml` — full cutover stage (`DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=false` on all replicas)

## Prerequisites

- Prometheus Operator CRDs are installed (`monitoring.coreos.com/v1`).
- `gotong-api` metrics are scraped into Prometheus.
- Namespace `monitoring` exists, or manifests are adjusted to your namespace.

## Rollout commands

Stage A:

```bash
kubectl apply -f deploy/monitoring/prometheusrule-pack-c-stage-a.yaml
kubectl delete -f deploy/monitoring/prometheusrule-pack-c-stage-b.yaml --ignore-not-found
kubectl delete -f deploy/monitoring/prometheusrule-pack-c-stage-c.yaml --ignore-not-found
```

Stage B:

```bash
kubectl apply -f deploy/monitoring/prometheusrule-pack-c-stage-b.yaml
kubectl delete -f deploy/monitoring/prometheusrule-pack-c-stage-a.yaml --ignore-not-found
kubectl delete -f deploy/monitoring/prometheusrule-pack-c-stage-c.yaml --ignore-not-found
```

Stage C:

```bash
kubectl apply -f deploy/monitoring/prometheusrule-pack-c-stage-c.yaml
kubectl delete -f deploy/monitoring/prometheusrule-pack-c-stage-a.yaml --ignore-not-found
kubectl delete -f deploy/monitoring/prometheusrule-pack-c-stage-b.yaml --ignore-not-found
```

## Validation

```bash
kubectl get prometheusrule -n monitoring | rg gotong-pack-c-cutover
kubectl describe prometheusrule gotong-pack-c-cutover-stage-c -n monitoring
```

Runbook and threshold references:
- `docs/deployment/feed-involvement-fallback-removal-runbook.md`
- `docs/deployment/feed-involvement-fallback-alert-thresholds.md`
