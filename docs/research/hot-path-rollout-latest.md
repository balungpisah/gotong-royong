# Hot-Path Rollout Report

Date: 2026-02-26T07:50:53Z
Namespace: `monitoring`
Pack C Prometheus URL: `http://127.0.0.1:9090`
Cluster status: `unreachable`
Prometheus status: `skipped(dry-run)`
Go/no-go mode: `dry-run`
Overall result: `PASS`

## Command Status

| Check | Status | Duration |
|---|---|---|
| Pack C readiness gate | SKIPPED | 0s |
| Pack C Stage A kickoff + go/no-go | PASS | 0s |
| Pack C Stage B kickoff + go/no-go | PASS | 1s |
| Pack C Stage C kickoff + go/no-go | PASS | 0s |
| Chat attachment alerts plan | PASS | 0s |

## Commands Executed

- `skipped: scripts/deploy/pack_c_cutover_readiness.sh --namespace monitoring`
- `scripts/deploy/pack_c_stage_kickoff.sh --stage stage-a --namespace monitoring --run-readiness false --run-go-no-go true --go-no-go-prom-url http://127.0.0.1:9090 --go-no-go-step 60s --go-no-go-dry-run true --output docs/research/pack-c-stage-a-kickoff-latest.md --go-no-go-output docs/research/pack-c-stage-a-go-no-go-latest.md`
- `scripts/deploy/pack_c_stage_kickoff.sh --stage stage-b --namespace monitoring --run-readiness false --run-go-no-go true --go-no-go-prom-url http://127.0.0.1:9090 --go-no-go-step 60s --go-no-go-dry-run true --output docs/research/pack-c-stage-b-kickoff-latest.md --go-no-go-output docs/research/pack-c-stage-b-go-no-go-latest.md`
- `scripts/deploy/pack_c_stage_kickoff.sh --stage stage-c --namespace monitoring --run-readiness false --run-go-no-go true --go-no-go-prom-url http://127.0.0.1:9090 --go-no-go-step 60s --go-no-go-dry-run true --output docs/research/pack-c-stage-c-kickoff-latest.md --go-no-go-output docs/research/pack-c-stage-c-go-no-go-latest.md`
- `scripts/deploy/chat_attachment_prometheus_rules.sh --namespace monitoring --dry-run`

## Related Artifacts

- `docs/research/pack-c-cutover-readiness-latest.md`
- `docs/research/pack-c-stage-a-go-no-go-latest.md`
- `docs/research/pack-c-stage-a-kickoff-latest.md`
- `docs/research/pack-c-stage-b-go-no-go-latest.md`
- `docs/research/pack-c-stage-b-kickoff-latest.md`
- `docs/research/pack-c-stage-c-go-no-go-latest.md`
- `docs/research/pack-c-stage-c-kickoff-latest.md`
- `docs/research/chat-attachment-alerts-apply-latest.md`
- `docs/research/chat-attachment-lifecycle-policy-latest.md`
- `docs/research/chat-attachment-lifecycle-verify-latest.md`
