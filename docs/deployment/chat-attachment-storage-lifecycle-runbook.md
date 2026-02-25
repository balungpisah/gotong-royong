# Chat Attachment Storage Lifecycle Runbook

Last updated: 2026-02-25

Objective:
- Close `CHAT-API-004` by operationalizing retention/lifecycle for chat attachments stored in S3/MinIO.
- Keep chat hot-path fast while preventing unbounded object growth.

## Scope

Applies to:
- `POST /v1/chat/attachments/upload`
- `GET /v1/chat/attachments/:attachment_id/download`
- Storage mode `CHAT_ATTACHMENT_STORAGE_BACKEND=s3` (or `auto` when S3 is healthy)

Related artifact:
- `docs/research/chat-attachment-s3-smoke-latest.md`
- `docs/research/chat-attachment-lifecycle-policy-latest.md`
- `docs/research/chat-attachment-alerts-apply-latest.md`

## Required Configuration

- `CHAT_ATTACHMENT_STORAGE_BACKEND=s3` (staging/prod recommended)
- `CHAT_ATTACHMENT_S3_PREFIX=chat-attachments` (or environment-specific prefix)
- `S3_ENDPOINT`, `S3_BUCKET`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`, `S3_REGION`

Recommended prefix convention:
- `chat-attachments/<env>/...`

## Lifecycle Policy Target

Minimum policy:
- Expire chat attachment objects after a fixed retention window (example: 30–90 days).
- Cover both payload and metadata objects:
  - `chat-attachments/<env>/*.bin`
  - `chat-attachments/<env>/*.json`

Policy guidance:
- Keep retention longer than expected user revisit window.
- Use shorter retention in development to limit storage churn.
- Apply policy at bucket lifecycle level (IaC preferred), not manual ad-hoc cleanup.

## Rollout Steps

1. **Enforce S3 backend**
   - Set `CHAT_ATTACHMENT_STORAGE_BACKEND=s3` in target environment.
   - Restart API and verify startup does not fallback.

2. **Apply lifecycle rules**
   - Plan:
     - `just chat-attachment-lifecycle-plan`
   - Apply:
     - `just chat-attachment-lifecycle-apply`
   - Optional overrides:
     - `APP_ENV=<env> EXPIRE_DAYS=<days> CHAT_ATTACHMENT_S3_PREFIX=<prefix> scripts/deploy/chat_attachment_lifecycle_policy.sh`
   - Version-control policy in infrastructure repo / deployment manifests.
   - Confirm report: `docs/research/chat-attachment-lifecycle-policy-latest.md`

3. **Validate runtime path**
   - Verify required lifecycle prefixes:
     - `just chat-attachment-lifecycle-verify`
     - Optional strict check: `EXPECT_EXPIRE_DAYS=<days> CHAT_ATTACHMENT_REQUIRED_PREFIXES=<prefix1,prefix2> scripts/deploy/verify_chat_attachment_lifecycle_rules.sh`
   - Confirm report: `docs/research/chat-attachment-lifecycle-verify-latest.md`
   - Run: `scripts/smoke/chat_attachment_s3_live.sh`
   - Confirm report: `docs/research/chat-attachment-s3-smoke-latest.md`

4. **Observe and alert**
   - Track object count/size growth for attachment prefix.
   - Alert on sustained unexpected growth or failed lifecycle execution.
   - PrometheusRule manifest:
     - `deploy/monitoring/prometheusrule-chat-attachment-lifecycle.yaml`
   - Apply monitoring rules:
     - `just chat-attachment-alerts-plan`
     - `just chat-attachment-alerts-apply`
   - Verify monitoring assets:
     - `just chat-attachment-alerts-verify`

5. **Enforce branch protection gate**
   - Verify repository branch protection requires `CI / surreal-release-gates`:
     - `just chat-attachment-branch-protection-plan <owner/repo>`
     - `just chat-attachment-branch-protection-check <owner/repo>`
   - Confirm report: `docs/research/branch-protection-surreal-release-gates-latest.md`

## CI Gate Integration

- Automated gate: `.github/workflows/ci.yml` → job `surreal-release-gates`.
- Gate flow in CI:
  1. Start local SurrealDB/Redis/MinIO.
  2. Ensure attachment bucket exists.
  3. Apply lifecycle rule for `chat-attachments/<env>/`.
  4. Run `scripts/release-gates-surreal.sh` (go/no-go + lifecycle verify + S3 smoke).
- Override per-environment inputs using repository variables:
  - `RELEASE_GATES_APP_ENV`
  - `RELEASE_GATES_CHAT_ATTACHMENT_S3_PREFIX`
  - `RELEASE_GATES_CHAT_ATTACHMENT_REQUIRED_PREFIXES`
  - `RELEASE_GATES_EXPIRE_DAYS`
  - `RELEASE_GATES_EXPECT_EXPIRE_DAYS`
  - `RELEASE_GATES_S3_BUCKET`

## Strict Staging/Prod Checklist (`CHAT-API-004`)

Use this exact sequence. Do not start production until staging is complete and reviewed.

### 0) Local preflight (operator machine)

```bash
just dev-db-up
docker compose -f compose.dev.yaml --profile storage up -d minio
just release-gates-surreal
just chat-attachment-alerts-verify
```

Expected artifacts (must be `PASS`):
- `docs/research/release-gates-surreal-latest.md`
- `docs/research/chat-attachment-lifecycle-verify-latest.md`
- `docs/research/chat-attachment-s3-smoke-latest.md`
- `docs/research/chat-attachment-alerts-apply-latest.md` (dry-run is acceptable in local preflight)

### 1) Staging live rollout

Set environment-specific values:

```bash
export ROLLOUT_ENV=staging
export K8S_NAMESPACE=monitoring
export PROM_URL=http://<staging-prometheus-host>:9090
export GH_REPO=<owner/repo>
export GH_BRANCH=main
export APP_ENV=staging
export CHAT_ATTACHMENT_S3_PREFIX=chat-attachments
```

Execute:

```bash
just hot-path-rollout "${K8S_NAMESPACE}" "${PROM_URL}" 60s false true true true true
just chat-attachment-branch-protection-check "${GH_REPO}" "${GH_BRANCH}"
```

Expected artifacts (must be `PASS`):
- `docs/research/hot-path-rollout-latest.md`
- `docs/research/chat-attachment-alerts-apply-latest.md` (`Mode: apply`)
- `docs/research/chat-attachment-lifecycle-policy-latest.md` (`Mode: apply`)
- `docs/research/chat-attachment-lifecycle-verify-latest.md`
- `docs/research/branch-protection-surreal-release-gates-latest.md`

Archive staging evidence before prod:

```bash
ts="$(date -u +%Y%m%dT%H%M%SZ)"
cp docs/research/hot-path-rollout-latest.md "docs/research/hot-path-rollout-staging-${ts}.md"
cp docs/research/chat-attachment-alerts-apply-latest.md "docs/research/chat-attachment-alerts-apply-staging-${ts}.md"
cp docs/research/chat-attachment-lifecycle-policy-latest.md "docs/research/chat-attachment-lifecycle-policy-staging-${ts}.md"
cp docs/research/chat-attachment-lifecycle-verify-latest.md "docs/research/chat-attachment-lifecycle-verify-staging-${ts}.md"
cp docs/research/branch-protection-surreal-release-gates-latest.md "docs/research/branch-protection-surreal-release-gates-staging-${ts}.md"
```

### 2) Production live rollout

Only after staging passes and is approved.

Set production values:

```bash
export ROLLOUT_ENV=production
export K8S_NAMESPACE=monitoring
export PROM_URL=http://<prod-prometheus-host>:9090
export GH_REPO=<owner/repo>
export GH_BRANCH=main
export APP_ENV=production
export CHAT_ATTACHMENT_S3_PREFIX=chat-attachments
```

Execute:

```bash
just hot-path-rollout "${K8S_NAMESPACE}" "${PROM_URL}" 60s false true true true true
just chat-attachment-branch-protection-check "${GH_REPO}" "${GH_BRANCH}"
```

Expected artifacts (must be `PASS`):
- `docs/research/hot-path-rollout-latest.md`
- `docs/research/chat-attachment-alerts-apply-latest.md` (`Mode: apply`)
- `docs/research/chat-attachment-lifecycle-policy-latest.md` (`Mode: apply`)
- `docs/research/chat-attachment-lifecycle-verify-latest.md`
- `docs/research/branch-protection-surreal-release-gates-latest.md`

Archive production evidence:

```bash
ts="$(date -u +%Y%m%dT%H%M%SZ)"
cp docs/research/hot-path-rollout-latest.md "docs/research/hot-path-rollout-production-${ts}.md"
cp docs/research/chat-attachment-alerts-apply-latest.md "docs/research/chat-attachment-alerts-apply-production-${ts}.md"
cp docs/research/chat-attachment-lifecycle-policy-latest.md "docs/research/chat-attachment-lifecycle-policy-production-${ts}.md"
cp docs/research/chat-attachment-lifecycle-verify-latest.md "docs/research/chat-attachment-lifecycle-verify-production-${ts}.md"
cp docs/research/branch-protection-surreal-release-gates-latest.md "docs/research/branch-protection-surreal-release-gates-production-${ts}.md"
```

### 3) Fail-fast rules

- Stop rollout immediately if `hot-path-rollout` exits non-zero.
- Do not continue if branch-protection check is not `PASS`.
- If lifecycle verify fails, re-apply lifecycle policy and rerun verify before proceeding.
- If alerts apply fails, keep debt `CHAT-API-004` open.

## Exit Criteria (`CHAT-API-004`)

- Lifecycle policy is deployed for attachment prefix in staging/prod.
- Lifecycle verification report passes for required release prefixes.
- Smoke script passes with `CHAT_ATTACHMENT_STORAGE_BACKEND=s3`.
- Branch protection requires `CI / surreal-release-gates` (verified by script report).
- Runbook link is referenced from debt tracker and deployment docs.
