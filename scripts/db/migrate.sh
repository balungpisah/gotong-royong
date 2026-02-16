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

run_migration() {
  local migration_file="$1"
  local migration_path="$WORKDIR/database/migrations/$migration_file"
  local output

  output=$(
    cat "$migration_path" | "${SUR_CMD[@]}" sql \
      --multi \
      --endpoint "$SURREAL_ENDPOINT" \
      --user "$SURREAL_USER" \
      --pass "$SURREAL_PASS" \
      --namespace "$SURREAL_NS" \
      --database "$SURREAL_DB"
  )

  echo "$output"

  if [[ "$output" == *"Thrown error"* ]]; then
    echo "migration failed for $migration_file due to Surreal thrown error" >&2
    return 1
  fi
}

if command -v "${SUR_CMD[0]}" >/dev/null 2>&1; then
  surreal_version="$(${SUR_CMD[0]} version 2>/dev/null | awk 'NR==1 {print $1}')"
  surreal_major="${surreal_version%%.*}"
  if [[ "$surreal_major" != "3" ]]; then
    SUR_CMD=(docker run --rm -i --network host -v "${WORKDIR}:${CONTAINER_WORKDIR}" "$SURREAL_IMAGE")
  fi
else
  # Note: Docker Desktop on macOS/Windows doesn't support --network host.
  # Fall back to container-based CLI for reliable protocol compatibility.
  SUR_CMD=(docker run --rm -i --network host -v "${WORKDIR}:${CONTAINER_WORKDIR}" "$SURREAL_IMAGE")
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
  "0012_adaptive_path_schema.surql"; do
  run_migration "$migration_file"
done
