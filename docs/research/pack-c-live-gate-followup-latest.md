# Pack C Live Go/No-Go Follow-up

Date: 2026-02-25T13:51:14Z
Status: `blocked`

## Current blocker

- Stage A/B/C go/no-go now run in **live mode** against local Prometheus and return `GO`.
- Remaining blockers for real rollout are:
  - Kubernetes cluster is unreachable from this environment (`docs/research/pack-c-stage-a-kickoff-latest.md`, `docs/research/pack-c-stage-b-kickoff-latest.md`, and `docs/research/pack-c-stage-c-kickoff-latest.md` all show `Kubernetes cluster status | unreachable`).
  - Stage kickoff automation is still running in dry-run mode for rule apply (no live `kubectl` context here).

## Evidence

- `docs/research/pack-c-stage-a-kickoff-latest.md`
- `docs/research/pack-c-stage-a-go-no-go-latest.md`
- `docs/research/pack-c-stage-b-kickoff-latest.md`
- `docs/research/pack-c-stage-b-go-no-go-latest.md`
- `docs/research/pack-c-stage-c-kickoff-latest.md`
- `docs/research/pack-c-stage-c-go-no-go-latest.md`

## Next test (real cluster + full windows)

Run in target environment (staging first, then production):

```bash
just hot-path-rollout monitoring http://<prometheus-host>:9090 60s false true false true true
```

```bash
just pack-c-stage-a-end-to-end monitoring http://<prometheus-host>:9090 60s false
just pack-c-stage-b-end-to-end monitoring http://<prometheus-host>:9090 60s false
just pack-c-stage-c-end-to-end monitoring http://<prometheus-host>:9090 60s false
```

```bash
scripts/deploy/pack_c_stage_kickoff.sh \
  --stage <stage-a|stage-b|stage-c> \
  --namespace monitoring \
  --run-go-no-go true \
  --go-no-go-prom-url http://<prometheus-host>:9090 \
  --go-no-go-step 60s \
  --go-no-go-dry-run false
```

Current achieved criteria:
- `docs/research/pack-c-stage-a-go-no-go-latest.md` decision is `GO` in `live` mode.
- `docs/research/pack-c-stage-b-go-no-go-latest.md` decision is `GO` in `live` mode.
- `docs/research/pack-c-stage-c-go-no-go-latest.md` decision is `GO` in `live` mode.
- `just pack-c-slice-gate` passes after monitoring query alignment fixes.

Remaining success criteria:
- Each stage kickoff report shows `Kubernetes cluster status | reachable`.
- Each stage kickoff report shows `Stage rule action mode | apply` and `Stage rule action result | applied`.
- Stage windows use rollout defaults in real environments (A=24h, B=4h, C=24h), not local 5m/10m validation windows.
