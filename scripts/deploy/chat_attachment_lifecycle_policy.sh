#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: scripts/deploy/chat_attachment_lifecycle_policy.sh [--expire-days <days>] [--prefix <prefix>] [--dry-run] [--output <report-path>]

Environment defaults:
  S3_ENDPOINT, S3_BUCKET, S3_ACCESS_KEY, S3_SECRET_KEY, S3_REGION
  CHAT_ATTACHMENT_S3_PREFIX (default: chat-attachments)
  APP_ENV (default: development)
EOF
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

require_cmd python3

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
DRY_RUN="false"
EXPIRE_DAYS="${EXPIRE_DAYS:-45}"
BASE_PREFIX="${CHAT_ATTACHMENT_S3_PREFIX:-chat-attachments}"
APP_ENV_NAME="${APP_ENV:-development}"
PREFIX="${BASE_PREFIX%/}/${APP_ENV_NAME}/"
OUTPUT_FILE="${ROOT_DIR}/docs/research/chat-attachment-lifecycle-policy-latest.md"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --expire-days)
      EXPIRE_DAYS="${2:-}"
      shift 2
      ;;
    --prefix)
      PREFIX="${2:-}"
      shift 2
      ;;
    --dry-run)
      DRY_RUN="true"
      shift
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

if ! [[ "${EXPIRE_DAYS}" =~ ^[0-9]+$ ]] || [[ "${EXPIRE_DAYS}" -le 0 ]]; then
  echo "--expire-days must be a positive integer" >&2
  exit 1
fi

S3_ENDPOINT="${S3_ENDPOINT:-http://127.0.0.1:9000}"
S3_BUCKET="${S3_BUCKET:-gotong-royong-evidence-dev}"
S3_ACCESS_KEY="${S3_ACCESS_KEY:-minioadmin}"
S3_SECRET_KEY="${S3_SECRET_KEY:-minioadmin}"
S3_REGION="${S3_REGION:-us-east-1}"
TARGET_ALIAS="grs3"
TARGET="${TARGET_ALIAS}/${S3_BUCKET}"
NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

if [[ -z "${PREFIX}" ]]; then
  echo "--prefix cannot be empty" >&2
  exit 1
fi

# Normalize prefix to '<path>/'
PREFIX="${PREFIX#/}"
PREFIX="${PREFIX%/}/"

MC_IMAGE="minio/mc:RELEASE.2025-08-13T08-35-41Z"
MC_CONFIG_DIR="$(mktemp -d)"
USING_LOCAL_MC="false"
if command -v mc >/dev/null 2>&1; then
  USING_LOCAL_MC="true"
else
  require_cmd docker
fi

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

echo "==> Reading existing lifecycle rules for ${TARGET}"
list_output="$(mc_exec ilm rule list --json "${TARGET}" 2>&1 || true)"
has_lifecycle="true"
if echo "${list_output}" | rg -q "NoSuchLifecycleConfiguration"; then
  has_lifecycle="false"
fi

rules_rows=""
if [[ "${has_lifecycle}" == "true" ]]; then
  rules_rows="$(
    python3 - "${list_output}" <<'PY'
import json
import sys
payload = json.loads(sys.argv[1])
rules = ((payload.get("config") or {}).get("Rules") or [])
for rule in rules:
    rule_id = rule.get("ID") or ""
    prefix = ((rule.get("Filter") or {}).get("Prefix") or "")
    days = ((rule.get("Expiration") or {}).get("Days"))
    days = "" if days is None else str(days)
    print(f"{rule_id}|{prefix}|{days}")
PY
  )"
fi

before_rule_count=0
if [[ -n "${rules_rows}" ]]; then
  before_rule_count="$(printf "%s\n" "${rules_rows}" | rg -c '.*')"
fi

declare -a prefix_rule_ids=()
declare -a ids_to_remove=()
desired_exists="false"

if [[ -n "${rules_rows}" ]]; then
  while IFS='|' read -r rule_id rule_prefix rule_days; do
    [[ -z "${rule_id}" ]] && continue
    if [[ "${rule_prefix}" != "${PREFIX}" ]]; then
      continue
    fi
    prefix_rule_ids+=("${rule_id}")
    if [[ "${rule_days}" == "${EXPIRE_DAYS}" && "${desired_exists}" == "false" ]]; then
      desired_exists="true"
      continue
    fi
    ids_to_remove+=("${rule_id}")
  done <<< "${rules_rows}"
fi

add_needed="false"
if [[ "${desired_exists}" == "false" ]]; then
  add_needed="true"
fi

status="PASS"
if [[ "${DRY_RUN}" == "true" ]]; then
  echo "[dry-run] lifecycle plan for ${TARGET}"
  if [[ "${#ids_to_remove[@]}" -gt 0 ]]; then
    for rule_id in "${ids_to_remove[@]}"; do
      echo "[dry-run] mc ilm rule rm --id ${rule_id} ${TARGET}"
    done
  else
    echo "[dry-run] no removal needed"
  fi
  if [[ "${add_needed}" == "true" ]]; then
    echo "[dry-run] mc ilm rule add --prefix ${PREFIX} --expire-days ${EXPIRE_DAYS} ${TARGET}"
  else
    echo "[dry-run] add not needed (matching rule already exists)"
  fi
else
  echo "==> Applying lifecycle policy changes"
  for rule_id in "${ids_to_remove[@]}"; do
    mc_exec ilm rule rm --id "${rule_id}" "${TARGET}" >/dev/null
  done
  if [[ "${add_needed}" == "true" ]]; then
    mc_exec ilm rule add --prefix "${PREFIX}" --expire-days "${EXPIRE_DAYS}" "${TARGET}" >/dev/null
  fi
fi

after_output="$(mc_exec ilm rule list --json "${TARGET}" 2>&1 || true)"
after_rule_count=0
prefix_rule_count_after=0
if ! echo "${after_output}" | rg -q "NoSuchLifecycleConfiguration"; then
  after_rows="$(
    python3 - "${after_output}" <<'PY'
import json
import sys
payload = json.loads(sys.argv[1])
rules = ((payload.get("config") or {}).get("Rules") or [])
for rule in rules:
    rule_id = rule.get("ID") or ""
    prefix = ((rule.get("Filter") or {}).get("Prefix") or "")
    days = ((rule.get("Expiration") or {}).get("Days"))
    days = "" if days is None else str(days)
    print(f"{rule_id}|{prefix}|{days}")
PY
  )"
  if [[ -n "${after_rows}" ]]; then
    after_rule_count="$(printf "%s\n" "${after_rows}" | rg -c '.*')"
    prefix_rule_count_after="$(printf "%s\n" "${after_rows}" | rg -c "^.*\\|${PREFIX}\\|${EXPIRE_DAYS}$" || true)"
    prefix_rule_count_after="${prefix_rule_count_after:-0}"
  fi
fi

if [[ "${DRY_RUN}" == "true" ]]; then
  if [[ "${add_needed}" == "true" || "${prefix_rule_count_after}" -ge 1 ]]; then
    status="PASS"
  else
    status="FAIL"
  fi
elif [[ "${prefix_rule_count_after}" -lt 1 ]]; then
  status="FAIL"
fi

mkdir -p "$(dirname "${OUTPUT_FILE}")"
cat > "${OUTPUT_FILE}" <<EOF
# Chat Attachment Lifecycle Policy Report

Date: ${NOW_UTC}
Mode: \`$( [[ "${DRY_RUN}" == "true" ]] && echo "dry-run" || echo "apply" )\`
Status: \`${status}\`

## Target

- Endpoint: \`${S3_ENDPOINT}\`
- Bucket: \`${S3_BUCKET}\`
- Region: \`${S3_REGION}\`
- Prefix: \`${PREFIX}\`
- Expire days: \`${EXPIRE_DAYS}\`

## Rule Summary

| Metric | Value |
|---|---:|
| Rules before | ${before_rule_count} |
| Prefix rules before | ${#prefix_rule_ids[@]} |
| Rules removed | ${#ids_to_remove[@]} |
| Rule add needed | ${add_needed} |
| Rules after | ${after_rule_count} |
| Matching prefix+expiry rules after | ${prefix_rule_count_after} |

## Commands

- \`scripts/deploy/chat_attachment_lifecycle_policy.sh $( [[ "${DRY_RUN}" == "true" ]] && echo "--dry-run " )--expire-days ${EXPIRE_DAYS} --prefix ${PREFIX}\`
- \`scripts/smoke/chat_attachment_s3_live.sh\`

## References

- \`docs/deployment/chat-attachment-storage-lifecycle-runbook.md\`
- \`docs/research/frontend-hot-path-integration-debt.md\` (CHAT-API-004)
EOF

if [[ "${status}" != "PASS" ]]; then
  echo "lifecycle policy verification failed; see ${OUTPUT_FILE}" >&2
  exit 1
fi

echo "==> OK (lifecycle policy $( [[ "${DRY_RUN}" == "true" ]] && echo "dry-run" || echo "apply" ) completed)"
echo "report: ${OUTPUT_FILE}"
