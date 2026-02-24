#!/usr/bin/env bash
set -euo pipefail

# End-to-end smoke test against an existing (live) SurrealDB + Redis.
#
# Assumes you already started dev services via:
#   just dev-db-up
#
# This script:
# - (optionally) runs migrations/checks
# - boots the API locally
# - validates signup/authz/logout-revocation flows

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

SURREAL_ENDPOINT="${SURREAL_ENDPOINT:-ws://127.0.0.1:8000}"
SURREAL_NS="${SURREAL_NS:-gotong}"
SURREAL_DB="${SURREAL_DB:-chat}"
SURREAL_USER="${SURREAL_USER:-root}"
SURREAL_PASS="${SURREAL_PASS:-root}"
REDIS_URL="${REDIS_URL:-redis://127.0.0.1:6379}"

API_PORT="${API_PORT:-$(pick_free_port)}"
API_PID=""

cleanup() {
  set +e
  if [[ -n "${API_PID}" ]]; then
    kill "${API_PID}" >/dev/null 2>&1 || true
    wait "${API_PID}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

wait_http_ok() {
  local url="$1"
  local tries="${2:-60}"
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

http_status() {
  local method="$1"
  local url="$2"
  local token="${3:-}"
  local body="${4:-}"

  local args=(-sS -o /dev/null -w "%{http_code}" -X "$method" "$url")
  if [[ -n "$token" ]]; then
    args+=(-H "authorization: Bearer $token")
  fi
  args+=(-H "content-type: application/json")
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
if [[ ! -x "$API_BIN" ]]; then
  echo "API binary not found/executable: $API_BIN" >&2
  exit 1
fi

echo "=== Starting API on :${API_PORT} ==="
APP_ENV="smoke" \
PORT="${API_PORT}" \
DATA_BACKEND="surrealdb" \
SURREAL_ENDPOINT="${SURREAL_ENDPOINT}" \
SURREAL_NS="${SURREAL_NS}" \
SURREAL_DB="${SURREAL_DB}" \
SURREAL_USER="${SURREAL_USER}" \
SURREAL_PASS="${SURREAL_PASS}" \
REDIS_URL="${REDIS_URL}" \
CHAT_REALTIME_TRANSPORT="local" \
LOG_LEVEL="warn" \
"$API_BIN" >/tmp/gr_smoke_api_live.log 2>&1 &
API_PID="$!"

wait_http_ok "http://127.0.0.1:${API_PORT}/health" 120 0.25

base_url="http://127.0.0.1:${API_PORT}"
run_id="$(uuid_hex)"

email_a="a_${run_id}@example.com"
email_b="b_${run_id}@example.com"

echo "=== Signup A ==="
signup_a="$(http_json POST "${base_url}/v1/auth/signup" "" "$(cat <<JSON
{"email":"${email_a}","pass":"secret12345","username":"a_${run_id}","community_id":"community_${run_id}"}
JSON
)")"
token_a="$(json_get "$signup_a" "access_token")"

echo "=== Signup B ==="
signup_b="$(http_json POST "${base_url}/v1/auth/signup" "" "$(cat <<JSON
{"email":"${email_b}","pass":"secret12345","username":"b_${run_id}","community_id":"community_${run_id}"}
JSON
)")"
token_b="$(json_get "$signup_b" "access_token")"

echo "=== Create private chat thread as A ==="
thread_resp="$(http_json POST "${base_url}/v1/chat/threads" "$token_a" '{"scope_id":"scope_smoke","privacy_level":"private"}')"
thread_id="$(json_get "$thread_resp" "thread_id")"
if [[ -z "$thread_id" ]]; then
  echo "failed to create chat thread; response: $thread_resp" >&2
  exit 1
fi

echo "=== List threads as B (should NOT see A's private thread) ==="
status_threads_b="$(http_status GET "${base_url}/v1/chat/threads" "$token_b")"
if [[ "$status_threads_b" != "200" ]]; then
  echo "expected 200 from /v1/chat/threads as B, got ${status_threads_b}" >&2
  echo "body: $(http_json GET "${base_url}/v1/chat/threads" "$token_b")" >&2
  exit 1
fi
threads_b="$(http_json GET "${base_url}/v1/chat/threads" "$token_b")"
python3 - "$threads_b" "$thread_id" <<'PY'
import json, sys
threads=json.loads(sys.argv[1])
target=sys.argv[2]
if not isinstance(threads, list):
  raise SystemExit(f"expected list, got: {type(threads)}")
ids=[t.get("thread_id") for t in threads if isinstance(t, dict)]
if target in ids:
  raise SystemExit(f"authz failed: B can see A's private thread {target}")
print("ok")
PY

echo "=== Logout A (revoke token) ==="
status_logout="$(http_status POST "${base_url}/v1/auth/logout" "$token_a" '{}')"
if [[ "$status_logout" != "204" ]]; then
  echo "expected 204 from logout, got ${status_logout}" >&2
  echo "body: $(http_json POST "${base_url}/v1/auth/logout" "$token_a" '{}')" >&2
  exit 1
fi

echo "=== Ensure revoked token is rejected ==="
status_me="$(http_status GET "${base_url}/v1/auth/me" "$token_a")"
if [[ "$status_me" == "200" ]]; then
  echo "expected revoked token to be rejected; /v1/auth/me returned 200" >&2
  echo "body: $(http_json GET "${base_url}/v1/auth/me" "$token_a")" >&2
  exit 1
fi

echo "=== OK (live auth E2E passed) ==="

