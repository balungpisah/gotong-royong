#!/usr/bin/env bash
set -euo pipefail

: "${SURREAL_ENDPOINT:=ws://127.0.0.1:8000}"
: "${SURREAL_NS:=gotong}"
: "${SURREAL_DB:=chat}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"
: "${SURREAL_IMAGE:=surrealdb/surrealdb:v3.0.0}"

SUR_CMD=(surreal)
WORKDIR="$(pwd)"
CONTAINER_WORKDIR="/workspace"
CLI_ENDPOINT="$SURREAL_ENDPOINT"

configure_docker_cli() {
  local os_name
  local docker_network_flag="--network host"
  os_name="$(uname -s 2>/dev/null || echo unknown)"

  case "$os_name" in
    Darwin|MINGW*|MSYS*|CYGWIN*)
      # Docker Desktop doesn't provide host networking semantics equivalent to Linux.
      docker_network_flag=""
      CLI_ENDPOINT="${CLI_ENDPOINT/127.0.0.1/host.docker.internal}"
      CLI_ENDPOINT="${CLI_ENDPOINT/localhost/host.docker.internal}"
      ;;
  esac

  if [[ -n "$docker_network_flag" ]]; then
    SUR_CMD=(docker run --rm -i "$docker_network_flag" -v "${WORKDIR}:${CONTAINER_WORKDIR}" "$SURREAL_IMAGE")
  else
    SUR_CMD=(docker run --rm -i -v "${WORKDIR}:${CONTAINER_WORKDIR}" "$SURREAL_IMAGE")
  fi
}

CHECKS=(
  "0001_initial_schema_check.surql"
  "0002_chat_indexes_check.surql"
  "0003_permissions_private_channels_check.surql"
  "0005_moderation_check.surql"
  "0006_vault_check.surql"
  "0007_siaga_check.surql"
  "0008_discovery_check.surql"
  "0009_audit_retention_fields_check.surql"
  "0010_contributions_evidence_vouch_check.surql"
  "0011_webhook_outbox_check.surql"
  "0012_adaptive_path_check.surql"
  "0013_ontology_schema_check.surql"
  "0016_add_mode_fields_check.surql"
  "0017_path_plan_action_type_check.surql"
  "0018_auth_schema_check.surql"
  "0019_record_permissions_check.surql"
  "0021_chat_optional_datetimes_check.surql"
  "0022_auth_token_permissions_fix_check.surql"
  "0023_enrichment_indexes_check.surql"
  "0024_discovery_payload_flexible_check.surql"
  "0025_hot_path_pack_a_indexes_check.surql"
  "0026_hot_path_pack_b_indexes_check.surql"
  "0027_hot_path_pack_c_feed_participant_edge_check.surql"
)

run_check() {
  local check_file="$1"

  local output
  if ! output=$(
    cat "$check_file" | "${SUR_CMD[@]}" sql \
      --multi \
      --endpoint "$CLI_ENDPOINT" \
      --user "$SURREAL_USER" \
      --pass "$SURREAL_PASS" \
      --namespace "$SURREAL_NS" \
      --database "$SURREAL_DB" \
      --json \
      2>&1
  ); then
    echo "$output"
    echo "$check_file failed: unable to execute Surreal SQL against $CLI_ENDPOINT" >&2
    return 1
  fi

  echo "$output"

  if [[ "$output" == *"Thrown error"* ]]; then
    echo "$check_file failed: Surreal returned thrown error" >&2
    return 1
  fi

  if [[ "$output" == *"Parse error"* ]]; then
    echo "$check_file failed: Surreal returned parse error" >&2
    return 1
  fi

  if [[ "$check_file" == *0009_audit_retention_fields_check.surql ]]; then
    if ! CHECK_OUTPUT="$output" python3 - <<'PY'
import json
import os
import sys

raw = os.environ.get("CHECK_OUTPUT", "")

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
  configure_docker_cli
else
  surreal_version="$("${SUR_CMD[0]}" version 2>/dev/null || "${SUR_CMD[0]}" --version 2>/dev/null || true)"
  surreal_major="$(echo "$surreal_version" | grep -Eo '[0-9]+' | head -n 1 || true)"
  if [[ "$surreal_major" != "3" ]]; then
    configure_docker_cli
  fi
fi

for check in "${CHECKS[@]}"; do
  run_check "$WORKDIR/database/checks/$check"
done
