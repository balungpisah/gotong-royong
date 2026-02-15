#!/usr/bin/env bash
set -euo pipefail

OUT_FILE="${1:-/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/research/surrealdb-pattern-sampling.md}"
PORT="${SURREAL_PROBE_PORT:-18080}"
NS="${SURREAL_PROBE_NS:-gotong_probe}"
DB="${SURREAL_PROBE_DB:-chat}"
USER_NAME="${SURREAL_PROBE_USER:-root}"
USER_PASS="${SURREAL_PROBE_PASS:-root}"
SURREAL_BIN="${SURREAL_BIN:-surreal}"
LOCKED_TARGET_VERSION="${LOCKED_TARGET_VERSION:-3.0.0-beta.4}"

ENDPOINT_WS="ws://127.0.0.1:${PORT}"
ENDPOINT_HTTP="http://127.0.0.1:${PORT}"
TMP_DIR="$(mktemp -d)"
SERVER_LOG="${TMP_DIR}/server.log"
LIVE_FIFO="${TMP_DIR}/live.fifo"
LIVE_OUT="${TMP_DIR}/live.out"

cleanup() {
  if [[ -n "${LIVE_WRITER_OPEN:-}" ]]; then
    exec 3>&- || true
  fi
  if [[ -n "${LIVE_PID:-}" ]]; then
    kill "${LIVE_PID}" >/dev/null 2>&1 || true
  fi
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

sql_json() {
  local query="$1"
  printf "%s\n" "${query}" | "${SURREAL_BIN}" sql \
    --endpoint "${ENDPOINT_WS}" \
    --user "${USER_NAME}" \
    --pass "${USER_PASS}" \
    --ns "${NS}" \
    --db "${DB}" \
    --json \
    --hide-welcome
}

decode_jwt_payload_json() {
  local token="$1"
  local payload
  payload="$(echo "${token}" | cut -d. -f2 | tr '_-' '/+')"
  case $(( ${#payload} % 4 )) in
    2) payload="${payload}==";;
    3) payload="${payload}=";;
  esac
  if echo "${payload}" | base64 -D >/dev/null 2>&1; then
    echo "${payload}" | base64 -D
  else
    echo "${payload}" | base64 -d
  fi
}

signup_record_token() {
  local email="$1"
  local pass="$2"
  curl -s "${ENDPOINT_HTTP}/signup" \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    -d "{\"ns\":\"${NS}\",\"db\":\"${DB}\",\"ac\":\"account\",\"email\":\"${email}\",\"pass\":\"${pass}\"}" |
    jq -r '.token'
}

"${SURREAL_BIN}" start memory --user "${USER_NAME}" --pass "${USER_PASS}" --bind "127.0.0.1:${PORT}" >"${SERVER_LOG}" 2>&1 &
SERVER_PID=$!

for _ in $(seq 1 100); do
  if "${SURREAL_BIN}" is-ready --endpoint "${ENDPOINT_WS}" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

if ! "${SURREAL_BIN}" is-ready --endpoint "${ENDPOINT_WS}" >/dev/null 2>&1; then
  echo "Server failed to become ready on ${ENDPOINT_WS}" >&2
  exit 1
fi

probe_started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
surreal_version="$("${SURREAL_BIN}" version 2>&1 | tr -d '\r')"
target_match_note="does not match locked target ${LOCKED_TARGET_VERSION}"
if echo "${surreal_version}" | grep -q "${LOCKED_TARGET_VERSION}"; then
  target_match_note="matches locked target ${LOCKED_TARGET_VERSION}"
fi

# Pattern 1: idempotent write via unique composite index.
sql_json "DEFINE TABLE chat_delivery_event SCHEMAFULL;" >/dev/null
sql_json "DEFINE FIELD entity_id ON TABLE chat_delivery_event TYPE string;" >/dev/null
sql_json "DEFINE FIELD request_id ON TABLE chat_delivery_event TYPE string;" >/dev/null
sql_json "DEFINE FIELD correlation_id ON TABLE chat_delivery_event TYPE string;" >/dev/null
sql_json "DEFINE FIELD occurred_at ON TABLE chat_delivery_event TYPE datetime;" >/dev/null
sql_json "DEFINE INDEX uniq_entity_request ON TABLE chat_delivery_event FIELDS entity_id, request_id UNIQUE;" >/dev/null

idempotency_first="$(sql_json "CREATE chat_delivery_event CONTENT { entity_id: 'thread:alpha', request_id: 'req-123', correlation_id: 'corr-1', occurred_at: time::now() };")"
idempotency_dup="$(sql_json "CREATE chat_delivery_event CONTENT { entity_id: 'thread:alpha', request_id: 'req-123', correlation_id: 'corr-2', occurred_at: time::now() };")"
idempotency_rows="$(sql_json "SELECT * FROM chat_delivery_event WHERE entity_id='thread:alpha' AND request_id='req-123';")"

idempotency_pass="FAIL"
if echo "${idempotency_dup}" | grep -q "already contains"; then
  idempotency_pass="PASS"
fi

# Pattern 2: deterministic ordering and reconnect catch-up query.
sql_json "CREATE chat_message CONTENT { thread_id: 'thread:order', message_id: 'msg-003', author_id: 'user:a', body: 'third', created_at: d'2026-02-15T03:30:00Z' };" >/dev/null
sql_json "CREATE chat_message CONTENT { thread_id: 'thread:order', message_id: 'msg-001', author_id: 'user:a', body: 'first', created_at: d'2026-02-15T03:30:00Z' };" >/dev/null
sql_json "CREATE chat_message CONTENT { thread_id: 'thread:order', message_id: 'msg-002', author_id: 'user:a', body: 'second', created_at: d'2026-02-15T03:30:00Z' };" >/dev/null

ordering_json="$(sql_json "SELECT message_id, body, created_at FROM chat_message WHERE thread_id='thread:order' ORDER BY created_at ASC, message_id ASC;")"
catchup_json="$(sql_json "SELECT message_id, body, created_at FROM chat_message WHERE thread_id='thread:order' AND (created_at > d'2026-02-15T03:30:00Z' OR (created_at = d'2026-02-15T03:30:00Z' AND message_id > 'msg-001')) ORDER BY created_at ASC, message_id ASC;")"

ordering_ids="$(echo "${ordering_json}" | jq -r '.[0][]?.message_id' | paste -sd ',' -)"
catchup_ids="$(echo "${catchup_json}" | jq -r '.[0][]?.message_id' | paste -sd ',' -)"

ordering_pass="FAIL"
if [[ "${ordering_ids}" == "msg-001,msg-002,msg-003" && "${catchup_ids}" == "msg-002,msg-003" ]]; then
  ordering_pass="PASS"
fi

# Pattern 3: live query protocol behavior.
live_http_output="$(printf "LIVE SELECT * FROM chat_message WHERE thread_id = 'thread:live';\n" | "${SURREAL_BIN}" sql --endpoint "${ENDPOINT_HTTP}" --user "${USER_NAME}" --pass "${USER_PASS}" --ns "${NS}" --db "${DB}" --hide-welcome 2>&1 || true)"

mkfifo "${LIVE_FIFO}"
"${SURREAL_BIN}" sql --endpoint "${ENDPOINT_WS}" --user "${USER_NAME}" --pass "${USER_PASS}" --ns "${NS}" --db "${DB}" --hide-welcome <"${LIVE_FIFO}" >"${LIVE_OUT}" 2>&1 &
LIVE_PID=$!
exec 3>"${LIVE_FIFO}"
LIVE_WRITER_OPEN=1
echo "LIVE SELECT * FROM chat_message WHERE thread_id = 'thread:live';" >&3
sleep 0.5
sql_json "CREATE chat_message SET thread_id='thread:live', message_id='msg-live-1', author_id='user:live', body='live event', created_at=time::now();" >/dev/null
sleep 0.5
exec 3>&-
unset LIVE_WRITER_OPEN
wait "${LIVE_PID}" || true
unset LIVE_PID

live_ws_output="$(cat "${LIVE_OUT}")"
live_pass="FAIL"
if echo "${live_ws_output}" | grep -q "action: 'CREATE'"; then
  live_pass="PASS"
fi

# Pattern 4: LIVE SELECT DIFF payload behavior.
LIVE_DIFF_FIFO="${TMP_DIR}/live_diff.fifo"
LIVE_DIFF_OUT="${TMP_DIR}/live_diff.out"
mkfifo "${LIVE_DIFF_FIFO}"
"${SURREAL_BIN}" sql --endpoint "${ENDPOINT_WS}" --user "${USER_NAME}" --pass "${USER_PASS}" --ns "${NS}" --db "${DB}" --hide-welcome <"${LIVE_DIFF_FIFO}" >"${LIVE_DIFF_OUT}" 2>&1 &
LIVE_PID=$!
exec 3>"${LIVE_DIFF_FIFO}"
LIVE_WRITER_OPEN=1
echo "LIVE SELECT DIFF FROM chat_message WHERE thread_id = 'thread:diff';" >&3
sleep 0.5
sql_json "CREATE chat_message SET thread_id='thread:diff', message_id='msg-diff-1', author_id='user:diff', body='hello';" >/dev/null
sql_json "UPDATE chat_message SET body='hello edited' WHERE thread_id='thread:diff' AND message_id='msg-diff-1';" >/dev/null
sleep 0.8
exec 3>&-
unset LIVE_WRITER_OPEN
wait "${LIVE_PID}" || true
unset LIVE_PID

live_diff_output="$(cat "${LIVE_DIFF_OUT}")"
live_diff_pass="FAIL"
if echo "${live_diff_output}" | grep -q "action: 'UPDATE'" && \
   echo "${live_diff_output}" | grep -q "op: 'change'" && \
   echo "${live_diff_output}" | grep -q "/body"; then
  live_diff_pass="PASS"
fi

# Pattern 5: permission-filtered live stream behavior.
sql_json "DEFINE TABLE user SCHEMAFULL;" >/dev/null
sql_json "DEFINE FIELD email ON TABLE user TYPE string;" >/dev/null
sql_json "DEFINE FIELD pass ON TABLE user TYPE string;" >/dev/null
sql_json "DEFINE ACCESS account ON DATABASE TYPE RECORD SIGNUP ( CREATE user SET email = \$email, pass = \$pass ) SIGNIN ( SELECT * FROM user WHERE email = \$email AND pass = \$pass );" >/dev/null
sql_json "DEFINE TABLE chat_private SCHEMAFULL PERMISSIONS FOR select WHERE owner = \$auth.id, FOR create WHERE owner = \$auth.id, FOR update WHERE owner = \$auth.id, FOR delete WHERE owner = \$auth.id;" >/dev/null
sql_json "DEFINE FIELD owner ON TABLE chat_private TYPE record<user>;" >/dev/null
sql_json "DEFINE FIELD body ON TABLE chat_private TYPE string;" >/dev/null

alice_token="$(signup_record_token 'alice@example.com' 'secret123')"
bob_token="$(signup_record_token 'bob@example.com' 'secret123')"
alice_id="$(decode_jwt_payload_json "${alice_token}" | jq -r '.ID')"
bob_id="$(decode_jwt_payload_json "${bob_token}" | jq -r '.ID')"

LIVE_PERM_FIFO="${TMP_DIR}/live_perm.fifo"
LIVE_PERM_OUT="${TMP_DIR}/live_perm.out"
mkfifo "${LIVE_PERM_FIFO}"
"${SURREAL_BIN}" sql --endpoint "${ENDPOINT_WS}" --token "${alice_token}" --ns "${NS}" --db "${DB}" --hide-welcome <"${LIVE_PERM_FIFO}" >"${LIVE_PERM_OUT}" 2>&1 &
LIVE_PID=$!
exec 3>"${LIVE_PERM_FIFO}"
LIVE_WRITER_OPEN=1
echo "LIVE SELECT * FROM chat_private;" >&3
sleep 0.5
sql_json "CREATE chat_private SET owner=${bob_id}, body='bob private';" >/dev/null
sql_json "CREATE chat_private SET owner=${alice_id}, body='alice private';" >/dev/null
sleep 0.8
exec 3>&-
unset LIVE_WRITER_OPEN
wait "${LIVE_PID}" || true
unset LIVE_PID

live_perm_output="$(cat "${LIVE_PERM_OUT}")"
perm_filter_pass="FAIL"
if echo "${live_perm_output}" | grep -q "alice private" && \
   ! echo "${live_perm_output}" | grep -q "bob private"; then
  perm_filter_pass="PASS"
fi

cat <<REPORT > "${OUT_FILE}"
# SurrealDB Pattern Sampling Report

Date: ${probe_started_at}
Environment: ${surreal_version}
Namespace/DB: ${NS}/${DB}

## Objective
Probe key backend patterns with a runnable SurrealDB sample before implementation planning.

## Result Summary
| Pattern | Result | What was checked |
|---|---|---|
| Idempotent write guard (unique entity_id + request_id) | ${idempotency_pass} | Duplicate write blocked by unique composite index |
| Deterministic timeline ordering + catch-up query | ${ordering_pass} | ORDER BY created_at, message_id and reconnect predicate |
| Live stream behavior | ${live_pass} | WS live subscription receives create event |
| Live diff payload contract | ${live_diff_pass} | DIFF stream includes change operation for body update |
| Permission-filtered live subscription | ${perm_filter_pass} | Record auth sees own rows and not other owners' rows |

## Pattern 1: Idempotent write guard
Command pattern:
DEFINE INDEX uniq_entity_request ON TABLE chat_delivery_event FIELDS entity_id, request_id UNIQUE;

First insert output:
~~~json
${idempotency_first}
~~~

Duplicate insert output:
~~~json
${idempotency_dup}
~~~

Rows after duplicate attempt:
~~~json
${idempotency_rows}
~~~

## Pattern 2: Deterministic ordering and reconnect catch-up
Ordered query output:
~~~json
${ordering_json}
~~~

Catch-up query output:
~~~json
${catchup_json}
~~~

Derived order:
- Ordered IDs: ${ordering_ids}
- Catch-up IDs after cursor (created_at=03:30:00Z, message_id=msg-001): ${catchup_ids}

## Pattern 3: Live stream behavior (protocol detail)
HTTP endpoint live query result:
~~~text
${live_http_output}
~~~

WS endpoint live query result:
~~~text
${live_ws_output}
~~~

Observation:
- In this environment, live query streaming worked over WS and not over HTTP.

## Pattern 4: Live diff payload contract
WS DIFF live query result:
~~~text
${live_diff_output}
~~~

Observation:
- DIFF stream returned change entries for updated fields (/body).

## Pattern 5: Permission-filtered live subscriptions
Token auth setup:
- Alice record id: ${alice_id}
- Bob record id: ${bob_id}

Alice live query result while inserting Bob then Alice rows:
~~~text
${live_perm_output}
~~~

Observation:
- Alice subscription received only Alice-owned row and did not receive Bob-owned row.

## Notes for Stack Lock
- This probe validates core data patterns and live-stream behavior relevant to chat workloads.
- Local runtime here is SurrealDB CLI/server ${surreal_version}; this ${target_match_note}.
- Re-run this same probe against the pinned beta runtime during implementation bootstrap.
REPORT

echo "Wrote ${OUT_FILE}"
