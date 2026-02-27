#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

: "${API_BASE_URL:=http://127.0.0.1:3100}"
: "${SEED_COUNT:=0}"
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
: "${SEED_MATRIX_FILE:=${ROOT_DIR}/scripts/dev/seed-feed-matrix.json}"

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

sql_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\'/\\\'}"
  printf '%s' "$value"
}

if ! [[ "$SEED_COUNT" =~ ^[0-9]+$ ]]; then
  echo "SEED_COUNT must be a non-negative integer (0 means all matrix rows)" >&2
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
if [[ ! -f "$SEED_MATRIX_FILE" ]]; then
  echo "Seed matrix file not found: ${SEED_MATRIX_FILE}" >&2
  exit 1
fi

matrix_count="$(jq -r 'if type == "array" then length else -1 end' "$SEED_MATRIX_FILE")"
if ! [[ "$matrix_count" =~ ^[0-9]+$ ]] || [[ "$matrix_count" -lt 1 ]]; then
  echo "Seed matrix must be a non-empty JSON array: ${SEED_MATRIX_FILE}" >&2
  exit 1
fi

seed_limit="$matrix_count"
if [[ "$SEED_COUNT" -gt 0 ]]; then
  if [[ "$SEED_COUNT" -lt "$matrix_count" ]]; then
    seed_limit="$SEED_COUNT"
  else
    echo "SEED_COUNT=${SEED_COUNT} exceeds matrix size (${matrix_count}); seeding all rows."
  fi
fi

echo "Seed matrix: ${SEED_MATRIX_FILE}"
echo "Matrix rows: ${matrix_count} (seeding ${seed_limit})"

seeded_count=0
for idx in $(seq 0 $((seed_limit - 1))); do
  entry="$(jq -c ".[$idx]" "$SEED_MATRIX_FILE")"
  slug="$(jq -r '.slug // empty' <<<"$entry")"
  if [[ -z "$slug" ]]; then
    slug="item-$((idx + 1))"
  fi
  slug_safe="$(echo "$slug" | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9_.-' '-' | sed 's/^-*//; s/-*$//')"
  if [[ -z "$slug_safe" ]]; then
    slug_safe="item-$((idx + 1))"
  fi

  source_type="$(jq -r '.source_type // "contribution"' <<<"$entry")"
  route="$(jq -r '.route // "komunitas"' <<<"$entry")"
  rahasia_level="$(jq -r '.rahasia_level // "L0"' <<<"$entry")"
  scope_id="$(jq -r --arg default "$SEED_COMMUNITY_ID" '.scope_id // $default' <<<"$entry")"
  privacy_level="$(jq -r '.privacy_level // "public"' <<<"$entry")"
  title="$(jq -r '.title // empty' <<<"$entry")"
  summary="$(jq -r '.summary // empty' <<<"$entry")"
  payload_base="$(jq -c '.payload // {}' <<<"$entry")"

  if [[ -z "$title" ]]; then
    title="[${SEED_PREFIX}] Seed ${slug_safe}"
  fi
  if [[ -z "$summary" ]]; then
    summary="Data sampel lokal untuk pengembangan frontend (${slug_safe})."
  fi

  witness_id="seed-${run_key}-${slug_safe}"
  feed_id="${run_key}-feed-${slug_safe}"
  request_id="${run_key}-req-${slug_safe}"

  feed_payload="$(jq -nc \
    --argjson payload_base "$payload_base" \
    --arg witness_id "$witness_id" \
    --arg route "$route" \
    --arg rahasia_level "$rahasia_level" \
    --arg entity_id "ent-${SEED_COMMUNITY_ID}" \
    --arg seed_batch_id "$run_key" \
    '(
      if ($payload_base | type) == "object" then $payload_base else {} end
    ) + {
      witness_id: $witness_id,
      route: $route,
      rahasia_level: $rahasia_level,
      status: "open",
      message_count: 0,
      unread_count: 0,
      entity_ids: (
        if ($payload_base.entity_ids | type) == "array"
        then $payload_base.entity_ids
        else [$entity_id]
        end
      )
    } | .dev_meta = {
      is_seed: true,
      seed_origin: "db",
      seed_batch_id: $seed_batch_id
    }')"

  sql_feed_id="$(sql_escape "$feed_id")"
  sql_source_type="$(sql_escape "$source_type")"
  sql_witness_id="$(sql_escape "$witness_id")"
  sql_seed_user_id="$(sql_escape "$seed_user_id")"
  sql_seed_username="$(sql_escape "$SEED_USERNAME")"
  sql_title="$(sql_escape "$title")"
  sql_summary="$(sql_escape "$summary")"
  sql_scope_id="$(sql_escape "$scope_id")"
  sql_privacy_level="$(sql_escape "$privacy_level")"
  sql_request_id="$(sql_escape "$request_id")"

  sql_feed="CREATE discovery_feed_item CONTENT {
    feed_id: '${sql_feed_id}',
    source_type: '${sql_source_type}',
    source_id: '${sql_witness_id}',
    actor_id: '${sql_seed_user_id}',
    actor_username: '${sql_seed_username}',
    title: '${sql_title}',
    summary: '${sql_summary}',
    scope_id: '${sql_scope_id}',
    privacy_level: '${sql_privacy_level}',
    occurred_at: time::now(),
    created_at: time::now(),
    request_id: '${sql_request_id}',
    correlation_id: '${sql_request_id}',
    participant_ids: ['${sql_seed_user_id}'],
    payload: ${feed_payload}
  };
  CREATE feed_participant_edge CONTENT {
    edge_id: '${sql_seed_user_id}:${sql_feed_id}',
    actor_id: '${sql_seed_user_id}',
    feed_id: '${sql_feed_id}',
    occurred_at: time::now(),
    scope_id: '${sql_scope_id}',
    privacy_level: '${sql_privacy_level}',
    source_type: '${sql_source_type}',
    source_id: '${sql_witness_id}',
    created_at: time::now(),
    request_id: '${sql_request_id}'
  };"

  surreal_exec "$sql_feed"
  echo "Seeded feed item: ${witness_id} (${source_type})"
  seeded_count=$((seeded_count + 1))
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
echo "Seeded records this run: ${seeded_count}"
echo "Feed items now visible: ${feed_count}"
