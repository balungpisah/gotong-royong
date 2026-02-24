#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

: "${SURREAL_PORT:=8000}"
: "${REDIS_PORT:=6379}"
: "${SURREAL_NS:=gotong}"
: "${SURREAL_DB:=chat}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"

compose_file="${COMPOSE_FILE:-compose.dev.yaml}"

echo "=== Starting dev services (SurrealDB + Redis) ==="
docker compose -f "$compose_file" up -d

echo "=== Waiting for SurrealDB readiness ==="
for _ in $(seq 1 60); do
  if docker compose -f "$compose_file" exec -T surrealdb /surreal is-ready --endpoint ws://127.0.0.1:8000 >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done
if ! docker compose -f "$compose_file" exec -T surrealdb /surreal is-ready --endpoint ws://127.0.0.1:8000 >/dev/null 2>&1; then
  echo "SurrealDB did not become ready (ws://127.0.0.1:8000)" >&2
  exit 1
fi

echo "=== Running migrations + checks ==="
SURREAL_ENDPOINT="ws://127.0.0.1:${SURREAL_PORT}" \
SURREAL_NS="${SURREAL_NS}" \
SURREAL_DB="${SURREAL_DB}" \
SURREAL_USER="${SURREAL_USER}" \
SURREAL_PASS="${SURREAL_PASS}" \
scripts/db/migrate.sh >/dev/null

SURREAL_ENDPOINT="ws://127.0.0.1:${SURREAL_PORT}" \
SURREAL_NS="${SURREAL_NS}" \
SURREAL_DB="${SURREAL_DB}" \
SURREAL_USER="${SURREAL_USER}" \
SURREAL_PASS="${SURREAL_PASS}" \
scripts/db/check.sh >/dev/null

echo "=== Dev DB ready ==="
echo "SurrealDB: ws://127.0.0.1:${SURREAL_PORT}"
echo "Redis: redis://127.0.0.1:${REDIS_PORT}"
