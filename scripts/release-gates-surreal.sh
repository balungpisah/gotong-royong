#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_FILE="${1:-${ROOT_DIR}/docs/research/release-gates-surreal-latest.md}"
NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

run_step() {
  local name="$1"
  shift
  local start_ts end_ts elapsed status
  start_ts="$(date +%s)"
  if "$@"; then
    status="PASS"
  else
    status="FAIL"
  fi
  end_ts="$(date +%s)"
  elapsed="$((end_ts - start_ts))s"
  STEP_ROWS+=$'\n'"| ${name} | ${status} | ${elapsed} |"
  if [[ "${status}" == "FAIL" ]]; then
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

STEP_ROWS=""
FAIL_COUNT=0

echo "==> Running SurrealDB go/no-go"
run_step "SurrealDB go/no-go" \
  "${ROOT_DIR}/scripts/surrealdb-go-no-go.sh" "${ROOT_DIR}/docs/research/surrealdb-go-no-go-latest.md"

echo "==> Verifying chat attachment lifecycle prefixes"
run_step "Chat attachment lifecycle verify" \
  "${ROOT_DIR}/scripts/deploy/verify_chat_attachment_lifecycle_rules.sh" \
  --output "${ROOT_DIR}/docs/research/chat-attachment-lifecycle-verify-latest.md"

echo "==> Running chat attachment S3 live smoke"
run_step "Chat attachment S3 smoke" \
  "${ROOT_DIR}/scripts/smoke/chat_attachment_s3_live.sh" \
  "${ROOT_DIR}/docs/research/chat-attachment-s3-smoke-latest.md"

mkdir -p "$(dirname "${OUTPUT_FILE}")"
cat > "${OUTPUT_FILE}" <<EOF
# Surreal Release Gates Report

Date: ${NOW_UTC}
Mode: \`live\`

## Command Status

| Check | Status | Duration |
|---|---|---|${STEP_ROWS}

## Related Artifacts

- \`docs/research/surrealdb-go-no-go-latest.md\`
- \`docs/research/chat-attachment-lifecycle-verify-latest.md\`
- \`docs/research/chat-attachment-s3-smoke-latest.md\`
- \`docs/deployment/chat-attachment-storage-lifecycle-runbook.md\`
EOF

if [[ "${FAIL_COUNT}" -gt 0 ]]; then
  echo "release gates failed (${FAIL_COUNT} step(s)); see ${OUTPUT_FILE}" >&2
  exit 1
fi

echo "Surreal release gates passed"
echo "Report: ${OUTPUT_FILE}"
