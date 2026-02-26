#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ROLLOUT_ENV=""
FRONTEND_URL=""
DRY_RUN=0
OUTPUT_FILE=""
GATE_OUTPUT=""

usage() {
  cat <<'EOF'
Usage:
  scripts/deploy/frontend_live_cutover_rollout.sh --env <staging|production> [--frontend-url <url>] [--dry-run] [--output <path>] [--gate-output <path>]

Examples:
  scripts/deploy/frontend_live_cutover_rollout.sh --env staging --dry-run
  scripts/deploy/frontend_live_cutover_rollout.sh --env staging --frontend-url https://staging-gotong.example.com
  scripts/deploy/frontend_live_cutover_rollout.sh --env production --frontend-url https://gotong.example.com
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --env)
      shift
      ROLLOUT_ENV="${1:-}"
      ;;
    --frontend-url)
      shift
      FRONTEND_URL="${1:-}"
      ;;
    --dry-run)
      DRY_RUN=1
      ;;
    --output)
      shift
      OUTPUT_FILE="${1:-}"
      ;;
    --gate-output)
      shift
      GATE_OUTPUT="${1:-}"
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

if [[ -z "${ROLLOUT_ENV}" ]]; then
  echo "--env is required (staging|production)" >&2
  usage
  exit 1
fi

if [[ "${ROLLOUT_ENV}" != "staging" && "${ROLLOUT_ENV}" != "production" ]]; then
  echo "--env must be one of: staging|production" >&2
  exit 1
fi

MODE="live"
if [[ "${DRY_RUN}" == "1" ]]; then
  MODE="dry-run"
fi

if [[ "${MODE}" == "live" && -z "${FRONTEND_URL}" ]]; then
  echo "--frontend-url is required in live mode" >&2
  exit 1
fi

if [[ -z "${OUTPUT_FILE}" ]]; then
  OUTPUT_FILE="docs/research/frontend-live-cutover-${ROLLOUT_ENV}-latest.md"
fi

if [[ -z "${GATE_OUTPUT}" ]]; then
  GATE_OUTPUT="docs/research/frontend-live-cutover-gate-${ROLLOUT_ENV}-latest.md"
fi

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

require_cmd bash
require_cmd date

NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
FAIL_COUNT=0
STEP_ROWS=""
COMMAND_ROWS=""

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
  finished_at="$(date +%s)"
  duration_s="$((finished_at - started_at))"

  STEP_ROWS+=$'\n'"| ${label} | ${status} | ${duration_s}s |"
  COMMAND_ROWS+=$'\n'"- \`${command}\`"
}

mkdir -p "$(dirname -- "${OUTPUT_FILE}")"
mkdir -p "$(dirname -- "${GATE_OUTPUT}")"

if [[ "${MODE}" == "dry-run" ]]; then
  run_step \
    "Frontend live cutover gate (dry-run)" \
    "scripts/deploy/frontend_live_cutover_gate.sh --dry-run --output ${GATE_OUTPUT}"
  FRONTEND_URL_DISPLAY="n/a (dry-run)"
else
  run_step \
    "Frontend live cutover gate (${ROLLOUT_ENV})" \
    "scripts/deploy/frontend_live_cutover_gate.sh --frontend-url '${FRONTEND_URL}' --output ${GATE_OUTPUT}"
  FRONTEND_URL_DISPLAY="${FRONTEND_URL}"
fi

RESULT="PASS"
if [[ "${FAIL_COUNT}" -gt 0 ]]; then
  RESULT="FAIL"
fi

cp "${GATE_OUTPUT}" "docs/research/frontend-live-cutover-gate-latest.md"

{
  echo "# Frontend Live Cutover Rollout (${ROLLOUT_ENV})"
  echo
  echo "Date: ${NOW_UTC}"
  echo "Mode: \`${MODE}\`"
  echo "Frontend URL: \`${FRONTEND_URL_DISPLAY}\`"
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
  echo "## Artifacts"
  echo
  echo "- \`${GATE_OUTPUT}\`"
  echo "- \`docs/research/frontend-live-cutover-gate-latest.md\` (updated from env-specific gate report)"
  echo
  echo "## Context"
  echo
  echo "- Slice tracker: \`docs/research/frontend-live-cutover-001-latest.md\`"
  echo "- Backlog: \`docs/research/frontend-service-api-cutover-backlog.md\`"
} > "${OUTPUT_FILE}"

echo "Wrote ${OUTPUT_FILE}"
echo "Gate report: ${GATE_OUTPUT}"

if [[ "${RESULT}" != "PASS" ]]; then
  echo "frontend live cutover rollout failed; see ${OUTPUT_FILE}" >&2
  exit 1
fi
