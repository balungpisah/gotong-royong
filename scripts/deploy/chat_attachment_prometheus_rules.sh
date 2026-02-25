#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
Usage:
  scripts/deploy/chat_attachment_prometheus_rules.sh [--namespace <ns>] [--dry-run]

Examples:
  scripts/deploy/chat_attachment_prometheus_rules.sh --namespace monitoring
  scripts/deploy/chat_attachment_prometheus_rules.sh --namespace monitoring --dry-run
EOF
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

NAMESPACE="monitoring"
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
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

TARGET="deploy/monitoring/prometheusrule-chat-attachment-lifecycle.yaml"
if [[ ! -f "${TARGET}" ]]; then
  echo "missing manifest: ${TARGET}" >&2
  exit 1
fi

if [[ "$DRY_RUN" == "1" ]]; then
  echo "[dry-run] kubectl apply -n ${NAMESPACE} -f ${TARGET}"
  exit 0
fi

require_cmd kubectl

echo "Applying chat attachment lifecycle alerts in namespace ${NAMESPACE}"
kubectl apply -n "${NAMESPACE}" -f "${TARGET}"
echo "Active PrometheusRule resources in ${NAMESPACE}:"
kubectl get prometheusrule -n "${NAMESPACE}" | rg 'gotong-chat-attachment-lifecycle|NAME' || true
