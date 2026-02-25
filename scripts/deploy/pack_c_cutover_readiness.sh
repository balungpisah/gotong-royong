#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

NAMESPACE="monitoring"
OUTPUT_FILE="docs/research/pack-c-cutover-readiness-latest.md"

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
    *)
      echo "unknown argument: $1" >&2
      echo "usage: $0 [--namespace <k8s-namespace>] [--output <report-path>]" >&2
      exit 1
      ;;
  esac
  shift
done

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

require_cmd bash
require_cmd cargo
require_cmd date

STEP_ROWS=""
NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

run_step() {
  local label="$1"
  shift
  local started_at
  local finished_at
  local duration_s

  echo "==> ${label}"
  started_at="$(date +%s)"
  "$@"
  finished_at="$(date +%s)"
  duration_s="$((finished_at - started_at))"
  STEP_ROWS+=$'\n'"| ${label} | PASS | ${duration_s}s |"
}

run_step "Pack C monitoring asset gate" \
  scripts/deploy/pack_c_slice_gate.sh "${NAMESPACE}"

run_step "Participant-edge backfill dry-run" \
  cargo run -p gotong-worker -- feed-participant-edge-backfill --dry-run --page-size 1000 --progress-every 1000

run_step "Involvement fallback on/off smoke" \
  scripts/smoke/feed_involvement_edge_cutover_live.sh

run_step "Feed involvement benchmark" \
  scripts/surrealdb-feed-involvement-bench.sh docs/research/surrealdb-feed-involvement-bench-latest.md

mkdir -p "$(dirname "${OUTPUT_FILE}")"

cat > "${OUTPUT_FILE}" <<EOF
# Pack C Cutover Readiness Report

Date: ${NOW_UTC}
Namespace: \`${NAMESPACE}\`
Purpose: one-command execution of mandatory pre-cutover checks from \`docs/deployment/feed-involvement-fallback-removal-runbook.md\`.

## Command Status

| Check | Status | Duration |
|---|---|---|${STEP_ROWS}

## Executed Commands

1. \`scripts/deploy/pack_c_slice_gate.sh ${NAMESPACE}\`
2. \`cargo run -p gotong-worker -- feed-participant-edge-backfill --dry-run --page-size 1000 --progress-every 1000\`
3. \`scripts/smoke/feed_involvement_edge_cutover_live.sh\`
4. \`scripts/surrealdb-feed-involvement-bench.sh docs/research/surrealdb-feed-involvement-bench-latest.md\`

## Related Artifacts

- \`docs/research/surrealdb-feed-involvement-bench-latest.md\`
- \`docs/deployment/feed-involvement-fallback-removal-runbook.md\`
- \`docs/deployment/feed-involvement-fallback-alert-thresholds.md\`
EOF

echo "Wrote ${OUTPUT_FILE}"
