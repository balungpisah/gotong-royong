#!/usr/bin/env bash
set -euo pipefail

: "${SURREAL_ENDPOINT:=ws://127.0.0.1:8000}"
: "${SURREAL_NS:=gotong}"
: "${SURREAL_DB:=chat}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"
: "${SURREAL_IMAGE:=surrealdb/surrealdb:v3.0.0-beta.4}"

SUR_CMD=(surreal)
WORKDIR="$(pwd)"
CONTAINER_WORKDIR="/workspace"

CHECKS=(
  "0001_initial_schema_check.surql"
  "0002_chat_indexes_check.surql"
  "0003_permissions_private_channels_check.surql"
  "0004_transition_prereq_check.surql"
  "0005_moderation_check.surql"
  "0006_vault_check.surql"
  "0007_siaga_check.surql"
  "0008_discovery_check.surql"
  "0009_audit_retention_fields_check.surql"
  "0010_contributions_evidence_vouch_check.surql"
  "0011_webhook_outbox_check.surql"
)

run_check() {
  local check_file="$1"

  local output
  output=$(
    cat "$check_file" | "${SUR_CMD[@]}" sql \
      --multi \
      --endpoint "$SURREAL_ENDPOINT" \
      --user "$SURREAL_USER" \
      --pass "$SURREAL_PASS" \
      --namespace "$SURREAL_NS" \
      --database "$SURREAL_DB" \
      --json
  )

  echo "$output"

  if [[ "$check_file" == *0009_audit_retention_fields_check.surql ]]; then
    if ! printf '%s\n' "$output" | python3 - <<'PY'
import json
import sys

raw = sys.stdin.read()

if "Thrown error" in raw:
    print("audit retention check failed: Surreal returned an error", file=sys.stderr)
    sys.exit(1)

parsed_lines = []
for line in raw.splitlines():
    value = line.strip()
    if not value:
        continue
    if not value.startswith("{") and not value.startswith("["):
        continue
    try:
        parsed_lines.append(json.loads(value))
    except json.JSONDecodeError:
        continue


def _extract_metric(payload, key):
    if isinstance(payload, dict):
        if key in payload and isinstance(payload[key], (int, float, str)):
            try:
                return int(payload[key])
            except (TypeError, ValueError):
                return None
        for nested in payload.values():
            found = _extract_metric(nested, key)
            if found is not None:
                return found
    elif isinstance(payload, list):
        for item in payload:
            found = _extract_metric(item, key)
            if found is not None:
                return found
    return None


def get_metric(key):
    for payload in parsed_lines:
        value = _extract_metric(payload, key)
        if value is not None:
            return value

    # Surreal v3 returns [] for count queries on empty tables.
    return 0


required_metrics = (
    "transition_rows_missing_event_hash",
    "transition_rows_missing_retention_tag",
    "vault_entry_rows_missing_event_hash",
    "vault_entry_rows_missing_retention_tag",
    "vault_timeline_rows_missing_event_hash",
    "vault_timeline_rows_missing_retention_tag",
    "siaga_broadcast_rows_missing_event_hash",
    "siaga_broadcast_rows_missing_retention_tag",
    "siaga_timeline_rows_missing_event_hash",
    "siaga_timeline_rows_missing_retention_tag",
    "content_rows_missing_event_hash",
    "content_rows_missing_retention_tag",
    "moderation_rows_missing_event_hash",
    "moderation_rows_missing_retention_tag",
)

violations = []
for key in required_metrics:
    value = get_metric(key)
    if value > 0:
        violations.append((key, value))

if violations:
    print("audit retention checks failed:", file=sys.stderr)
    for key, value in violations:
        print(f"  {key}: {value}", file=sys.stderr)
    sys.exit(1)
PY
    then
      echo "0009_audit_retention_fields_check failed due to missing or non-zero metrics" >&2
      return 1
    fi
  fi
}

if ! command -v "${SUR_CMD[0]}" >/dev/null 2>&1; then
  # Note: Docker Desktop on macOS/Windows doesn't support --network host.
  # Fall back to container-based CLI for reliable protocol compatibility.
  SUR_CMD=(docker run --rm -i --network host -v "${WORKDIR}:${CONTAINER_WORKDIR}" "$SURREAL_IMAGE")
else
  surreal_version="$("${SUR_CMD[0]}" version 2>/dev/null | awk 'NR==1 {print $1}')"
  surreal_major="${surreal_version%%.*}"
  if [[ "$surreal_major" != "3" ]]; then
    SUR_CMD=(docker run --rm -i --network host -v "${WORKDIR}:${CONTAINER_WORKDIR}" "$SURREAL_IMAGE")
  fi
fi

for check in "${CHECKS[@]}"; do
  run_check "$WORKDIR/database/checks/$check"
done
