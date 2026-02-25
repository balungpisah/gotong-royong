#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: scripts/deploy/verify_chat_attachment_lifecycle_rules.sh [--required-prefix <prefix>]... [--expect-expire-days <days>] [--output <report-path>]

Behavior:
  - Verifies lifecycle rules exist for each required prefix.
  - Fails if a required prefix has no enabled expiration-days rule.
  - If --expect-expire-days is set, expiration days must match exactly.

Defaults:
  - Required prefixes from CHAT_ATTACHMENT_REQUIRED_PREFIXES (comma-separated), or:
    "${CHAT_ATTACHMENT_S3_PREFIX:-chat-attachments}/${APP_ENV:-development}/"
EOF
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_FILE="${ROOT_DIR}/docs/research/chat-attachment-lifecycle-verify-latest.md"
EXPECT_EXPIRE_DAYS="${EXPECT_EXPIRE_DAYS:-}"
declare -a REQUIRED_PREFIXES=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --required-prefix)
      REQUIRED_PREFIXES+=("${2:-}")
      shift 2
      ;;
    --expect-expire-days)
      EXPECT_EXPIRE_DAYS="${2:-}"
      shift 2
      ;;
    --output)
      OUTPUT_FILE="${2:-}"
      shift 2
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

if [[ -n "${EXPECT_EXPIRE_DAYS}" ]] && { ! [[ "${EXPECT_EXPIRE_DAYS}" =~ ^[0-9]+$ ]] || [[ "${EXPECT_EXPIRE_DAYS}" -le 0 ]]; }; then
  echo "--expect-expire-days must be a positive integer" >&2
  exit 1
fi

if [[ "${#REQUIRED_PREFIXES[@]}" -eq 0 ]]; then
  if [[ -n "${CHAT_ATTACHMENT_REQUIRED_PREFIXES:-}" ]]; then
    IFS=',' read -r -a REQUIRED_PREFIXES <<< "${CHAT_ATTACHMENT_REQUIRED_PREFIXES}"
  else
    default_prefix="${CHAT_ATTACHMENT_S3_PREFIX:-chat-attachments}/${APP_ENV:-development}/"
    REQUIRED_PREFIXES=("${default_prefix}")
  fi
fi

normalize_prefix() {
  local prefix="$1"
  prefix="${prefix#/}"
  prefix="${prefix%/}/"
  printf "%s" "${prefix}"
}

for idx in "${!REQUIRED_PREFIXES[@]}"; do
  normalized="$(normalize_prefix "${REQUIRED_PREFIXES[$idx]}")"
  if [[ -z "${normalized}" || "${normalized}" == "/" ]]; then
    echo "required prefix cannot be empty" >&2
    exit 1
  fi
  REQUIRED_PREFIXES[$idx]="${normalized}"
done

S3_ENDPOINT="${S3_ENDPOINT:-http://127.0.0.1:9000}"
S3_BUCKET="${S3_BUCKET:-gotong-royong-evidence-dev}"
S3_ACCESS_KEY="${S3_ACCESS_KEY:-minioadmin}"
S3_SECRET_KEY="${S3_SECRET_KEY:-minioadmin}"
S3_REGION="${S3_REGION:-us-east-1}"

TARGET_ALIAS="grs3verify"
TARGET="${TARGET_ALIAS}/${S3_BUCKET}"
NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
MC_IMAGE="minio/mc:RELEASE.2025-08-13T08-35-41Z"
MC_CONFIG_DIR="$(mktemp -d)"
USING_LOCAL_MC="false"
if command -v mc >/dev/null 2>&1; then
  USING_LOCAL_MC="true"
else
  require_cmd docker
fi
require_cmd python3
require_cmd rg

cleanup() {
  rm -rf "${MC_CONFIG_DIR}" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

docker_endpoint="${S3_ENDPOINT}"
docker_endpoint="${docker_endpoint/http:\/\/127.0.0.1/http:\/\/host.docker.internal}"
docker_endpoint="${docker_endpoint/http:\/\/localhost/http:\/\/host.docker.internal}"

mc_exec() {
  if [[ "${USING_LOCAL_MC}" == "true" ]]; then
    MC_CONFIG_DIR="${MC_CONFIG_DIR}" mc "$@"
    return
  fi

  local extra_hosts=()
  if [[ "${docker_endpoint}" == *"host.docker.internal"* ]]; then
    extra_hosts+=(--add-host=host.docker.internal:host-gateway)
  fi

  docker run --rm "${extra_hosts[@]}" \
    -v "${MC_CONFIG_DIR}:/tmp/mc" \
    --entrypoint mc \
    "${MC_IMAGE}" \
    --config-dir /tmp/mc \
    "$@"
}

echo "==> Configuring S3 alias"
if [[ "${USING_LOCAL_MC}" == "true" ]]; then
  mc_exec alias set "${TARGET_ALIAS}" "${S3_ENDPOINT}" "${S3_ACCESS_KEY}" "${S3_SECRET_KEY}" >/dev/null
else
  mc_exec alias set "${TARGET_ALIAS}" "${docker_endpoint}" "${S3_ACCESS_KEY}" "${S3_SECRET_KEY}" >/dev/null
fi

echo "==> Reading lifecycle rules for ${TARGET}"
list_output="$(mc_exec ilm rule list --json "${TARGET}" 2>&1 || true)"
if echo "${list_output}" | rg -q "NoSuchLifecycleConfiguration"; then
  lifecycle_missing="true"
  rules_json='{"config":{"Rules":[]}}'
else
  lifecycle_missing="false"
  rules_json="${list_output}"
fi

results_lines="$(
  python3 - "${rules_json}" "${EXPECT_EXPIRE_DAYS}" "${REQUIRED_PREFIXES[@]}" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
expect_days_raw = sys.argv[2]
required_prefixes = sys.argv[3:]
expect_days = int(expect_days_raw) if expect_days_raw else None
rules = ((payload.get("config") or {}).get("Rules") or [])

rows = []
overall_ok = True
for prefix in required_prefixes:
    matching = []
    for rule in rules:
        rule_prefix = ((rule.get("Filter") or {}).get("Prefix") or "")
        if rule_prefix != prefix:
            continue
        days = ((rule.get("Expiration") or {}).get("Days"))
        status = (rule.get("Status") or "").lower()
        if days is None or status != "enabled":
            continue
        matching.append({
            "id": rule.get("ID") or "",
            "days": int(days),
        })

    if not matching:
        overall_ok = False
        rows.append((prefix, "", "", "FAIL", "no enabled expiration-days rule"))
        continue

    # Choose highest expiry when multiple rules exist for same prefix.
    best = sorted(matching, key=lambda item: item["days"], reverse=True)[0]
    if expect_days is not None and best["days"] != expect_days:
        overall_ok = False
        rows.append((prefix, best["id"], str(best["days"]), "FAIL", f"expected {expect_days}"))
    else:
        rows.append((prefix, best["id"], str(best["days"]), "PASS", "ok"))

print(f"OVERALL={'PASS' if overall_ok else 'FAIL'}")
for row in rows:
    print("|".join(row))
PY
)"

overall_status="$(printf "%s\n" "${results_lines}" | rg '^OVERALL=' | sed 's/OVERALL=//')"
table_rows="$(printf "%s\n" "${results_lines}" | rg -v '^OVERALL=')"

required_prefixes_rendered=""
for prefix in "${REQUIRED_PREFIXES[@]}"; do
  required_prefixes_rendered+="- \`${prefix}\`\n"
done

mkdir -p "$(dirname "${OUTPUT_FILE}")"
{
  echo "# Chat Attachment Lifecycle Verification Report"
  echo
  echo "Date: ${NOW_UTC}"
  echo "Status: \`${overall_status}\`"
  echo "Bucket: \`${S3_BUCKET}\`"
  echo "Endpoint: \`${S3_ENDPOINT}\`"
  if [[ -n "${EXPECT_EXPIRE_DAYS}" ]]; then
    echo "Expected expire days: \`${EXPECT_EXPIRE_DAYS}\`"
  else
    echo "Expected expire days: \`any enabled expiration-days rule\`"
  fi
  echo
  echo "## Required Prefixes"
  echo
  printf "%b" "${required_prefixes_rendered}"
  echo
  echo "## Rule Verification"
  echo
  echo "| Prefix | Rule ID | Expire Days | Status | Notes |"
  echo "|---|---|---:|---|---|"
  if [[ -n "${table_rows}" ]]; then
    while IFS='|' read -r prefix rule_id expire_days status notes; do
      echo "| \`${prefix}\` | \`${rule_id}\` | ${expire_days:-n/a} | ${status} | ${notes} |"
    done <<< "${table_rows}"
  else
    echo "| n/a | n/a | n/a | FAIL | no lifecycle rules returned |"
  fi
  echo
  echo "## Context"
  echo
  echo "- Lifecycle config missing on bucket: \`${lifecycle_missing}\`"
  echo "- Verification command: \`scripts/deploy/verify_chat_attachment_lifecycle_rules.sh\`"
  echo "- Related runbook: \`docs/deployment/chat-attachment-storage-lifecycle-runbook.md\`"
  echo "- Debt tracker: \`docs/research/frontend-hot-path-integration-debt.md\` (CHAT-API-004)"
} > "${OUTPUT_FILE}"

if [[ "${overall_status}" != "PASS" ]]; then
  echo "lifecycle verification failed; see ${OUTPUT_FILE}" >&2
  exit 1
fi

echo "==> OK (chat attachment lifecycle verification passed)"
echo "report: ${OUTPUT_FILE}"
