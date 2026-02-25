#!/usr/bin/env bash
set -euo pipefail

# Live smoke for chat attachment S3/MinIO storage path.
# - boots API with CHAT_ATTACHMENT_STORAGE_BACKEND=s3
# - uploads a chat attachment
# - verifies signature-gated download returns 307 to presigned object URL
# - verifies object fetch from MinIO succeeds

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

ensure_bucket_exists() {
  local endpoint="$1"
  local bucket="$2"
  local access_key="$3"
  local secret_key="$4"

  if command -v mc >/dev/null 2>&1; then
    mc alias set local "$endpoint" "$access_key" "$secret_key" >/dev/null
    mc mb --ignore-existing "local/${bucket}" >/dev/null
    return 0
  fi

  local endpoint_for_container="$endpoint"
  endpoint_for_container="${endpoint_for_container/http:\/\/127.0.0.1/http:\/\/host.docker.internal}"
  endpoint_for_container="${endpoint_for_container/http:\/\/localhost/http:\/\/host.docker.internal}"

  docker run --rm --entrypoint /bin/sh minio/mc:RELEASE.2025-08-13T08-35-41Z \
    -c \
    "mc alias set local '${endpoint_for_container}' '${access_key}' '${secret_key}' >/dev/null && \
     mc mb --ignore-existing 'local/${bucket}' >/dev/null"
}

COMPOSE_FILE="${COMPOSE_FILE:-compose.dev.yaml}"
SURREAL_ENDPOINT="${SURREAL_ENDPOINT:-ws://127.0.0.1:8000}"
SURREAL_NS="${SURREAL_NS:-gotong}"
SURREAL_DB="${SURREAL_DB:-chat}"
SURREAL_USER="${SURREAL_USER:-root}"
SURREAL_PASS="${SURREAL_PASS:-root}"
REDIS_URL="${REDIS_URL:-redis://127.0.0.1:6379}"
S3_ENDPOINT="${S3_ENDPOINT:-http://127.0.0.1:9000}"
S3_BUCKET="${S3_BUCKET:-gotong-royong-evidence-dev}"
S3_ACCESS_KEY="${S3_ACCESS_KEY:-minioadmin}"
S3_SECRET_KEY="${S3_SECRET_KEY:-minioadmin}"
S3_REGION="${S3_REGION:-us-east-1}"
CHAT_ATTACHMENT_S3_PREFIX="${CHAT_ATTACHMENT_S3_PREFIX:-chat-attachments}"
REPORT_FILE="${1:-${ROOT_DIR}/docs/research/chat-attachment-s3-smoke-latest.md}"

API_PORT="${API_PORT:-$(pick_free_port)}"
API_PID=""
UPLOAD_FILE=""
DOWNLOAD_BODY=""
DOWNLOAD_HEADERS=""
S3_BODY=""
NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cleanup() {
  set +e
  if [[ -n "${API_PID}" ]]; then
    kill "${API_PID}" >/dev/null 2>&1 || true
    wait "${API_PID}" >/dev/null 2>&1 || true
  fi
  rm -f "${UPLOAD_FILE}" "${DOWNLOAD_BODY}" "${DOWNLOAD_HEADERS}" "${S3_BODY}" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

echo "=== Ensuring dev services (SurrealDB + Redis + MinIO) ==="
docker compose -f "${COMPOSE_FILE}" up -d surrealdb redis
docker compose -f "${COMPOSE_FILE}" --profile storage up -d minio

wait_http_ok "http://127.0.0.1:9000/minio/health/live" 120 0.5
ensure_bucket_exists "${S3_ENDPOINT}" "${S3_BUCKET}" "${S3_ACCESS_KEY}" "${S3_SECRET_KEY}"

if [[ "${SKIP_MIGRATE_CHECK:-0}" != "1" ]]; then
  echo "=== Migrating/checking schema (live) ==="
  SURREAL_ENDPOINT="${SURREAL_ENDPOINT}" \
  SURREAL_NS="${SURREAL_NS}" \
  SURREAL_DB="${SURREAL_DB}" \
  SURREAL_USER="${SURREAL_USER}" \
  SURREAL_PASS="${SURREAL_PASS}" \
  scripts/db/migrate.sh >/dev/null

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

echo "=== Starting API on :${API_PORT} (S3 attachment backend) ==="
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
CHAT_ATTACHMENT_STORAGE_BACKEND="s3" \
CHAT_ATTACHMENT_S3_PREFIX="${CHAT_ATTACHMENT_S3_PREFIX}" \
S3_ENDPOINT="${S3_ENDPOINT}" \
S3_BUCKET="${S3_BUCKET}" \
S3_ACCESS_KEY="${S3_ACCESS_KEY}" \
S3_SECRET_KEY="${S3_SECRET_KEY}" \
S3_REGION="${S3_REGION}" \
LOG_LEVEL="warn" \
"$API_BIN" >/tmp/gr_smoke_chat_attachment_s3_live.log 2>&1 &
API_PID="$!"

wait_http_ok "http://127.0.0.1:${API_PORT}/health" 120 0.25

base_url="http://127.0.0.1:${API_PORT}"
run_id="$(uuid_hex)"
email="chat_attachment_${run_id}@example.com"

echo "=== Signup smoke user ==="
signup="$(http_json POST "${base_url}/v1/auth/signup" "" "$(cat <<JSON
{"email":"${email}","pass":"secret12345","username":"chat_attachment_${run_id}","community_id":"community_${run_id}"}
JSON
)")"
token="$(json_get "$signup" "access_token")"

UPLOAD_FILE="$(mktemp)"
printf 'PNGDATA-S3-SMOKE-%s' "$run_id" > "${UPLOAD_FILE}"
DOWNLOAD_BODY="$(mktemp)"
DOWNLOAD_HEADERS="$(mktemp)"
S3_BODY="$(mktemp)"

echo "=== Upload chat attachment ==="
upload_response="$(
  curl -sS -X POST "${base_url}/v1/chat/attachments/upload" \
    -H "authorization: Bearer ${token}" \
    -H "x-request-id: $(uuid_hex)" \
    -H "x-correlation-id: $(uuid_hex)" \
    -F "file=@${UPLOAD_FILE};filename=foto.png;type=image/png"
)"
download_path="$(json_get "${upload_response}" "url")"
attachment_id="$(json_get "${upload_response}" "attachment_id")"
if [[ -z "${download_path}" || -z "${attachment_id}" ]]; then
  echo "failed to parse upload response: ${upload_response}" >&2
  exit 1
fi

echo "=== Validate API download gate returns redirect ==="
download_status="$(
  curl -sS -o "${DOWNLOAD_BODY}" -D "${DOWNLOAD_HEADERS}" -w "%{http_code}" \
    "${base_url}${download_path}"
)"
if [[ "${download_status}" != "307" ]]; then
  echo "expected 307 from signed download endpoint, got ${download_status}" >&2
  echo "upload_response=${upload_response}" >&2
  exit 1
fi
s3_url="$(python3 - "${DOWNLOAD_HEADERS}" <<'PY'
import sys
from pathlib import Path
headers = Path(sys.argv[1]).read_text()
for line in headers.splitlines():
    if line.lower().startswith("location:"):
        print(line.split(":", 1)[1].strip())
        break
else:
    raise SystemExit("missing Location header")
PY
)"

echo "=== Validate presigned object URL fetch ==="
s3_status="$(curl -sS -o "${S3_BODY}" -w "%{http_code}" "${s3_url}")"
if [[ "${s3_status}" != "200" ]]; then
  echo "expected 200 from presigned object URL, got ${s3_status}" >&2
  exit 1
fi
if ! cmp -s "${UPLOAD_FILE}" "${S3_BODY}"; then
  echo "downloaded object body mismatch" >&2
  exit 1
fi

mkdir -p "$(dirname "${REPORT_FILE}")"
cat > "${REPORT_FILE}" <<EOF
# Chat Attachment S3 Smoke Report

Date: ${NOW_UTC}
Mode: \`live\`
API URL: \`${base_url}\`
S3 Endpoint: \`${S3_ENDPOINT}\`
S3 Bucket: \`${S3_BUCKET}\`
Attachment Backend: \`s3\`

## Checks

| Check | Status |
|---|---|
| Upload \`POST /v1/chat/attachments/upload\` | PASS |
| Download gate \`GET /v1/chat/attachments/:id/download\` returns 307 | PASS |
| Presigned object URL fetch from MinIO returns 200 | PASS |
| Uploaded bytes match downloaded bytes | PASS |

## Sample Artifact

- Attachment ID: \`${attachment_id}\`
- Signed download path: \`${download_path}\`
- Presigned object URL host: \`$(python3 - "${s3_url}" <<'PY'
import sys
from urllib.parse import urlparse
print(urlparse(sys.argv[1]).netloc)
PY
)\`
EOF

echo "=== OK (chat attachment S3 smoke passed) ==="
echo "report: ${REPORT_FILE}"
