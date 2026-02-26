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

run_migration() {
  local migration_file="$1"
  local migration_path="$WORKDIR/database/migrations/$migration_file"
  local output

  if ! output=$(
    cat "$migration_path" | "${SUR_CMD[@]}" sql \
      --multi \
      --endpoint "$CLI_ENDPOINT" \
      --user "$SURREAL_USER" \
      --pass "$SURREAL_PASS" \
      --namespace "$SURREAL_NS" \
      --database "$SURREAL_DB" \
      2>&1
  ); then
    echo "$output"
    echo "migration failed for $migration_file: unable to execute Surreal SQL against $CLI_ENDPOINT" >&2
    return 1
  fi

  echo "$output"

  if [[ "$output" == *"Thrown error"* ]]; then
    echo "migration failed for $migration_file due to Surreal thrown error" >&2
    return 1
  fi

  if [[ "$output" == *"Parse error"* ]]; then
    echo "migration failed for $migration_file due to Surreal parse error" >&2
    return 1
  fi
}

if command -v "${SUR_CMD[0]}" >/dev/null 2>&1; then
  surreal_version="$(${SUR_CMD[0]} version 2>/dev/null || ${SUR_CMD[0]} --version 2>/dev/null || true)"
  surreal_major="$(echo "$surreal_version" | grep -Eo '[0-9]+' | head -n 1 || true)"
  if [[ "$surreal_major" != "3" ]]; then
    configure_docker_cli
  fi
else
  configure_docker_cli
fi

for migration_file in \
  "0001_initial_schema.surql" \
  "0002_chat_indexes.surql" \
  "0003_permissions_private_channels.surql" \
  "0004_moderation_schema.surql" \
  "0006_vault_schema.surql" \
  "0007_siaga_schema.surql" \
  "0008_discovery_schema.surql" \
  "0009_audit_retention_fields.surql" \
  "0010_contributions_evidence_vouch_schema.surql" \
  "0011_webhook_outbox_schema.surql" \
  "0012_adaptive_path_schema.surql" \
  "0013_ontology_schema.surql" \
  "0014_remove_track_state_transition.surql" \
  "0015_discovery_drop_track_stage.surql" \
  "0016_add_mode_fields.surql" \
  "0017_path_plan_action_type.surql" \
  "0018_auth_schema.surql" \
  "0019_record_permissions.surql" \
  "0020_auth_access_overwrite.surql" \
  "0021_chat_optional_datetimes.surql" \
  "0022_auth_token_permissions_fix.surql" \
  "0023_enrichment_indexes.surql" \
  "0024_discovery_payload_flexible.surql" \
  "0025_hot_path_pack_a_indexes.surql" \
  "0026_hot_path_pack_b_indexes.surql" \
  "0027_hot_path_pack_c_feed_participant_edge.surql" \
  "0028_chat_message_request_index_order.surql" \
  "0029_group_read_model_schema.surql" \
  "0030_webhook_payload_flexible.surql" \
  "0031_feed_preference_schema.surql"; do
  run_migration "$migration_file"
done
