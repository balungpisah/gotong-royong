#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

: "${API_BASE_URL:=http://127.0.0.1:3100}"
: "${SEED_COUNT:=4}"
: "${SEED_PREFIX:=devseed}"
: "${SEED_EMAIL:=${SEED_PREFIX}@example.com}"
: "${SEED_PASSWORD:=secret123}"
: "${SEED_USERNAME:=${SEED_PREFIX}_user}"
: "${SEED_COMMUNITY_ID:=rt05}"
: "${SEED_BEARER_TOKEN:=}"
: "${COMPOSE_FILE:=compose.dev.yaml}"
: "${SURREAL_ENDPOINT:=ws://127.0.0.1:8000}"
: "${SURREAL_NS:=gotong}"
: "${SURREAL_DB:=chat}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
}

require_cmd curl
require_cmd jq
require_cmd docker

if ! [[ "$SEED_COUNT" =~ ^[0-9]+$ ]] || [[ "$SEED_COUNT" -lt 1 ]]; then
  echo "SEED_COUNT must be a positive integer" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

curl_json_public() {
  local method="$1"
  local path="$2"
  local body="$3"
  local outfile="$4"
  curl -sS -X "$method" "${API_BASE_URL}${path}" \
    -H 'accept: application/json' \
    -H 'content-type: application/json' \
    -d "$body" \
    -o "$outfile" \
    -w '%{http_code}'
}

curl_get_authed() {
  local path="$1"
  local token="$2"
  local outfile="$3"
  curl -sS "${API_BASE_URL}${path}" \
    -H 'accept: application/json' \
    -H "authorization: Bearer ${token}" \
    -o "$outfile" \
    -w '%{http_code}'
}

surreal_exec() {
  local sql="$1"
  docker compose -f "$COMPOSE_FILE" exec -T surrealdb /surreal sql \
    --endpoint "$SURREAL_ENDPOINT" \
    --user "$SURREAL_USER" \
    --pass "$SURREAL_PASS" \
    --ns "$SURREAL_NS" \
    --db "$SURREAL_DB" \
    --hide-welcome \
    --multi >/dev/null <<<"$sql"
}

seed_token="$SEED_BEARER_TOKEN"
seed_user_id=""

ensure_seed_session() {
  if [[ -n "$seed_token" ]]; then
    local me_out="$tmp_dir/auth-me.json"
    local me_status
    me_status="$(curl_get_authed '/v1/auth/me' "$seed_token" "$me_out")"
    if [[ "$me_status" == "200" ]]; then
      seed_user_id="$(jq -r '.user_id // empty' "$me_out")"
      if [[ -n "$seed_user_id" ]]; then
        return 0
      fi
    fi
    echo "Provided SEED_BEARER_TOKEN is invalid for /v1/auth/me" >&2
    echo "Response: $(cat "$me_out")" >&2
    exit 1
  fi

  local signup_out="$tmp_dir/auth-signup.json"
  local signup_body
  signup_body="$(jq -nc \
    --arg email "$SEED_EMAIL" \
    --arg pass "$SEED_PASSWORD" \
    --arg username "$SEED_USERNAME" \
    --arg community "$SEED_COMMUNITY_ID" \
    '{email:$email, pass:$pass, username:$username, community_id:$community}')"
  local signup_status
  signup_status="$(curl_json_public POST '/v1/auth/signup' "$signup_body" "$signup_out")"

  if [[ "$signup_status" == "200" || "$signup_status" == "201" ]]; then
    seed_token="$(jq -r '.access_token // empty' "$signup_out")"
    seed_user_id="$(jq -r '.user_id // empty' "$signup_out")"
    if [[ -z "$seed_token" || -z "$seed_user_id" ]]; then
      echo "Auth signup succeeded but response missing token/user_id: $(cat "$signup_out")" >&2
      exit 1
    fi
    echo "Seed auth: created user ${SEED_EMAIL} (${seed_user_id})"
    return 0
  fi

  local signin_out="$tmp_dir/auth-signin.json"
  local signin_body
  signin_body="$(jq -nc --arg email "$SEED_EMAIL" --arg pass "$SEED_PASSWORD" '{email:$email, pass:$pass}')"
  local signin_status
  signin_status="$(curl_json_public POST '/v1/auth/signin' "$signin_body" "$signin_out")"
  if [[ "$signin_status" != "200" ]]; then
    echo "Failed to bootstrap seed auth." >&2
    echo "Signup response (HTTP ${signup_status}): $(cat "$signup_out")" >&2
    echo "Signin response (HTTP ${signin_status}): $(cat "$signin_out")" >&2
    exit 1
  fi

  seed_token="$(jq -r '.access_token // empty' "$signin_out")"
  seed_user_id="$(jq -r '.user_id // empty' "$signin_out")"
  if [[ -z "$seed_token" || -z "$seed_user_id" ]]; then
    echo "Auth signin succeeded but response missing token/user_id: $(cat "$signin_out")" >&2
    exit 1
  fi
  echo "Seed auth: signed in existing user ${SEED_EMAIL} (${seed_user_id})"
}

echo "=== Seeding local API data ==="
echo "API base: ${API_BASE_URL}"
ensure_seed_session

run_key="${SEED_PREFIX}-$(date +%Y%m%d%H%M%S)"

for i in $(seq 1 "$SEED_COUNT"); do
  witness_id="${run_key}-witness-${i}"
  feed_id="${run_key}-feed-${i}"
  request_id="${run_key}-req-${i}"

  title="[${SEED_PREFIX}] Contoh Saksi #${i}"
  summary="Data sampel lokal untuk pengembangan frontend (${witness_id})."

  feed_payload="$(jq -nc \
    --arg witness_id "$witness_id" \
    --arg route "komunitas" \
    --arg rahasia_level "L0" \
    --arg entity_id "ent-${SEED_COMMUNITY_ID}" \
    '{witness_id:$witness_id, route:$route, rahasia_level:$rahasia_level, status:"open", message_count:0, unread_count:0, entity_ids:[$entity_id]}')"

  sql_feed="CREATE discovery_feed_item CONTENT {
    feed_id: '${feed_id}',
    source_type: 'contribution',
    source_id: '${witness_id}',
    actor_id: '${seed_user_id}',
    actor_username: '${SEED_USERNAME}',
    title: '${title}',
    summary: '${summary}',
    scope_id: '${SEED_COMMUNITY_ID}',
    privacy_level: 'public',
    occurred_at: time::now(),
    created_at: time::now(),
    request_id: '${request_id}',
    correlation_id: '${request_id}',
    participant_ids: ['${seed_user_id}'],
    payload: ${feed_payload}
  };
  CREATE feed_participant_edge CONTENT {
    edge_id: '${seed_user_id}:${feed_id}',
    actor_id: '${seed_user_id}',
    feed_id: '${feed_id}',
    occurred_at: time::now(),
    scope_id: '${SEED_COMMUNITY_ID}',
    privacy_level: 'public',
    source_type: 'contribution',
    source_id: '${witness_id}',
    created_at: time::now(),
    request_id: '${request_id}'
  };"

  surreal_exec "$sql_feed"
  echo "Seeded feed witness: ${witness_id}"
done

feed_out="$tmp_dir/feed.json"
feed_status="$(curl_get_authed '/v1/feed?limit=20' "$seed_token" "$feed_out")"
if [[ "$feed_status" != "200" ]]; then
  echo "Failed to read /v1/feed after seeding (HTTP ${feed_status})" >&2
  echo "Response: $(cat "$feed_out")" >&2
  exit 1
fi

feed_count="$(jq '.items | length' "$feed_out")"

echo "=== Seed complete ==="
echo "Seed user: ${SEED_EMAIL}"
echo "Seeded records this run: ${SEED_COUNT}"
echo "Feed items now visible: ${feed_count}"
