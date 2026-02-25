#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

NAMESPACE="monitoring"
STAGE=""
OUTPUT_FILE=""
OBSERVATION_HOURS=""
RUN_READINESS=""
RUN_GO_NO_GO=""
GO_NO_GO_PROM_URL="http://127.0.0.1:9090"
GO_NO_GO_WINDOW=""
GO_NO_GO_STEP="60s"
GO_NO_GO_DRY_RUN=""
GO_NO_GO_OUTPUT=""

usage() {
  cat <<'EOF'
usage: scripts/deploy/pack_c_stage_kickoff.sh --stage <stage-a|stage-b|stage-c> [--namespace <k8s-namespace>] [--output <report-path>] [--observation-hours <hours>] [--run-readiness <true|false>] [--run-go-no-go <true|false>] [--go-no-go-prom-url <url>] [--go-no-go-window <duration>] [--go-no-go-step <duration>] [--go-no-go-dry-run <true|false>] [--go-no-go-output <report-path>]
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --stage)
      shift
      STAGE="${1:-}"
      ;;
    --namespace)
      shift
      NAMESPACE="${1:-}"
      ;;
    --output)
      shift
      OUTPUT_FILE="${1:-}"
      ;;
    --observation-hours)
      shift
      OBSERVATION_HOURS="${1:-}"
      ;;
    --run-readiness)
      shift
      RUN_READINESS="${1:-}"
      ;;
    --run-go-no-go)
      shift
      RUN_GO_NO_GO="${1:-}"
      ;;
    --go-no-go-prom-url)
      shift
      GO_NO_GO_PROM_URL="${1:-}"
      ;;
    --go-no-go-window)
      shift
      GO_NO_GO_WINDOW="${1:-}"
      ;;
    --go-no-go-step)
      shift
      GO_NO_GO_STEP="${1:-}"
      ;;
    --go-no-go-dry-run)
      shift
      GO_NO_GO_DRY_RUN="${1:-}"
      ;;
    --go-no-go-output)
      shift
      GO_NO_GO_OUTPUT="${1:-}"
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

case "${STAGE}" in
  stage-a|stage-b|stage-c) ;;
  *)
    echo "--stage is required and must be one of: stage-a|stage-b|stage-c" >&2
    usage
    exit 1
    ;;
esac

if [[ -z "${OBSERVATION_HOURS}" ]]; then
  case "${STAGE}" in
    stage-a) OBSERVATION_HOURS="24" ;;
    stage-b) OBSERVATION_HOURS="4" ;;
    stage-c) OBSERVATION_HOURS="24" ;;
  esac
fi

if [[ -z "${RUN_READINESS}" ]]; then
  if [[ "${STAGE}" == "stage-a" ]]; then
    RUN_READINESS="true"
  else
    RUN_READINESS="false"
  fi
fi

if [[ "${RUN_READINESS}" != "true" && "${RUN_READINESS}" != "false" ]]; then
  echo "--run-readiness must be true or false" >&2
  exit 1
fi

if [[ -z "${RUN_GO_NO_GO}" ]]; then
  RUN_GO_NO_GO="false"
fi

if [[ "${RUN_GO_NO_GO}" != "true" && "${RUN_GO_NO_GO}" != "false" ]]; then
  echo "--run-go-no-go must be true or false" >&2
  exit 1
fi

if [[ -z "${GO_NO_GO_WINDOW}" ]]; then
  GO_NO_GO_WINDOW="${OBSERVATION_HOURS}h"
fi

if [[ -z "${GO_NO_GO_OUTPUT}" ]]; then
  GO_NO_GO_OUTPUT="docs/research/pack-c-${STAGE}-go-no-go-latest.md"
fi

if [[ -z "${OUTPUT_FILE}" ]]; then
  OUTPUT_FILE="docs/research/pack-c-${STAGE}-kickoff-latest.md"
fi

NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
MODE="dry-run"
CLUSTER_STATUS="unreachable"
STAGE_RESULT="not executed"
READINESS_RESULT="skipped"
readiness_cmd="not run"
stage_cmd=""
stage_title=""
stage_summary=""
GO_NO_GO_RESULT="skipped"
GO_NO_GO_MODE="n/a"
go_no_go_cmd="not run"
go_no_go_report="n/a"

run_and_capture() {
  local cmd="$1"
  if output="$(eval "$cmd" 2>&1)"; then
    echo "$output"
    return 0
  fi
  echo "$output" >&2
  return 1
}

if [[ "${RUN_READINESS}" == "true" ]]; then
  readiness_cmd="scripts/deploy/pack_c_cutover_readiness.sh --namespace ${NAMESPACE}"
  echo "==> Running Pack C readiness gate"
  if run_and_capture "${readiness_cmd}"; then
    READINESS_RESULT="pass"
  else
    READINESS_RESULT="fail"
  fi
fi

if command -v kubectl >/dev/null 2>&1 && kubectl version --request-timeout=5s >/dev/null 2>&1; then
  CLUSTER_STATUS="reachable"
  MODE="apply"
fi

if [[ -z "${GO_NO_GO_DRY_RUN}" ]]; then
  if [[ "${MODE}" == "dry-run" ]]; then
    GO_NO_GO_DRY_RUN="true"
  else
    GO_NO_GO_DRY_RUN="false"
  fi
fi

if [[ "${GO_NO_GO_DRY_RUN}" != "true" && "${GO_NO_GO_DRY_RUN}" != "false" ]]; then
  echo "--go-no-go-dry-run must be true or false" >&2
  exit 1
fi

case "${STAGE}" in
  stage-a)
    stage_title="Stage A"
    stage_summary="Baseline rollout with fallback ON across all replicas."
    stage_checklist=$'- Keep `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true` on all replicas.\n- Observe lane usage, shadow mismatch, feed/search latency for at least the target window.'
    ;;
  stage-b)
    stage_title="Stage B"
    stage_summary="Canary rollout with fallback OFF on a subset of replicas."
    stage_checklist=$'- Set `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=false` on 5–10% of replicas.\n- Increase to 25% then 50% only if no critical alerts and SLO remains stable.'
    ;;
  stage-c)
    stage_title="Stage C"
    stage_summary="Full rollout with fallback OFF on all replicas."
    stage_checklist=$'- Set `DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=false` on all replicas.\n- Keep enhanced monitoring for at least the target window before considering fallback code removal.'
    ;;
esac

stage_cmd="scripts/deploy/pack_c_prometheus_rules.sh --stage ${STAGE} --namespace ${NAMESPACE}"
if [[ "${MODE}" == "apply" ]]; then
  echo "==> Applying ${stage_title} Prometheus rules in namespace ${NAMESPACE}"
  if run_and_capture "${stage_cmd}"; then
    STAGE_RESULT="applied"
  else
    STAGE_RESULT="apply_failed"
  fi
else
  echo "==> Kubernetes cluster unavailable; running ${stage_title} dry-run"
  if run_and_capture "${stage_cmd} --dry-run"; then
    STAGE_RESULT="dry_run_only"
    stage_cmd="${stage_cmd} --dry-run"
  else
    STAGE_RESULT="dry_run_failed"
  fi
fi

if [[ "${RUN_GO_NO_GO}" == "true" ]]; then
  GO_NO_GO_MODE="live"
  go_no_go_cmd="scripts/deploy/pack_c_stage_go_no_go.sh --stage ${STAGE} --prom-url ${GO_NO_GO_PROM_URL} --window ${GO_NO_GO_WINDOW} --step ${GO_NO_GO_STEP} --output ${GO_NO_GO_OUTPUT}"
  if [[ "${GO_NO_GO_DRY_RUN}" == "true" ]]; then
    GO_NO_GO_MODE="dry-run"
    go_no_go_cmd="${go_no_go_cmd} --dry-run"
  fi
  go_no_go_report="${GO_NO_GO_OUTPUT}"
  echo "==> Running ${stage_title} go/no-go gate (${GO_NO_GO_MODE})"
  if run_and_capture "${go_no_go_cmd}"; then
    GO_NO_GO_RESULT="pass"
  else
    GO_NO_GO_RESULT="fail"
  fi
fi

mkdir -p "$(dirname "${OUTPUT_FILE}")"
cat > "${OUTPUT_FILE}" <<EOF
# Pack C ${stage_title} Kickoff Report

Date: ${NOW_UTC}
Namespace: \`${NAMESPACE}\`
Observation window target: ${OBSERVATION_HOURS}h
Stage summary: ${stage_summary}

## Kickoff Summary

| Item | Result |
|---|---|
| Readiness gate | ${READINESS_RESULT} |
| Kubernetes cluster status | ${CLUSTER_STATUS} |
| Stage rule action mode | ${MODE} |
| Stage rule action result | ${STAGE_RESULT} |
| Go/no-go gate | ${GO_NO_GO_RESULT} |
| Go/no-go mode | ${GO_NO_GO_MODE} |
| Go/no-go window | ${GO_NO_GO_WINDOW} |
| Go/no-go report | ${go_no_go_report} |

## Commands Executed

1. \`${readiness_cmd}\`
2. \`${stage_cmd}\`
3. \`${go_no_go_cmd}\`

## Stage Checklist (${OBSERVATION_HOURS}h)

${stage_checklist}
- Watch lane distribution:
  - \`sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m])) by (lane)\`
- Watch shadow mismatch:
  - \`increase(gotong_api_feed_involvement_shadow_mismatch_total[30m])\`
- Watch feed SLO:
  - \`histogram_quantile(0.95, sum(rate(gotong_api_http_request_duration_seconds_bucket{route="/v1/feed",method="GET"}[5m])) by (le))\`

References:
- \`docs/deployment/feed-involvement-fallback-removal-runbook.md\`
- \`docs/deployment/feed-involvement-fallback-alert-thresholds.md\`
- \`docs/research/pack-c-cutover-readiness-latest.md\`
- \`${GO_NO_GO_OUTPUT}\`
EOF

echo "Wrote ${OUTPUT_FILE}"

if [[ "${READINESS_RESULT}" == "fail" || "${STAGE_RESULT}" == "apply_failed" || "${STAGE_RESULT}" == "dry_run_failed" || "${GO_NO_GO_RESULT}" == "fail" ]]; then
  exit 1
fi
