#!/usr/bin/env bash
set -euo pipefail

: "${SURREAL_ENDPOINT:=ws://127.0.0.1:8000}"
: "${SURREAL_NS:=gotong}"
: "${SURREAL_DB:=chat}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"
: "${SURREAL_IMAGE:=surrealdb/surrealdb:v3.0.0-beta-4}"

SUR_CMD=(surreal)
WORKDIR="$(pwd)"

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
)

run_check() {
  local check_file="$1"

  local output
  output=$(
    cat "$check_file" | "${SUR_CMD[@]}" sql \
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
import re
import sys

raw = sys.stdin.read()
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
    match = re.search(rf'"{re.escape(key)}"\s*:\s*([0-9]+)', raw)
    if match is None:
        print(f"audit retention check output is missing metric '{key}'", file=sys.stderr)
        sys.exit(1)

    if int(match.group(1)) > 0:
        violations.append((key, int(match.group(1))))

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
  # Prefer the local SurrealDB CLI binary for dev on those platforms.
  SUR_CMD=(docker run --rm --network host -v "${WORKDIR}:/workspace" "$SURREAL_IMAGE")
  WORKDIR="/workspace"
fi

for check in "${CHECKS[@]}"; do
  run_check "$WORKDIR/database/checks/$check"
done
