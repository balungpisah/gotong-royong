#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

compose_file="${COMPOSE_FILE:-compose.dev.yaml}"
output_file="${1:-${ROOT_DIR}/docs/research/surrealdb-notification-bench-latest.md}"

: "${SURREAL_PORT:=8000}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"
: "${PROBE_NS:=gotong_notification_probe}"
: "${PROBE_DB:=bench}"
: "${HOT_USER_ID:=u-hot}"
: "${HOT_UNREAD_ROWS:=10000}"
: "${HOT_READ_ROWS:=10000}"
: "${NOISE_ROWS:=80000}"
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

surreal_sql_root_json "REMOVE TABLE probe_notification_bench;" >/dev/null || true

surreal_sql_root_json $'DEFINE TABLE probe_notification_bench SCHEMAFULL;\nDEFINE FIELD notification_id ON TABLE probe_notification_bench TYPE string;\nDEFINE FIELD user_id ON TABLE probe_notification_bench TYPE string;\nDEFINE FIELD created_at ON TABLE probe_notification_bench TYPE datetime;\nDEFINE FIELD read_at ON TABLE probe_notification_bench TYPE option<datetime>;\nDEFINE INDEX uniq_probe_notification_id ON TABLE probe_notification_bench FIELDS notification_id UNIQUE;\nDEFINE INDEX idx_probe_notification_user ON TABLE probe_notification_bench FIELDS user_id, created_at, notification_id;\nDEFINE INDEX idx_probe_notification_unread ON TABLE probe_notification_bench FIELDS user_id, read_at, created_at, notification_id;' >/dev/null

surreal_sql_root_json "FOR \$i IN 0..${HOT_UNREAD_ROWS} { LET \$created_at = time::now() - duration::from_millis(<int>\$i); CREATE probe_notification_bench SET notification_id=string::concat('hu-', <string>\$i), user_id='${HOT_USER_ID}', created_at=\$created_at, read_at=NONE; };" >/dev/null

surreal_sql_root_json "FOR \$i IN 0..${HOT_READ_ROWS} { LET \$created_at = time::now() - duration::from_millis(<int>(\$i + ${HOT_UNREAD_ROWS})); LET \$read_at = \$created_at + duration::from_millis(1); CREATE probe_notification_bench SET notification_id=string::concat('hr-', <string>\$i), user_id='${HOT_USER_ID}', created_at=\$created_at, read_at=\$read_at; };" >/dev/null

if [[ "${NOISE_ROWS}" -gt 0 ]]; then
  surreal_sql_root_json "FOR \$i IN 0..${NOISE_ROWS} { LET \$created_at = time::now() - duration::from_millis(<int>(\$i + ${HOT_UNREAD_ROWS} + ${HOT_READ_ROWS})); LET \$noise_user = string::concat('noise-u-', <string>(<int>\$i % 10000)); IF (<int>\$i % 2 = 0) { CREATE probe_notification_bench SET notification_id=string::concat('nu-', <string>\$i), user_id=\$noise_user, created_at=\$created_at, read_at=NONE; } ELSE { LET \$read_at = \$created_at + duration::from_millis(1); CREATE probe_notification_bench SET notification_id=string::concat('nr-', <string>\$i), user_id=\$noise_user, created_at=\$created_at, read_at=\$read_at; }; };" >/dev/null
fi

row_count_json="$(surreal_sql_root_json "SELECT count() FROM probe_notification_bench GROUP ALL;")"
row_count="$(printf "%s" "${row_count_json}" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(data[0][0]["count"])')"

hot_unread_count_json="$(surreal_sql_root_json "SELECT count() FROM probe_notification_bench WHERE user_id='${HOT_USER_ID}' AND read_at IS NONE GROUP ALL;")"
hot_unread_count="$(printf "%s" "${hot_unread_count_json}" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(data[0][0]["count"])')"

hot_total_count_json="$(surreal_sql_root_json "SELECT count() FROM probe_notification_bench WHERE user_id='${HOT_USER_ID}' GROUP ALL;")"
hot_total_count="$(printf "%s" "${hot_total_count_json}" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(data[0][0]["count"])')"

unread_list_explain="$(surreal_sql_root_json "SELECT notification_id, created_at FROM probe_notification_bench WHERE user_id='${HOT_USER_ID}' AND read_at IS NONE ORDER BY created_at DESC, notification_id DESC LIMIT 20 EXPLAIN FULL;")"

all_list_explain="$(surreal_sql_root_json "SELECT notification_id, created_at FROM probe_notification_bench WHERE user_id='${HOT_USER_ID}' ORDER BY created_at DESC, notification_id DESC LIMIT 20 EXPLAIN FULL;")"

unread_count_explain="$(surreal_sql_root_json "SELECT count() AS unread_count FROM probe_notification_bench WHERE user_id='${HOT_USER_ID}' AND read_at IS NONE EXPLAIN FULL;")"

unread_list_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT notification_id, created_at FROM probe_notification_bench WHERE user_id='${HOT_USER_ID}' AND read_at IS NONE ORDER BY created_at DESC, notification_id DESC LIMIT 20; }; RETURN time::now() - \$start;")"

all_list_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT notification_id, created_at FROM probe_notification_bench WHERE user_id='${HOT_USER_ID}' ORDER BY created_at DESC, notification_id DESC LIMIT 20; }; RETURN time::now() - \$start;")"

unread_count_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT count() AS unread_count FROM probe_notification_bench WHERE user_id='${HOT_USER_ID}' AND read_at IS NONE; }; RETURN time::now() - \$start;")"

unread_list_has_index="$(json_extract "${unread_list_explain}" has_index_scan)"
unread_list_index_name="$(json_extract "${unread_list_explain}" index_name)"
unread_list_index_rows="$(json_extract "${unread_list_explain}" index_output_rows)"
unread_list_index_elapsed_ns="$(json_extract "${unread_list_explain}" index_elapsed_ns)"
unread_list_filter_elapsed_ns="$(json_extract "${unread_list_explain}" filter_elapsed_ns)"
unread_list_sort_elapsed_ns="$(json_extract "${unread_list_explain}" sort_elapsed_ns)"
unread_list_loop_duration="$(json_extract "${unread_list_loop}" loop_duration)"

all_list_has_index="$(json_extract "${all_list_explain}" has_index_scan)"
all_list_index_name="$(json_extract "${all_list_explain}" index_name)"
all_list_index_rows="$(json_extract "${all_list_explain}" index_output_rows)"
all_list_index_elapsed_ns="$(json_extract "${all_list_explain}" index_elapsed_ns)"
all_list_filter_elapsed_ns="$(json_extract "${all_list_explain}" filter_elapsed_ns)"
all_list_sort_elapsed_ns="$(json_extract "${all_list_explain}" sort_elapsed_ns)"
all_list_loop_duration="$(json_extract "${all_list_loop}" loop_duration)"

unread_count_has_index="$(json_extract "${unread_count_explain}" has_index_scan)"
unread_count_index_name="$(json_extract "${unread_count_explain}" index_name)"
unread_count_index_rows="$(json_extract "${unread_count_explain}" index_output_rows)"
unread_count_index_elapsed_ns="$(json_extract "${unread_count_explain}" index_elapsed_ns)"
unread_count_filter_elapsed_ns="$(json_extract "${unread_count_explain}" filter_elapsed_ns)"
unread_count_sort_elapsed_ns="$(json_extract "${unread_count_explain}" sort_elapsed_ns)"
unread_count_loop_duration="$(json_extract "${unread_count_loop}" loop_duration)"

utc_now="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cat > "${output_file}" <<EOF
# SurrealDB v3 — Notification Hot-Path Benchmark (Live Docker DB)

Date: ${utc_now}
Compose: \`${compose_file}\`
Endpoint: \`${endpoint_ws}\`
Server version (HTTP \`/version\`): \`${server_version}\`
Namespace/DB: \`${PROBE_NS}/${PROBE_DB}\`
Status note: validates list/unread-count query shapes used by \`GET /v1/notifications\` and \`GET /v1/notifications/unread-count\`.

## Benchmark Intent

Evaluate hot-path notification reads:

\`\`\`sql
-- default list (include_read=false)
SELECT notification_id, created_at
FROM discovery_notification
WHERE user_id = \$actor_id AND read_at IS NONE
ORDER BY created_at DESC, notification_id DESC
LIMIT 20

-- include_read=true list
SELECT notification_id, created_at
FROM discovery_notification
WHERE user_id = \$actor_id
ORDER BY created_at DESC, notification_id DESC
LIMIT 20

-- unread count
SELECT count() AS unread_count
FROM discovery_notification
WHERE user_id = \$actor_id AND read_at IS NONE
\`\`\`

This probe uses a structurally equivalent table and indexes:
- Table: \`probe_notification_bench\`
- Indexes:
  - \`idx_probe_notification_user(user_id, created_at, notification_id)\`
  - \`idx_probe_notification_unread(user_id, read_at, created_at, notification_id)\`
- Seed shape:
  - total rows: \`${row_count}\`
  - hot user rows: \`${hot_total_count}\`
  - hot user unread rows: \`${hot_unread_count}\`
  - extra global noise rows requested: \`${NOISE_ROWS}\`

## Summary

| Scenario | IndexScan used | Index used | Index rows scanned | Index elapsed (ns) | Filter elapsed (ns) | SortTopK elapsed (ns) | Loop duration |
|---|---|---|---:|---:|---:|---:|---|
| Unread list (include_read=false) | ${unread_list_has_index} | ${unread_list_index_name} | ${unread_list_index_rows} | ${unread_list_index_elapsed_ns} | ${unread_list_filter_elapsed_ns} | ${unread_list_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${unread_list_loop_duration} |
| Full list (include_read=true) | ${all_list_has_index} | ${all_list_index_name} | ${all_list_index_rows} | ${all_list_index_elapsed_ns} | ${all_list_filter_elapsed_ns} | ${all_list_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${all_list_loop_duration} |
| Unread count | ${unread_count_has_index} | ${unread_count_index_name} | ${unread_count_index_rows} | ${unread_count_index_elapsed_ns} | ${unread_count_filter_elapsed_ns} | ${unread_count_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${unread_count_loop_duration} |

## Decision

- Keep unread-list path anchored to \`(user_id, read_at, created_at, notification_id)\` index.
- Keep include-read list path anchored to \`(user_id, created_at, notification_id)\` index.
- Keep unread-count shape as simple index-backed aggregate; no graph/triple traversal involvement.
- Re-run this benchmark when notification fanout semantics change (new notification types, digest batching, or read-state model updates).

## Raw Explain (Unread list)
\`\`\`json
${unread_list_explain}
\`\`\`

## Raw Explain (Full list)
\`\`\`json
${all_list_explain}
\`\`\`

## Raw Explain (Unread count)
\`\`\`json
${unread_count_explain}
\`\`\`
EOF

echo "Wrote ${output_file}"
