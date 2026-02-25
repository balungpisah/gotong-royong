#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

NAMESPACE="monitoring"
PROM_URL=""
GO_NO_GO_STEP="60s"
GO_NO_GO_DRY_RUN="false"
APPLY_CHAT_ALERTS="true"
APPLY_CHAT_LIFECYCLE="false"
REQUIRE_CLUSTER="true"
RUN_READINESS="true"
OUTPUT_FILE="docs/research/hot-path-rollout-latest.md"

usage() {
  cat <<'EOF'
usage: scripts/deploy/hot_path_rollout.sh [--prom-url <url>] [--namespace <k8s-namespace>] [--go-no-go-step <duration>] [--go-no-go-dry-run <true|false>] [--apply-chat-alerts <true|false>] [--apply-chat-lifecycle <true|false>] [--require-cluster <true|false>] [--run-readiness <true|false>] [--output <report-path>]

notes:
  - --prom-url is required only when --go-no-go-dry-run=false.
EOF
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --namespace)
      shift
      NAMESPACE="${1:-}"
      ;;
    --prom-url)
      shift
      PROM_URL="${1:-}"
      ;;
    --go-no-go-step)
      shift
      GO_NO_GO_STEP="${1:-}"
      ;;
    --go-no-go-dry-run)
      shift
      GO_NO_GO_DRY_RUN="${1:-}"
      ;;
    --apply-chat-alerts)
      shift
      APPLY_CHAT_ALERTS="${1:-}"
      ;;
    --apply-chat-lifecycle)
      shift
      APPLY_CHAT_LIFECYCLE="${1:-}"
      ;;
    --require-cluster)
      shift
      REQUIRE_CLUSTER="${1:-}"
      ;;
    --run-readiness)
      shift
      RUN_READINESS="${1:-}"
      ;;
    --output)
      shift
      OUTPUT_FILE="${1:-}"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
  shift
done

for boolean_arg in GO_NO_GO_DRY_RUN APPLY_CHAT_ALERTS APPLY_CHAT_LIFECYCLE REQUIRE_CLUSTER RUN_READINESS; do
  value="${!boolean_arg}"
  if [[ "${value}" != "true" && "${value}" != "false" ]]; then
    echo "invalid boolean value for ${boolean_arg}: ${value}" >&2
    exit 1
  fi
done

if [[ "${GO_NO_GO_DRY_RUN}" == "false" && -z "${PROM_URL}" ]]; then
  echo "--prom-url is required when --go-no-go-dry-run=false" >&2
  usage
  exit 1
fi

if [[ "${GO_NO_GO_DRY_RUN}" == "true" && -z "${PROM_URL}" ]]; then
  PROM_URL="http://127.0.0.1:9090"
fi

require_cmd bash
require_cmd date
require_cmd curl

NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
CLUSTER_STATUS="unreachable"
PROM_STATUS="unknown"
STEP_ROWS=""
COMMAND_ROWS=""
FAIL_COUNT=0
LAST_STEP_STATUS=""

run_step() {
  local label="$1"
  local command="$2"
  local started_at
  local finished_at
  local duration_s
  local status

  echo "==> ${label}"
  started_at="$(date +%s)"
  if eval "${command}"; then
    status="PASS"
  else
    status="FAIL"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  LAST_STEP_STATUS="${status}"
  finished_at="$(date +%s)"
  duration_s="$((finished_at - started_at))"

  STEP_ROWS+=$'\n'"| ${label} | ${status} | ${duration_s}s |"
  COMMAND_ROWS+=$'\n'"- \`${command}\`"
}

if command -v kubectl >/dev/null 2>&1 && kubectl version --request-timeout=5s >/dev/null 2>&1; then
  CLUSTER_STATUS="reachable"
fi

if [[ "${REQUIRE_CLUSTER}" == "true" && "${CLUSTER_STATUS}" != "reachable" ]]; then
  echo "kubernetes cluster is unreachable and --require-cluster=true" >&2
  exit 1
fi

if [[ "${GO_NO_GO_DRY_RUN}" == "false" ]]; then
  if curl -fsS "${PROM_URL%/}/-/ready" >/dev/null; then
    PROM_STATUS="ready"
  else
    PROM_STATUS="unreachable"
    echo "prometheus is unreachable at ${PROM_URL} while --go-no-go-dry-run=false" >&2
    exit 1
  fi
else
  PROM_STATUS="skipped(dry-run)"
fi

if [[ "${RUN_READINESS}" == "true" ]]; then
  run_step \
    "Pack C readiness gate" \
    "scripts/deploy/pack_c_cutover_readiness.sh --namespace ${NAMESPACE}"
else
  COMMAND_ROWS+=$'\n'"- \`skipped: scripts/deploy/pack_c_cutover_readiness.sh --namespace ${NAMESPACE}\`"
  STEP_ROWS+=$'\n'"| Pack C readiness gate | SKIPPED | 0s |"
fi

run_step \
  "Pack C Stage A kickoff + go/no-go" \
  "scripts/deploy/pack_c_stage_kickoff.sh --stage stage-a --namespace ${NAMESPACE} --run-readiness ${RUN_READINESS} --run-go-no-go true --go-no-go-prom-url ${PROM_URL} --go-no-go-step ${GO_NO_GO_STEP} --go-no-go-dry-run ${GO_NO_GO_DRY_RUN} --output docs/research/pack-c-stage-a-kickoff-latest.md --go-no-go-output docs/research/pack-c-stage-a-go-no-go-latest.md"

run_step \
  "Pack C Stage B kickoff + go/no-go" \
  "scripts/deploy/pack_c_stage_kickoff.sh --stage stage-b --namespace ${NAMESPACE} --run-readiness false --run-go-no-go true --go-no-go-prom-url ${PROM_URL} --go-no-go-step ${GO_NO_GO_STEP} --go-no-go-dry-run ${GO_NO_GO_DRY_RUN} --output docs/research/pack-c-stage-b-kickoff-latest.md --go-no-go-output docs/research/pack-c-stage-b-go-no-go-latest.md"

run_step \
  "Pack C Stage C kickoff + go/no-go" \
  "scripts/deploy/pack_c_stage_kickoff.sh --stage stage-c --namespace ${NAMESPACE} --run-readiness false --run-go-no-go true --go-no-go-prom-url ${PROM_URL} --go-no-go-step ${GO_NO_GO_STEP} --go-no-go-dry-run ${GO_NO_GO_DRY_RUN} --output docs/research/pack-c-stage-c-kickoff-latest.md --go-no-go-output docs/research/pack-c-stage-c-go-no-go-latest.md"

if [[ "${APPLY_CHAT_ALERTS}" == "true" ]]; then
  CHAT_ALERTS_COMMAND=""
  CHAT_ALERTS_MODE="apply"
  if [[ "${GO_NO_GO_DRY_RUN}" == "true" ]]; then
    CHAT_ALERTS_COMMAND="scripts/deploy/chat_attachment_prometheus_rules.sh --namespace ${NAMESPACE} --dry-run"
    CHAT_ALERTS_MODE="dry-run"
    run_step \
      "Chat attachment alerts plan" \
      "${CHAT_ALERTS_COMMAND}"
  else
    CHAT_ALERTS_COMMAND="scripts/deploy/chat_attachment_prometheus_rules.sh --namespace ${NAMESPACE}"
    run_step \
      "Chat attachment alerts apply" \
      "${CHAT_ALERTS_COMMAND}"
  fi
  cat > "docs/research/chat-attachment-alerts-apply-latest.md" <<EOF
# Chat Attachment Alerts Apply Report

Date: ${NOW_UTC}
Namespace: \`${NAMESPACE}\`
Mode: \`${CHAT_ALERTS_MODE}\`
Status: \`${LAST_STEP_STATUS}\`
Command: \`${CHAT_ALERTS_COMMAND}\`
EOF
  echo "Wrote docs/research/chat-attachment-alerts-apply-latest.md"
fi

if [[ "${APPLY_CHAT_LIFECYCLE}" == "true" ]]; then
  if [[ "${GO_NO_GO_DRY_RUN}" == "true" ]]; then
    run_step \
      "Chat attachment lifecycle policy plan" \
      "scripts/deploy/chat_attachment_lifecycle_policy.sh --dry-run --output docs/research/chat-attachment-lifecycle-policy-latest.md"
  else
    run_step \
      "Chat attachment lifecycle policy apply" \
      "scripts/deploy/chat_attachment_lifecycle_policy.sh --output docs/research/chat-attachment-lifecycle-policy-latest.md"
    run_step \
      "Chat attachment lifecycle verify" \
      "scripts/deploy/verify_chat_attachment_lifecycle_rules.sh"
  fi
fi

result="PASS"
if [[ "${FAIL_COUNT}" -gt 0 ]]; then
  result="FAIL"
fi

mkdir -p "$(dirname "${OUTPUT_FILE}")"
cat > "${OUTPUT_FILE}" <<EOF
# Hot-Path Rollout Report

Date: ${NOW_UTC}
Namespace: \`${NAMESPACE}\`
Pack C Prometheus URL: \`${PROM_URL}\`
Cluster status: \`${CLUSTER_STATUS}\`
Prometheus status: \`${PROM_STATUS}\`
Go/no-go mode: \`$([[ "${GO_NO_GO_DRY_RUN}" == "true" ]] && echo "dry-run" || echo "live")\`
Overall result: \`${result}\`

## Command Status

| Check | Status | Duration |
|---|---|---|${STEP_ROWS}

## Commands Executed
${COMMAND_ROWS}

## Related Artifacts

- \`docs/research/pack-c-cutover-readiness-latest.md\`
- \`docs/research/pack-c-stage-a-go-no-go-latest.md\`
- \`docs/research/pack-c-stage-a-kickoff-latest.md\`
- \`docs/research/pack-c-stage-b-go-no-go-latest.md\`
- \`docs/research/pack-c-stage-b-kickoff-latest.md\`
- \`docs/research/pack-c-stage-c-go-no-go-latest.md\`
- \`docs/research/pack-c-stage-c-kickoff-latest.md\`
- \`docs/research/chat-attachment-alerts-apply-latest.md\`
- \`docs/research/chat-attachment-lifecycle-policy-latest.md\`
- \`docs/research/chat-attachment-lifecycle-verify-latest.md\`
EOF

echo "Wrote ${OUTPUT_FILE}"
if [[ "${FAIL_COUNT}" -gt 0 ]]; then
  exit 1
fi
