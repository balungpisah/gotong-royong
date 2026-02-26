#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

FRONTEND_URL=""
OUTPUT_FILE="docs/research/frontend-live-cutover-gate-latest.md"
DRY_RUN=0

usage() {
  cat <<'EOF'
Usage:
  scripts/deploy/frontend_live_cutover_gate.sh [--frontend-url <url>] [--output <path>] [--dry-run]

Examples:
  scripts/deploy/frontend_live_cutover_gate.sh --dry-run
  scripts/deploy/frontend_live_cutover_gate.sh --frontend-url https://app.gotong.local
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --frontend-url)
      shift
      FRONTEND_URL="${1:-}"
      ;;
    --output)
      shift
      OUTPUT_FILE="${1:-}"
      ;;
    --dry-run)
      DRY_RUN=1
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

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

require_cmd bash
require_cmd rg
require_cmd date

NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
FAIL_COUNT=0
STEP_ROWS=""
COMMAND_ROWS=""
LAST_STEP_STATUS="PASS"
MODE="dry-run"

if [[ "${DRY_RUN}" == "0" ]]; then
  MODE="live"
  if [[ -z "${FRONTEND_URL}" ]]; then
    echo "--frontend-url is required in live mode" >&2
    exit 1
  fi
fi

run_step() {
  local label="$1"
  local command="$2"
  local started_at
  local finished_at
  local duration_s
  local status

  echo "==> ${label}"
  started_at="$(date +%s)"
  if ( eval "${command}" ); then
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

run_step \
  "Production guard enforces API-only services" \
  "rg -q \"assertApiEnabledInProduction\" apps/web/src/lib/services/index.ts && \
   rg -q \"PUBLIC_GR_USE_API_NOTIFICATIONS\" apps/web/src/lib/services/index.ts && \
   rg -q \"PUBLIC_GR_USE_API_FEED\" apps/web/src/lib/services/index.ts && \
   rg -q \"PUBLIC_GR_USE_API_CHAT\" apps/web/src/lib/services/index.ts && \
   rg -q \"PUBLIC_GR_USE_API_USER\" apps/web/src/lib/services/index.ts && \
   rg -q \"PUBLIC_GR_USE_API_TRIAGE\" apps/web/src/lib/services/index.ts && \
   rg -q \"PUBLIC_GR_USE_API_SIGNAL\" apps/web/src/lib/services/index.ts && \
   rg -q \"PUBLIC_GR_USE_API_GROUP\" apps/web/src/lib/services/index.ts"

run_step \
  "External live smoke entrypoints exist" \
  "rg -q \"test:e2e:live-api:external\" apps/web/package.json && \
   rg -q \"web-test-e2e-live-api-external\" justfile && \
   test -f apps/web/tests/e2e/live-api-proxy.spec.ts"

if [[ "${DRY_RUN}" == "0" ]]; then
  require_cmd bun
  run_step \
    "Playwright live smoke via deployed frontend host" \
    "cd apps/web && PLAYWRIGHT_EXTERNAL_BASE_URL='${FRONTEND_URL}' bun run test:e2e:live-api:external"
else
  STEP_ROWS+=$'\n'"| Playwright live smoke via deployed frontend host | SKIPPED | 0s |"
  if [[ -n "${FRONTEND_URL}" ]]; then
    COMMAND_ROWS+=$'\n'"- \`skipped: cd apps/web && PLAYWRIGHT_EXTERNAL_BASE_URL='${FRONTEND_URL}' bun run test:e2e:live-api:external\`"
  else
    COMMAND_ROWS+=$'\n'"- \`skipped: cd apps/web && PLAYWRIGHT_EXTERNAL_BASE_URL=<frontend-url> bun run test:e2e:live-api:external\`"
  fi
fi

RESULT="PASS"
if [[ "${FAIL_COUNT}" -gt 0 ]]; then
  RESULT="FAIL"
fi

mkdir -p "$(dirname -- "${OUTPUT_FILE}")"
{
  echo "# Frontend Live Cutover Gate Report"
  echo
  echo "Date: ${NOW_UTC}"
  echo "Mode: \`${MODE}\`"
  echo "Frontend URL: \`${FRONTEND_URL:-n/a}\`"
  echo "Result: \`${RESULT}\`"
  echo
  echo "## Command Status"
  echo
  echo "| Check | Status | Duration |"
  echo "|---|---|---|${STEP_ROWS}"
  echo
  echo "## Commands Executed"
  echo "${COMMAND_ROWS}"
  echo
  echo "## Context"
  echo
  echo "- Related backlog: \`docs/research/frontend-service-api-cutover-backlog.md\`"
  echo "- Related debt tracker: \`docs/research/frontend-hot-path-integration-debt.md\`"
  echo "- Related smoke spec: \`apps/web/tests/e2e/live-api-proxy.spec.ts\`"
} > "${OUTPUT_FILE}"

echo "Wrote ${OUTPUT_FILE}"

if [[ "${RESULT}" != "PASS" ]]; then
  echo "frontend live cutover gate failed; see ${OUTPUT_FILE}" >&2
  exit 1
fi
