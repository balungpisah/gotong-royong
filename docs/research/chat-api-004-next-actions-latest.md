# CHAT-API-004 Next Actions

Date: 2026-02-25
Status: `PENDING_EXTERNAL_INFRA`

## Completed Locally

- `just dev-db-up`
- `just release-gates-surreal` → `docs/research/release-gates-surreal-latest.md` (`PASS`)
- `just chat-attachment-alerts-verify` (`PASS`)
- `scripts/deploy/hot_path_rollout.sh --namespace monitoring --go-no-go-dry-run true --go-no-go-step 60s --apply-chat-alerts true --apply-chat-lifecycle true --require-cluster false --run-readiness false` → `docs/research/hot-path-rollout-latest.md` (`PASS`)
- `scripts/deploy/verify_surreal_release_gate_branch_protection.sh --repo balungpisah/gotong-royong --branch main` → `docs/research/branch-protection-surreal-release-gates-latest.md` (`PASS`)

## Remaining to Close Debt

Run on staging and production clusters (with real Prometheus URL + kubectl access):

```bash
just hot-path-rollout monitoring http://<prometheus-host>:9090 60s false true true true true
```

After each environment rollout:

- Archive updated evidence artifacts under `docs/research/`:
  - `hot-path-rollout-latest.md`
  - `chat-attachment-alerts-apply-latest.md`
  - `chat-attachment-lifecycle-policy-latest.md`
  - `chat-attachment-lifecycle-verify-latest.md`

## Exit Condition

`CHAT-API-004` can be moved to closed once staging + production live rollout evidence is archived and reviewed.
