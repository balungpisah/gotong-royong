#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

NAMESPACE="${1:-monitoring}"

echo "==> Verifying Pack C monitoring assets"
scripts/deploy/verify_pack_c_monitoring_assets.sh

echo "==> Verifying stage rollout plans (dry-run)"
scripts/deploy/pack_c_prometheus_rules.sh --stage stage-a --namespace "${NAMESPACE}" --dry-run
scripts/deploy/pack_c_prometheus_rules.sh --stage stage-b --namespace "${NAMESPACE}" --dry-run
scripts/deploy/pack_c_prometheus_rules.sh --stage stage-c --namespace "${NAMESPACE}" --dry-run

echo "Pack C slice gate: OK"
