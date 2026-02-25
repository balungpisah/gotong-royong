#!/usr/bin/env bash
set -euo pipefail

# Live smoke for ontology->feed enrichment invariants against SurrealDB.
# Assumes `just dev-db-up` has started SurrealDB + Redis.

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
"$API_BIN" >/tmp/gr_smoke_ontology_enrichment_live.log 2>&1 &
API_PID="$!"

wait_http_ok "http://127.0.0.1:${API_PORT}/health" 120 0.25

base_url="http://127.0.0.1:${API_PORT}"
run_id="$(uuid_hex)"
community_id="community_${run_id}"
email="ontology_${run_id}@example.com"

echo "=== Signup test user ==="
signup="$(http_json POST "${base_url}/v1/auth/signup" "" "$(cat <<JSON
{"email":"${email}","pass":"secret12345","username":"ontology_${run_id}","community_id":"${community_id}"}
JSON
)")"
token="$(json_get "$signup" "access_token")"

echo "=== Create ontology note (public) ==="
create_payload="$(cat <<JSON
{
  "content": "Harga beras naik di pasar warga ${run_id}",
  "community_id": "${community_id}",
  "temporal_class": "persistent",
  "rahasia_level": 0
}
JSON
)"
created="$(http_json POST "${base_url}/v1/ontology/feed" "$token" "${create_payload}")"
note_id="$(python3 - "$created" <<'PY'
import json
import sys
data = json.loads(sys.argv[1])
print((data.get("note") or {}).get("note_id", ""))
PY
)"
feed_id="$(python3 - "$created" <<'PY'
import json
import sys
data = json.loads(sys.argv[1])
print(data.get("feed_id", ""))
PY
)"
if [[ -z "${note_id}" || -z "${feed_id}" ]]; then
  echo "failed to parse note_id/feed_id from create response: ${created}" >&2
  exit 1
fi

echo "=== Validate initial enrichment fields ==="
feed_response_1="$(http_json GET "${base_url}/v1/feed?scope_id=${community_id}&limit=50" "$token")"
python3 - "$feed_response_1" "$note_id" "$feed_id" <<'PY'
import json
import sys
feed = json.loads(sys.argv[1])
note_id = sys.argv[2]
feed_id = sys.argv[3]
items = feed.get("items") or []
target = None
for item in items:
    if item.get("feed_id") == feed_id and item.get("source_type") == "ontology_note" and item.get("source_id") == note_id:
        target = item
        break
if target is None:
    raise SystemExit("feed item not found after ontology note creation")
enrichment = (((target.get("payload") or {}).get("enrichment")) or {})
if "tags_enriched_at_ms" not in enrichment:
    raise SystemExit("tags_enriched_at_ms missing after create")
if "feedback_enriched_at_ms" not in enrichment:
    raise SystemExit("feedback_enriched_at_ms missing after create")
print("ok")
PY

echo "=== Apply vouch and validate feedback timestamp + tags timestamp retention ==="
_vouch_resp="$(http_json POST "${base_url}/v1/ontology/notes/${note_id}/vouches" "$token" '{"metadata":{"reason":"valid"}}')"
feed_response_2="$(http_json GET "${base_url}/v1/feed?scope_id=${community_id}&limit=50" "$token")"
python3 - "$feed_response_2" "$note_id" "$feed_id" <<'PY'
import json
import sys
feed = json.loads(sys.argv[1])
note_id = sys.argv[2]
feed_id = sys.argv[3]
items = feed.get("items") or []
target = None
for item in items:
    if item.get("feed_id") == feed_id and item.get("source_type") == "ontology_note" and item.get("source_id") == note_id:
        target = item
        break
if target is None:
    raise SystemExit("feed item not found after vouch")
enrichment = (((target.get("payload") or {}).get("enrichment")) or {})
if "tags_enriched_at_ms" not in enrichment:
    raise SystemExit("tags_enriched_at_ms missing after vouch patch")
if "feedback_enriched_at_ms" not in enrichment:
    raise SystemExit("feedback_enriched_at_ms missing after vouch patch")
feedback = enrichment.get("feedback") or {}
if feedback.get("vouch_count") != 1:
    raise SystemExit(f"expected vouch_count=1, got {feedback.get('vouch_count')}")
print("ok")
PY

echo "=== OK (ontology enrichment live smoke passed) ==="
