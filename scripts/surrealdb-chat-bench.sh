#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

compose_file="${COMPOSE_FILE:-compose.dev.yaml}"
output_file="${1:-${ROOT_DIR}/docs/research/surrealdb-chat-bench-latest.md}"

: "${SURREAL_PORT:=8000}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"
: "${PROBE_NS:=gotong_chat_probe}"
: "${PROBE_DB:=bench}"
: "${HOT_THREAD_ID:=thread-hot}"
: "${HOT_USER_ID:=u-hot}"
: "${HOT_THREAD_ROWS:=10000}"
: "${NOISE_ROWS:=50000}"
: "${BENCH_LOOPS:=120}"

endpoint_ws="ws://127.0.0.1:${SURREAL_PORT}"
endpoint_http="http://127.0.0.1:${SURREAL_PORT}"

surreal_sql_root_json() {
  local sql="$1"
  docker compose -f "${compose_file}" exec -T surrealdb /surreal sql \
    --endpoint "${endpoint_ws}" \
    --user "${SURREAL_USER}" \
    --pass "${SURREAL_PASS}" \
    --ns "${PROBE_NS}" \
    --db "${PROBE_DB}" \
    --json \
    --hide-welcome <<SQL | sed '/^onnxruntime cpuid_info warning:/d'
${sql}
SQL
}

ensure_dev_db_up() {
  docker compose -f "${compose_file}" up -d >/dev/null
  for _ in $(seq 1 60); do
    if docker compose -f "${compose_file}" exec -T surrealdb /surreal is-ready --endpoint "${endpoint_ws}" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.5
  done
  echo "SurrealDB did not become ready (${endpoint_ws})" >&2
  exit 1
}

json_extract() {
  local json_payload="$1"
  local mode="$2"
  JSON_PAYLOAD="${json_payload}" python3 - "${mode}" <<'PY'
import json
import os
import sys

def find_operator(node, name):
    if isinstance(node, dict):
        if node.get("operator") == name:
            return node
        for child in node.get("children", []) or []:
            found = find_operator(child, name)
            if found is not None:
                return found
    return None

raw = os.environ.get("JSON_PAYLOAD", "").strip()
mode = sys.argv[1]
if not raw:
    print("n/a")
    raise SystemExit(0)

try:
    parsed = json.loads(raw)
except Exception:
    print("n/a")
    raise SystemExit(0)

if isinstance(parsed, list) and parsed:
    root = parsed[0]
else:
    root = parsed

if mode == "has_index_scan":
    print("yes" if find_operator(root, "IndexScan") else "no")
elif mode == "index_name":
    op = find_operator(root, "IndexScan")
    print(op.get("attributes", {}).get("index", "n/a") if op else "n/a")
elif mode == "index_elapsed_ns":
    op = find_operator(root, "IndexScan")
    print(op.get("metrics", {}).get("elapsed_ns", "n/a") if op else "n/a")
elif mode == "index_output_rows":
    op = find_operator(root, "IndexScan")
    print(op.get("metrics", {}).get("output_rows", "n/a") if op else "n/a")
elif mode == "filter_elapsed_ns":
    op = find_operator(root, "Filter")
    print(op.get("metrics", {}).get("elapsed_ns", "n/a") if op else "n/a")
elif mode == "sort_elapsed_ns":
    op = find_operator(root, "SortTopKByKey")
    print(op.get("metrics", {}).get("elapsed_ns", "n/a") if op else "n/a")
elif mode == "loop_duration":
    if isinstance(parsed, list):
        for item in reversed(parsed):
            if isinstance(item, str):
                print(item)
                break
        else:
            print("n/a")
    elif isinstance(parsed, str):
        print(parsed)
    else:
        print("n/a")
else:
    print("n/a")
PY
}

ensure_dev_db_up

server_version="$(curl -fsS "${endpoint_http}/version" 2>/dev/null || true)"
if [[ -z "${server_version}" ]]; then
  server_version="(unknown; /version failed)"
fi

surreal_sql_root_json "REMOVE TABLE probe_chat_message_bench;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_chat_member_bench;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_chat_cursor_bench;" >/dev/null || true

surreal_sql_root_json $'DEFINE TABLE probe_chat_message_bench SCHEMAFULL;\nDEFINE FIELD thread_id ON TABLE probe_chat_message_bench TYPE string;\nDEFINE FIELD message_id ON TABLE probe_chat_message_bench TYPE string;\nDEFINE FIELD request_id ON TABLE probe_chat_message_bench TYPE string;\nDEFINE FIELD author_id ON TABLE probe_chat_message_bench TYPE string;\nDEFINE FIELD body ON TABLE probe_chat_message_bench TYPE string;\nDEFINE FIELD created_at ON TABLE probe_chat_message_bench TYPE datetime;\nDEFINE INDEX idx_probe_message_order ON TABLE probe_chat_message_bench FIELDS thread_id, created_at, message_id;\nDEFINE INDEX uniq_probe_message_request ON TABLE probe_chat_message_bench FIELDS thread_id, request_id UNIQUE;' >/dev/null

surreal_sql_root_json $'DEFINE TABLE probe_chat_member_bench SCHEMAFULL;\nDEFINE FIELD thread_id ON TABLE probe_chat_member_bench TYPE string;\nDEFINE FIELD user_id ON TABLE probe_chat_member_bench TYPE string;\nDEFINE FIELD role ON TABLE probe_chat_member_bench TYPE string;\nDEFINE FIELD joined_at ON TABLE probe_chat_member_bench TYPE datetime;\nDEFINE FIELD left_at ON TABLE probe_chat_member_bench TYPE option<datetime>;\nDEFINE INDEX idx_probe_member_lookup ON TABLE probe_chat_member_bench FIELDS user_id, thread_id;' >/dev/null

surreal_sql_root_json $'DEFINE TABLE probe_chat_cursor_bench SCHEMAFULL;\nDEFINE FIELD thread_id ON TABLE probe_chat_cursor_bench TYPE string;\nDEFINE FIELD user_id ON TABLE probe_chat_cursor_bench TYPE string;\nDEFINE FIELD message_id ON TABLE probe_chat_cursor_bench TYPE string;\nDEFINE FIELD read_at ON TABLE probe_chat_cursor_bench TYPE datetime;\nDEFINE INDEX idx_probe_read_cursor_lookup ON TABLE probe_chat_cursor_bench FIELDS user_id, thread_id;\nDEFINE INDEX uniq_probe_read_cursor_key ON TABLE probe_chat_cursor_bench FIELDS thread_id, user_id UNIQUE;' >/dev/null

surreal_sql_root_json "FOR \$i IN 0..${HOT_THREAD_ROWS} { LET \$created_at = time::now() - duration::from_millis(<int>(${HOT_THREAD_ROWS} - <int>\$i)); CREATE probe_chat_message_bench SET thread_id='${HOT_THREAD_ID}', message_id=string::concat('msg-hot-', <string>\$i), request_id=string::concat('req-hot-', <string>\$i), author_id=string::concat('u-', <string>(<int>\$i % 50)), body='hot thread message', created_at=\$created_at; };" >/dev/null

if [[ "${NOISE_ROWS}" -gt 0 ]]; then
  surreal_sql_root_json "FOR \$i IN 0..${NOISE_ROWS} { LET \$noise_thread = string::concat('thread-noise-', <string>(<int>\$i % 5000)); LET \$created_at = time::now() - duration::from_millis(<int>(\$i + ${HOT_THREAD_ROWS})); CREATE probe_chat_message_bench SET thread_id=\$noise_thread, message_id=string::concat('msg-noise-', <string>\$i), request_id=string::concat('req-noise-', <string>\$i), author_id=string::concat('u-noise-', <string>(<int>\$i % 10000)), body='noise message', created_at=\$created_at; };" >/dev/null
fi

surreal_sql_root_json "CREATE probe_chat_member_bench SET thread_id='${HOT_THREAD_ID}', user_id='${HOT_USER_ID}', role='member', joined_at=time::now() - 3d, left_at=NONE;" >/dev/null
surreal_sql_root_json "CREATE probe_chat_cursor_bench SET thread_id='${HOT_THREAD_ID}', user_id='${HOT_USER_ID}', message_id='msg-hot-5000', read_at=time::now();" >/dev/null

message_count_json="$(surreal_sql_root_json "SELECT count() FROM probe_chat_message_bench GROUP ALL;")"
message_count="$(printf "%s" "${message_count_json}" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(data[0][0]["count"])')"

hot_message_count_json="$(surreal_sql_root_json "SELECT count() FROM probe_chat_message_bench WHERE thread_id='${HOT_THREAD_ID}' GROUP ALL;")"
hot_message_count="$(printf "%s" "${hot_message_count_json}" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(data[0][0]["count"])')"

cursor_row_json="$(surreal_sql_root_json "SELECT created_at, message_id FROM probe_chat_message_bench WHERE thread_id='${HOT_THREAD_ID}' ORDER BY created_at ASC, message_id ASC LIMIT 1 START 5000;")"
cursor_created_at="$(printf "%s" "${cursor_row_json}" | python3 -c 'import json,sys; data=json.load(sys.stdin); row=data[0][0] if data and data[0] else {}; print(row.get("created_at", ""))')"
cursor_message_id="$(printf "%s" "${cursor_row_json}" | python3 -c 'import json,sys; data=json.load(sys.stdin); row=data[0][0] if data and data[0] else {}; print(row.get("message_id", ""))')"
if [[ -z "${cursor_created_at}" || -z "${cursor_message_id}" ]]; then
  echo "Failed to derive benchmark cursor row from hot thread seed" >&2
  exit 1
fi

catchup_explain="$(surreal_sql_root_json "SELECT message_id, created_at FROM probe_chat_message_bench WHERE thread_id='${HOT_THREAD_ID}' AND (created_at > <datetime>'${cursor_created_at}' OR (created_at = <datetime>'${cursor_created_at}' AND message_id > '${cursor_message_id}')) ORDER BY created_at ASC, message_id ASC LIMIT 50 EXPLAIN FULL;")"

idempotent_lookup_explain="$(surreal_sql_root_json "SELECT message_id, request_id FROM probe_chat_message_bench WHERE thread_id='${HOT_THREAD_ID}' AND request_id='req-hot-5000' LIMIT 1 EXPLAIN FULL;")"

member_lookup_explain="$(surreal_sql_root_json "SELECT user_id, role, joined_at FROM probe_chat_member_bench WHERE user_id='${HOT_USER_ID}' AND thread_id='${HOT_THREAD_ID}' AND left_at IS NONE ORDER BY joined_at DESC LIMIT 1 EXPLAIN FULL;")"

read_cursor_lookup_explain="$(surreal_sql_root_json "SELECT user_id, thread_id, message_id, read_at FROM probe_chat_cursor_bench WHERE user_id='${HOT_USER_ID}' AND thread_id='${HOT_THREAD_ID}' LIMIT 1 EXPLAIN FULL;")"

catchup_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT message_id, created_at FROM probe_chat_message_bench WHERE thread_id='${HOT_THREAD_ID}' AND (created_at > <datetime>'${cursor_created_at}' OR (created_at = <datetime>'${cursor_created_at}' AND message_id > '${cursor_message_id}')) ORDER BY created_at ASC, message_id ASC LIMIT 50; }; RETURN time::now() - \$start;")"

idempotent_lookup_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT message_id, request_id FROM probe_chat_message_bench WHERE thread_id='${HOT_THREAD_ID}' AND request_id='req-hot-5000' LIMIT 1; }; RETURN time::now() - \$start;")"

member_lookup_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT user_id, role, joined_at FROM probe_chat_member_bench WHERE user_id='${HOT_USER_ID}' AND thread_id='${HOT_THREAD_ID}' AND left_at IS NONE ORDER BY joined_at DESC LIMIT 1; }; RETURN time::now() - \$start;")"

read_cursor_lookup_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT user_id, thread_id, message_id, read_at FROM probe_chat_cursor_bench WHERE user_id='${HOT_USER_ID}' AND thread_id='${HOT_THREAD_ID}' LIMIT 1; }; RETURN time::now() - \$start;")"

catchup_has_index="$(json_extract "${catchup_explain}" has_index_scan)"
catchup_index_name="$(json_extract "${catchup_explain}" index_name)"
catchup_index_rows="$(json_extract "${catchup_explain}" index_output_rows)"
catchup_index_elapsed_ns="$(json_extract "${catchup_explain}" index_elapsed_ns)"
catchup_filter_elapsed_ns="$(json_extract "${catchup_explain}" filter_elapsed_ns)"
catchup_sort_elapsed_ns="$(json_extract "${catchup_explain}" sort_elapsed_ns)"
catchup_loop_duration="$(json_extract "${catchup_loop}" loop_duration)"

idempotent_has_index="$(json_extract "${idempotent_lookup_explain}" has_index_scan)"
idempotent_index_name="$(json_extract "${idempotent_lookup_explain}" index_name)"
idempotent_index_rows="$(json_extract "${idempotent_lookup_explain}" index_output_rows)"
idempotent_index_elapsed_ns="$(json_extract "${idempotent_lookup_explain}" index_elapsed_ns)"
idempotent_filter_elapsed_ns="$(json_extract "${idempotent_lookup_explain}" filter_elapsed_ns)"
idempotent_sort_elapsed_ns="$(json_extract "${idempotent_lookup_explain}" sort_elapsed_ns)"
idempotent_loop_duration="$(json_extract "${idempotent_lookup_loop}" loop_duration)"

member_has_index="$(json_extract "${member_lookup_explain}" has_index_scan)"
member_index_name="$(json_extract "${member_lookup_explain}" index_name)"
member_index_rows="$(json_extract "${member_lookup_explain}" index_output_rows)"
member_index_elapsed_ns="$(json_extract "${member_lookup_explain}" index_elapsed_ns)"
member_filter_elapsed_ns="$(json_extract "${member_lookup_explain}" filter_elapsed_ns)"
member_sort_elapsed_ns="$(json_extract "${member_lookup_explain}" sort_elapsed_ns)"
member_loop_duration="$(json_extract "${member_lookup_loop}" loop_duration)"

cursor_has_index="$(json_extract "${read_cursor_lookup_explain}" has_index_scan)"
cursor_index_name="$(json_extract "${read_cursor_lookup_explain}" index_name)"
cursor_index_rows="$(json_extract "${read_cursor_lookup_explain}" index_output_rows)"
cursor_index_elapsed_ns="$(json_extract "${read_cursor_lookup_explain}" index_elapsed_ns)"
cursor_filter_elapsed_ns="$(json_extract "${read_cursor_lookup_explain}" filter_elapsed_ns)"
cursor_sort_elapsed_ns="$(json_extract "${read_cursor_lookup_explain}" sort_elapsed_ns)"
cursor_loop_duration="$(json_extract "${read_cursor_lookup_loop}" loop_duration)"

utc_now="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cat > "${output_file}" <<EOF
# SurrealDB v3 — Chat Hot-Path Benchmark (Live Docker DB)

Date: ${utc_now}
Compose: \`${compose_file}\`
Endpoint: \`${endpoint_ws}\`
Server version (HTTP \`/version\`): \`${server_version}\`
Namespace/DB: \`${PROBE_NS}/${PROBE_DB}\`
Status note: validates chat message send/read query shapes used by \`POST /v1/chat/threads/:thread_id/messages/send\` and \`GET /v1/chat/threads/:thread_id/messages\`.

## Benchmark Intent

Evaluate chat fast-path read patterns:

\`\`\`sql
-- catch-up page after cursor
SELECT message_id, created_at
FROM chat_message
WHERE thread_id = \$thread_id
  AND (created_at > \$t OR (created_at = \$t AND message_id > \$message_id))
ORDER BY created_at ASC, message_id ASC
LIMIT 50

-- idempotent send lookup
SELECT message_id, request_id
FROM chat_message
WHERE thread_id = \$thread_id AND request_id = \$request_id
LIMIT 1

-- membership check
SELECT user_id, role
FROM chat_member
WHERE user_id = \$actor_id AND thread_id = \$thread_id AND left_at IS NONE
ORDER BY joined_at DESC
LIMIT 1

-- read cursor lookup
SELECT user_id, thread_id, message_id, read_at
FROM chat_read_cursor
WHERE user_id = \$actor_id AND thread_id = \$thread_id
LIMIT 1
\`\`\`

This probe uses structurally equivalent benchmark tables and indexes:
- \`probe_chat_message_bench\`: \`idx_probe_message_order(thread_id, created_at, message_id)\`, \`uniq_probe_message_request(thread_id, request_id)\`
- \`probe_chat_member_bench\`: \`idx_probe_member_lookup(user_id, thread_id)\`
- \`probe_chat_cursor_bench\`: \`idx_probe_read_cursor_lookup(user_id, thread_id)\`

Seed shape:
- total message rows: \`${message_count}\`
- hot thread rows: \`${hot_message_count}\`
- global noise rows requested: \`${NOISE_ROWS}\`
- catch-up cursor row: \`${cursor_created_at}\` / \`${cursor_message_id}\`

## Summary

| Scenario | IndexScan used | Index used | Index rows scanned | Index elapsed (ns) | Filter elapsed (ns) | SortTopK elapsed (ns) | Loop duration |
|---|---|---|---:|---:|---:|---:|---|
| Catch-up list | ${catchup_has_index} | ${catchup_index_name} | ${catchup_index_rows} | ${catchup_index_elapsed_ns} | ${catchup_filter_elapsed_ns} | ${catchup_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${catchup_loop_duration} |
| Idempotent send lookup | ${idempotent_has_index} | ${idempotent_index_name} | ${idempotent_index_rows} | ${idempotent_index_elapsed_ns} | ${idempotent_filter_elapsed_ns} | ${idempotent_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${idempotent_loop_duration} |
| Member lookup | ${member_has_index} | ${member_index_name} | ${member_index_rows} | ${member_index_elapsed_ns} | ${member_filter_elapsed_ns} | ${member_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${member_loop_duration} |
| Read cursor lookup | ${cursor_has_index} | ${cursor_index_name} | ${cursor_index_rows} | ${cursor_index_elapsed_ns} | ${cursor_filter_elapsed_ns} | ${cursor_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${cursor_loop_duration} |

## Decision

- Keep chat read/send path index-backed with \`idx_message_order\` + \`uniq_message_request\`.
- Keep explicit member and read-cursor lookup indexes (\`idx_member_lookup\`, \`idx_read_cursor_lookup\`) as baseline for chat authorization/read-state checks.
- Keep hot-thread catch-up window bounded (default/target \`limit=50\`) and benchmark again before raising catch-up limits.
- Keep graph/triple traversal out of chat request-path queries.

## Raw Explain (Catch-up)
\`\`\`json
${catchup_explain}
\`\`\`

## Raw Explain (Idempotent send lookup)
\`\`\`json
${idempotent_lookup_explain}
\`\`\`

## Raw Explain (Member lookup)
\`\`\`json
${member_lookup_explain}
\`\`\`

## Raw Explain (Read cursor lookup)
\`\`\`json
${read_cursor_lookup_explain}
\`\`\`
EOF

echo "Wrote ${output_file}"
