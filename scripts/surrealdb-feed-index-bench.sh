#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

compose_file="${COMPOSE_FILE:-compose.dev.yaml}"
output_file="${1:-${ROOT_DIR}/docs/research/surrealdb-feed-index-bench-latest.md}"

: "${SURREAL_PORT:=8000}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"
: "${PROBE_NS:=gotong_feed_index_probe}"
: "${PROBE_DB:=bench}"
: "${HOT_ROWS:=10000}"
: "${COLD_ROWS:=10000}"
: "${BENCH_LOOPS:=120}"
: "${NO_INDEX_LOOPS:=30}"

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
elif mode == "index_elapsed_ns":
    op = find_operator(root, "IndexScan")
    print(op.get("metrics", {}).get("elapsed_ns", "n/a") if op else "n/a")
elif mode == "index_output_rows":
    op = find_operator(root, "IndexScan")
    print(op.get("metrics", {}).get("output_rows", "n/a") if op else "n/a")
elif mode == "sort_elapsed_ns":
    op = find_operator(root, "SortTopKByKey")
    print(op.get("metrics", {}).get("elapsed_ns", "n/a") if op else "n/a")
elif mode == "index_direction":
    op = find_operator(root, "IndexScan")
    print(op.get("attributes", {}).get("direction", "n/a") if op else "n/a")
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

surreal_sql_root_json "REMOVE TABLE probe_feed_index_bench;" >/dev/null || true

surreal_sql_root_json $'DEFINE TABLE probe_feed_index_bench SCHEMAFULL;\nDEFINE FIELD source_type ON TABLE probe_feed_index_bench TYPE string;\nDEFINE FIELD source_id ON TABLE probe_feed_index_bench TYPE string;\nDEFINE FIELD occurred_at ON TABLE probe_feed_index_bench TYPE datetime;\nDEFINE FIELD feed_id ON TABLE probe_feed_index_bench TYPE string;\nDEFINE INDEX idx_probe_feed_source_latest ON TABLE probe_feed_index_bench FIELDS source_type, source_id, occurred_at, feed_id;' >/dev/null

surreal_sql_root_json "FOR \$i IN 0..${HOT_ROWS} { CREATE probe_feed_index_bench SET source_type='ontology_note', source_id='hot-note', occurred_at=time::now() - duration::from_millis(<int>\$i), feed_id=string::concat('hot-', <string>\$i); };" >/dev/null
surreal_sql_root_json "FOR \$i IN 0..${COLD_ROWS} { CREATE probe_feed_index_bench SET source_type='ontology_note', source_id=string::concat('cold-', <string>\$i), occurred_at=time::now() - duration::from_millis(<int>\$i), feed_id=string::concat('cold-', <string>\$i); };" >/dev/null

row_count_json="$(surreal_sql_root_json "SELECT count() FROM probe_feed_index_bench GROUP ALL;")"
row_count="$(printf "%s" "${row_count_json}" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(data[0][0]["count"])')"

desc_explain_with_index="$(surreal_sql_root_json "SELECT feed_id, occurred_at FROM probe_feed_index_bench WHERE source_type='ontology_note' AND source_id='hot-note' ORDER BY occurred_at DESC, feed_id DESC LIMIT 1 EXPLAIN FULL;")"
asc_explain_with_index="$(surreal_sql_root_json "SELECT feed_id, occurred_at FROM probe_feed_index_bench WHERE source_type='ontology_note' AND source_id='hot-note' ORDER BY occurred_at ASC, feed_id ASC LIMIT 1 EXPLAIN FULL;")"

desc_loop_with_index="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT feed_id, occurred_at FROM probe_feed_index_bench WHERE source_type='ontology_note' AND source_id='hot-note' ORDER BY occurred_at DESC, feed_id DESC LIMIT 1; }; RETURN time::now() - \$start;")"
asc_loop_with_index="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT feed_id, occurred_at FROM probe_feed_index_bench WHERE source_type='ontology_note' AND source_id='hot-note' ORDER BY occurred_at ASC, feed_id ASC LIMIT 1; }; RETURN time::now() - \$start;")"

surreal_sql_root_json "REMOVE INDEX idx_probe_feed_source_latest ON TABLE probe_feed_index_bench;" >/dev/null

desc_explain_no_index="$(surreal_sql_root_json "SELECT feed_id, occurred_at FROM probe_feed_index_bench WHERE source_type='ontology_note' AND source_id='hot-note' ORDER BY occurred_at DESC, feed_id DESC LIMIT 1 EXPLAIN FULL;")"
desc_loop_no_index="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${NO_INDEX_LOOPS} { SELECT feed_id, occurred_at FROM probe_feed_index_bench WHERE source_type='ontology_note' AND source_id='hot-note' ORDER BY occurred_at DESC, feed_id DESC LIMIT 1; }; RETURN time::now() - \$start;")"

desc_has_index="$(json_extract "${desc_explain_with_index}" has_index_scan)"
desc_index_elapsed_ns="$(json_extract "${desc_explain_with_index}" index_elapsed_ns)"
desc_index_rows="$(json_extract "${desc_explain_with_index}" index_output_rows)"
desc_sort_elapsed_ns="$(json_extract "${desc_explain_with_index}" sort_elapsed_ns)"
desc_index_direction="$(json_extract "${desc_explain_with_index}" index_direction)"

asc_has_index="$(json_extract "${asc_explain_with_index}" has_index_scan)"
asc_index_elapsed_ns="$(json_extract "${asc_explain_with_index}" index_elapsed_ns)"
asc_index_rows="$(json_extract "${asc_explain_with_index}" index_output_rows)"
asc_sort_elapsed_ns="$(json_extract "${asc_explain_with_index}" sort_elapsed_ns)"
asc_index_direction="$(json_extract "${asc_explain_with_index}" index_direction)"

no_index_has_index="$(json_extract "${desc_explain_no_index}" has_index_scan)"

desc_loop_with_index_duration="$(json_extract "${desc_loop_with_index}" loop_duration)"
asc_loop_with_index_duration="$(json_extract "${asc_loop_with_index}" loop_duration)"
desc_loop_no_index_duration="$(json_extract "${desc_loop_no_index}" loop_duration)"

utc_now="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cat > "${output_file}" <<EOF
# SurrealDB v3 — Feed Source Lookup Benchmark (Live Docker DB)

Date: ${utc_now}
Compose: \`${compose_file}\`
Endpoint: \`${endpoint_ws}\`
Server version (HTTP \`/version\`): \`${server_version}\`
Namespace/DB: \`${PROBE_NS}/${PROBE_DB}\`

## Benchmark Intent

Evaluate the hot-path lookup used by \`get_latest_by_source\`:

\`\`\`sql
SELECT feed_id, occurred_at
FROM discovery_feed_item
WHERE source_type = \$source_type AND source_id = \$source_id
ORDER BY occurred_at DESC, feed_id DESC
LIMIT 1
\`\`\`

This probe uses a structurally equivalent table and index:
- Table: \`probe_feed_index_bench\`
- Index: \`(source_type, source_id, occurred_at, feed_id)\`
- Seed shape: \`${row_count}\` rows total, \`${HOT_ROWS}\` hot-source rows + \`${COLD_ROWS}\` distributed cold-source rows

## Summary

| Scenario | IndexScan used | Index direction | Index rows scanned | Index elapsed (ns) | SortTopK elapsed (ns) | Loop duration |
|---|---|---|---:|---:|---:|---|
| DESC with index | ${desc_has_index} | ${desc_index_direction} | ${desc_index_rows} | ${desc_index_elapsed_ns} | ${desc_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${desc_loop_with_index_duration} |
| ASC with index | ${asc_has_index} | ${asc_index_direction} | ${asc_index_rows} | ${asc_index_elapsed_ns} | ${asc_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${asc_loop_with_index_duration} |
| DESC without index | ${no_index_has_index} | n/a | n/a | n/a | n/a | ${NO_INDEX_LOOPS} loops: ${desc_loop_no_index_duration} |

## Decision

- Keep the current index shape (\`source_type, source_id, occurred_at, feed_id\`).
- No DESC-specific index action now. In SurrealDB v3.0.0, this index is used for both ASC and DESC query shapes and planner reports the same \`IndexScan ... direction: Forward\` plus \`SortTopKByKey\`.
- Revisit only if production latency shows regression under higher per-source fan-out than this probe.

## Raw Explain (DESC with index)
\`\`\`json
${desc_explain_with_index}
\`\`\`

## Raw Explain (ASC with index)
\`\`\`json
${asc_explain_with_index}
\`\`\`

## Raw Explain (DESC without index)
\`\`\`json
${desc_explain_no_index}
\`\`\`
EOF

echo "Wrote ${output_file}"
