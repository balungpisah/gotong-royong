#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
Usage:
  scripts/deploy/pack_c_prometheus_rules.sh --stage <stage-a|stage-b|stage-c> [--namespace <ns>] [--dry-run]

Examples:
  scripts/deploy/pack_c_prometheus_rules.sh --stage stage-a --namespace monitoring
  scripts/deploy/pack_c_prometheus_rules.sh --stage stage-c --namespace monitoring
EOF
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

STAGE=""
NAMESPACE="monitoring"
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --stage)
      STAGE="${2:-}"
      shift 2
      ;;
    --namespace)
      NAMESPACE="${2:-}"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
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
done

if [[ -z "$STAGE" ]]; then
  echo "--stage is required" >&2
  usage
  exit 1
fi

case "$STAGE" in
  stage-a|stage-b|stage-c) ;;
  *)
    echo "invalid stage: $STAGE (expected stage-a|stage-b|stage-c)" >&2
    exit 1
    ;;
esac

MANIFEST_DIR="deploy/monitoring"
TARGET="${MANIFEST_DIR}/prometheusrule-pack-c-${STAGE}.yaml"

all_stages=(stage-a stage-b stage-c)
stale_manifests=()
for stage in "${all_stages[@]}"; do
  if [[ "$stage" != "$STAGE" ]]; then
    stale_manifests+=("${MANIFEST_DIR}/prometheusrule-pack-c-${stage}.yaml")
  fi
done

if [[ "$DRY_RUN" == "1" ]]; then
  echo "[dry-run] kubectl apply -n ${NAMESPACE} -f ${TARGET}"
  for stale in "${stale_manifests[@]}"; do
    echo "[dry-run] kubectl delete -n ${NAMESPACE} -f ${stale} --ignore-not-found"
  done
  exit 0
fi

require_cmd kubectl

echo "Applying Pack C alerts for ${STAGE} in namespace ${NAMESPACE}"
kubectl apply -n "${NAMESPACE}" -f "${TARGET}"

for stale in "${stale_manifests[@]}"; do
  kubectl delete -n "${NAMESPACE}" -f "${stale}" --ignore-not-found
done

echo "Active PrometheusRule resources in ${NAMESPACE}:"
kubectl get prometheusrule -n "${NAMESPACE}" | rg 'gotong-pack-c-cutover|NAME' || true
