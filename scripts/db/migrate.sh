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

if ! command -v "${SUR_CMD[0]}" >/dev/null 2>&1; then
  # Note: Docker Desktop on macOS/Windows doesn't support --network host.
  # Prefer the local SurrealDB CLI binary for dev on those platforms.
  SUR_CMD=(docker run --rm --network host -v "${WORKDIR}:/workspace" "$SURREAL_IMAGE")
  WORKDIR="/workspace"
fi

"${SUR_CMD[@]}" import \
  --endpoint "$SURREAL_ENDPOINT" \
  --user "$SURREAL_USER" \
  --pass "$SURREAL_PASS" \
  --namespace "$SURREAL_NS" \
  --database "$SURREAL_DB" \
  "$WORKDIR/database/migrations/0001_initial_schema.surql"

"${SUR_CMD[@]}" import \
  --endpoint "$SURREAL_ENDPOINT" \
  --user "$SURREAL_USER" \
  --pass "$SURREAL_PASS" \
  --namespace "$SURREAL_NS" \
  --database "$SURREAL_DB" \
  "$WORKDIR/database/migrations/0002_chat_indexes.surql"

"${SUR_CMD[@]}" import \
  --endpoint "$SURREAL_ENDPOINT" \
  --user "$SURREAL_USER" \
  --pass "$SURREAL_PASS" \
  --namespace "$SURREAL_NS" \
  --database "$SURREAL_DB" \
  "$WORKDIR/database/migrations/0003_permissions_private_channels.surql"

"${SUR_CMD[@]}" import \
  --endpoint "$SURREAL_ENDPOINT" \
  --user "$SURREAL_USER" \
  --pass "$SURREAL_PASS" \
  --namespace "$SURREAL_NS" \
  --database "$SURREAL_DB" \
  "$WORKDIR/database/migrations/0004_moderation_schema.surql"

"${SUR_CMD[@]}" import \
  --endpoint "$SURREAL_ENDPOINT" \
  --user "$SURREAL_USER" \
  --pass "$SURREAL_PASS" \
  --namespace "$SURREAL_NS" \
  --database "$SURREAL_DB" \
  "$WORKDIR/database/migrations/0006_vault_schema.surql"

"${SUR_CMD[@]}" import \
  --endpoint "$SURREAL_ENDPOINT" \
  --user "$SURREAL_USER" \
  --pass "$SURREAL_PASS" \
  --namespace "$SURREAL_NS" \
  --database "$SURREAL_DB" \
  "$WORKDIR/database/migrations/0007_siaga_schema.surql"

"${SUR_CMD[@]}" import \
  --endpoint "$SURREAL_ENDPOINT" \
  --user "$SURREAL_USER" \
  --pass "$SURREAL_PASS" \
  --namespace "$SURREAL_NS" \
  --database "$SURREAL_DB" \
  "$WORKDIR/database/migrations/0008_discovery_schema.surql"
