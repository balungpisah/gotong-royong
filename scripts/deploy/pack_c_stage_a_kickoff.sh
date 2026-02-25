#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

NAMESPACE="monitoring"
OUTPUT_FILE="docs/research/pack-c-stage-a-kickoff-latest.md"
OBSERVATION_HOURS="24"

while [[ $# -gt 0 ]]; do
  case "$1" in
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
    *)
      echo "unknown argument: $1" >&2
      echo "usage: $0 [--namespace <k8s-namespace>] [--output <report-path>] [--observation-hours <hours>]" >&2
      exit 1
      ;;
  esac
  shift
done

NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
MODE="dry-run"
CLUSTER_STATUS="unreachable"
STAGE_A_RESULT="not executed"
READINESS_RESULT="not executed"

run_and_capture() {
  local cmd="$1"
  if output="$(eval "$cmd" 2>&1)"; then
    echo "$output"
    return 0
  fi
  echo "$output" >&2
  return 1
}

echo "==> Running Pack C readiness gate"
if run_and_capture "scripts/deploy/pack_c_cutover_readiness.sh --namespace ${NAMESPACE}"; then
  READINESS_RESULT="pass"
else
  READINESS_RESULT="fail"
fi

if command -v kubectl >/dev/null 2>&1 && kubectl version --request-timeout=5s >/dev/null 2>&1; then
  CLUSTER_STATUS="reachable"
  MODE="apply"
fi

if [[ "$MODE" == "apply" ]]; then
  echo "==> Applying Stage A Prometheus rules in namespace ${NAMESPACE}"
  if run_and_capture "scripts/deploy/pack_c_prometheus_rules.sh --stage stage-a --namespace ${NAMESPACE}"; then
    STAGE_A_RESULT="applied"
  else
    STAGE_A_RESULT="apply_failed"
  fi
else
  echo "==> Kubernetes cluster unavailable; running Stage A dry-run"
  if run_and_capture "scripts/deploy/pack_c_prometheus_rules.sh --stage stage-a --namespace ${NAMESPACE} --dry-run"; then
    STAGE_A_RESULT="dry_run_only"
  else
    STAGE_A_RESULT="dry_run_failed"
  fi
fi

mkdir -p "$(dirname "${OUTPUT_FILE}")"
cat > "${OUTPUT_FILE}" <<EOF
# Pack C Stage A Kickoff Report

Date: ${NOW_UTC}
Namespace: \`${NAMESPACE}\`
Observation window target: ${OBSERVATION_HOURS}h

## Kickoff Summary

| Item | Result |
|---|---|
| Readiness gate | ${READINESS_RESULT} |
| Kubernetes cluster status | ${CLUSTER_STATUS} |
| Stage A rule action mode | ${MODE} |
| Stage A rule action result | ${STAGE_A_RESULT} |

## Commands Executed

1. \`scripts/deploy/pack_c_cutover_readiness.sh --namespace ${NAMESPACE}\`
2. \`scripts/deploy/pack_c_prometheus_rules.sh --stage stage-a --namespace ${NAMESPACE}${MODE:+$( [[ "$MODE" == "dry-run" ]] && printf " --dry-run" )}\`

## Stage A Observation Checklist (${OBSERVATION_HOURS}h)

- Keep \`DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED=true\` on all replicas.
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
EOF

echo "Wrote ${OUTPUT_FILE}"

if [[ "${READINESS_RESULT}" != "pass" || "${STAGE_A_RESULT}" == "apply_failed" || "${STAGE_A_RESULT}" == "dry_run_failed" ]]; then
  exit 1
fi
