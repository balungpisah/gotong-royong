#!/usr/bin/env bash
set -euo pipefail

# Live smoke for Pack C edge-lane cutover behavior.
# Validates that involvement feed falls back to legacy lane when enabled,
# and returns edge-only results when fallback is disabled.

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

require_cmd curl
require_cmd python3
require_cmd cargo
require_cmd docker

uuid_hex() {
  python3 -c 'import uuid; print(uuid.uuid4().hex)'
}

pick_free_port() {
  python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
}

wait_http_ok() {
  local url="$1"
  local tries="${2:-80}"
  local delay_s="${3:-0.25}"
  local i
  for ((i=1; i<=tries; i++)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$delay_s"
  done
  echo "timeout waiting for $url" >&2
  return 1
}

http_json() {
  local method="$1"
  local url="$2"
  local token="${3:-}"
  local body="${4:-}"

  local args=(-sS -X "$method" "$url" -H "content-type: application/json")
  if [[ -n "$token" ]]; then
    args+=(-H "authorization: Bearer $token")
  fi
  args+=(-H "x-request-id: $(uuid_hex)")
  args+=(-H "x-correlation-id: $(uuid_hex)")
  if [[ -n "$body" ]]; then
    args+=(-d "$body")
  fi
  curl "${args[@]}"
}

json_get() {
  local json="$1"
  local key="$2"
  python3 - "$json" "$key" <<'PY'
import json
import sys
data = json.loads(sys.argv[1])
value = data.get(sys.argv[2])
if value is None:
    raise SystemExit(2)
print(value)
PY
}

surreal_sql_root_json() {
  local sql="$1"
  docker compose -f "${COMPOSE_FILE}" exec -T surrealdb /surreal sql \
    --endpoint "${SURREAL_ENDPOINT}" \
    --user "${SURREAL_USER}" \
    --pass "${SURREAL_PASS}" \
    --ns "${SURREAL_NS}" \
    --db "${SURREAL_DB}" \
    --json \
    --hide-welcome <<SQL | sed '/^onnxruntime cpuid_info warning:/d'
${sql}
SQL
}

API_PID=""

stop_api() {
  if [[ -n "${API_PID}" ]]; then
    kill "${API_PID}" >/dev/null 2>&1 || true
    wait "${API_PID}" >/dev/null 2>&1 || true
    API_PID=""
  fi
}

cleanup() {
  set +e
  stop_api
}
trap cleanup EXIT INT TERM

start_api() {
  local port="$1"
  local fallback_enabled="$2"
  local log_file="$3"

  stop_api
  APP_ENV="smoke" \
  PORT="${port}" \
  DATA_BACKEND="surrealdb" \
  SURREAL_ENDPOINT="${SURREAL_ENDPOINT}" \
  SURREAL_NS="${SURREAL_NS}" \
  SURREAL_DB="${SURREAL_DB}" \
  SURREAL_USER="${SURREAL_USER}" \
  SURREAL_PASS="${SURREAL_PASS}" \
  REDIS_URL="${REDIS_URL}" \
  CHAT_REALTIME_TRANSPORT="local" \
  DISCOVERY_FEED_INVOLVEMENT_FALLBACK_ENABLED="${fallback_enabled}" \
  LOG_LEVEL="warn" \
  "${API_BIN}" >"${log_file}" 2>&1 &
  API_PID="$!"

  wait_http_ok "http://127.0.0.1:${port}/health" 120 0.25
}

SURREAL_ENDPOINT="${SURREAL_ENDPOINT:-ws://127.0.0.1:8000}"
SURREAL_NS="${SURREAL_NS:-gotong}"
SURREAL_DB="${SURREAL_DB:-chat}"
SURREAL_USER="${SURREAL_USER:-root}"
SURREAL_PASS="${SURREAL_PASS:-root}"
REDIS_URL="${REDIS_URL:-redis://127.0.0.1:6379}"
COMPOSE_FILE="${COMPOSE_FILE:-compose.dev.yaml}"

if [[ "${SKIP_MIGRATE_CHECK:-0}" != "1" ]]; then
  echo "=== Migrating database (live) ==="
  SURREAL_ENDPOINT="${SURREAL_ENDPOINT}" \
  SURREAL_NS="${SURREAL_NS}" \
  SURREAL_DB="${SURREAL_DB}" \
  SURREAL_USER="${SURREAL_USER}" \
  SURREAL_PASS="${SURREAL_PASS}" \
  scripts/db/migrate.sh >/dev/null

  echo "=== Checking schema (live) ==="
  SURREAL_ENDPOINT="${SURREAL_ENDPOINT}" \
  SURREAL_NS="${SURREAL_NS}" \
  SURREAL_DB="${SURREAL_DB}" \
  SURREAL_USER="${SURREAL_USER}" \
  SURREAL_PASS="${SURREAL_PASS}" \
  scripts/db/check.sh >/dev/null
fi

echo "=== Building API ==="
cargo build -q -p gotong-api
API_BIN="${API_BIN:-$ROOT_DIR/target/debug/gotong-api}"

run_id="$(uuid_hex)"
community_id="community_edge_cutover_${run_id}"
email="edge_cutover_${run_id}@example.com"
password="secret12345"

port_fallback_on="$(pick_free_port)"
port_fallback_off="$(pick_free_port)"
base_on="http://127.0.0.1:${port_fallback_on}"
base_off="http://127.0.0.1:${port_fallback_off}"

echo "=== Starting API (fallback enabled) on :${port_fallback_on} ==="
start_api "${port_fallback_on}" "true" "/tmp/gr_smoke_feed_edge_cutover_on.log"

echo "=== Signup test user and create two feed items ==="
signup="$(http_json POST "${base_on}/v1/auth/signup" "" "$(cat <<JSON
{"email":"${email}","pass":"${password}","username":"edge_${run_id}","community_id":"${community_id}"}
JSON
)")"
token="$(json_get "$signup" "access_token")"
user_id="$(json_get "$signup" "user_id")"

create_note() {
  local note_text="$1"
  http_json POST "${base_on}/v1/ontology/feed" "$token" "$(cat <<JSON
{
  "content": "${note_text}",
  "community_id": "${community_id}",
  "temporal_class": "persistent",
  "rahasia_level": 0
}
JSON
)"
}

created_a="$(create_note "Edge cutover note A ${run_id}")"
created_b="$(create_note "Edge cutover note B ${run_id}")"
feed_id_a="$(json_get "$created_a" "feed_id")"
feed_id_b="$(json_get "$created_b" "feed_id")"

if [[ -z "${feed_id_a}" || -z "${feed_id_b}" ]]; then
  echo "failed to parse feed IDs from create responses" >&2
  exit 1
fi

missing_feed_id="${feed_id_a}"
present_feed_id="${feed_id_b}"

echo "=== Delete one participant-edge row to create edge coverage gap ==="
surreal_sql_root_json "DELETE feed_participant_edge WHERE actor_id='${user_id}' AND feed_id='${missing_feed_id}';" >/dev/null

echo "=== Verify fallback-enabled API still returns both items ==="
feed_on="$(http_json GET "${base_on}/v1/feed?scope_id=${community_id}&involvement_only=true&limit=20" "$token")"
python3 - "$feed_on" "$missing_feed_id" "$present_feed_id" <<'PY'
import json
import sys

feed = json.loads(sys.argv[1])
missing_id = sys.argv[2]
present_id = sys.argv[3]
ids = [item.get("feed_id") for item in (feed.get("items") or [])]
if missing_id not in ids:
    raise SystemExit(f"fallback-enabled lane did not recover missing edge-backed row; response={json.dumps(feed)}")
if present_id not in ids:
    raise SystemExit(f"fallback-enabled lane missing expected edge-backed row; response={json.dumps(feed)}")
print("fallback-enabled check ok")
PY

echo "=== Restart API with fallback disabled (edge-only) on :${port_fallback_off} ==="
start_api "${port_fallback_off}" "false" "/tmp/gr_smoke_feed_edge_cutover_off.log"

echo "=== Sign in again against fallback-disabled API ==="
signin_off="$(http_json POST "${base_off}/v1/auth/signin" "" "$(cat <<JSON
{"email":"${email}","pass":"${password}"}
JSON
)")"
token_off="$(json_get "$signin_off" "access_token")"

echo "=== Verify edge-only mode excludes row without edge ==="
feed_off="$(http_json GET "${base_off}/v1/feed?scope_id=${community_id}&involvement_only=true&limit=20" "$token_off")"
python3 - "$feed_off" "$missing_feed_id" "$present_feed_id" <<'PY'
import json
import sys

feed = json.loads(sys.argv[1])
missing_id = sys.argv[2]
present_id = sys.argv[3]
ids = [item.get("feed_id") for item in (feed.get("items") or [])]
if missing_id in ids:
    raise SystemExit(f"edge-only lane unexpectedly returned row without participant edge; response={json.dumps(feed)}")
if present_id not in ids:
    raise SystemExit(f"edge-only lane missing expected edge-backed row; response={json.dumps(feed)}")
print("edge-only check ok")
PY

echo "=== OK (feed involvement edge cutover smoke passed) ==="
