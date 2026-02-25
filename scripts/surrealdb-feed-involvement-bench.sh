#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

compose_file="${COMPOSE_FILE:-compose.dev.yaml}"
output_file="${1:-${ROOT_DIR}/docs/research/surrealdb-feed-involvement-bench-latest.md}"

: "${SURREAL_PORT:=8000}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"
: "${PROBE_NS:=gotong_feed_involvement_probe}"
: "${PROBE_DB:=bench}"
: "${PARTICIPANT_ROWS:=10000}"
: "${ACTOR_ROWS:=10000}"
: "${BENCH_LOOPS:=120}"
: "${HOT_ACTOR_ID:=u-hot}"

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

surreal_sql_root_json "REMOVE TABLE probe_feed_involvement_bench;" >/dev/null || true

surreal_sql_root_json $'DEFINE TABLE probe_feed_involvement_bench SCHEMAFULL;\nDEFINE FIELD actor_id ON TABLE probe_feed_involvement_bench TYPE string;\nDEFINE FIELD participant_ids ON TABLE probe_feed_involvement_bench TYPE array;\nDEFINE FIELD occurred_at ON TABLE probe_feed_involvement_bench TYPE datetime;\nDEFINE FIELD feed_id ON TABLE probe_feed_involvement_bench TYPE string;\nDEFINE INDEX idx_probe_actor_latest ON TABLE probe_feed_involvement_bench FIELDS actor_id, occurred_at, feed_id;\nDEFINE INDEX idx_probe_time ON TABLE probe_feed_involvement_bench FIELDS occurred_at, feed_id;' >/dev/null

surreal_sql_root_json "FOR \$i IN 0..${PARTICIPANT_ROWS} { CREATE probe_feed_involvement_bench SET actor_id=string::concat('actor-', <string>(<int>\$i % 1000)), participant_ids=['${HOT_ACTOR_ID}', string::concat('u-', <string>(<int>\$i % 2000))], occurred_at=time::now() - duration::from_millis(<int>\$i), feed_id=string::concat('p-', <string>\$i); };" >/dev/null
surreal_sql_root_json "FOR \$i IN 0..${ACTOR_ROWS} { CREATE probe_feed_involvement_bench SET actor_id='${HOT_ACTOR_ID}', participant_ids=[string::concat('u-', <string>(<int>\$i % 2000))], occurred_at=time::now() - duration::from_millis(<int>\$i), feed_id=string::concat('a-', <string>\$i); };" >/dev/null

row_count_json="$(surreal_sql_root_json "SELECT count() FROM probe_feed_involvement_bench GROUP ALL;")"
row_count="$(printf "%s" "${row_count_json}" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(data[0][0]["count"])')"

actor_explain="$(surreal_sql_root_json "SELECT feed_id, occurred_at FROM probe_feed_involvement_bench WHERE actor_id='${HOT_ACTOR_ID}' ORDER BY occurred_at DESC, feed_id DESC LIMIT 20 EXPLAIN FULL;")"
participant_explain="$(surreal_sql_root_json "SELECT feed_id, occurred_at FROM probe_feed_involvement_bench WHERE '${HOT_ACTOR_ID}' IN participant_ids ORDER BY occurred_at DESC, feed_id DESC LIMIT 20 EXPLAIN FULL;")"
combined_explain="$(surreal_sql_root_json "SELECT feed_id, occurred_at FROM probe_feed_involvement_bench WHERE (actor_id='${HOT_ACTOR_ID}' OR '${HOT_ACTOR_ID}' IN participant_ids) ORDER BY occurred_at DESC, feed_id DESC LIMIT 20 EXPLAIN FULL;")"

actor_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT feed_id, occurred_at FROM probe_feed_involvement_bench WHERE actor_id='${HOT_ACTOR_ID}' ORDER BY occurred_at DESC, feed_id DESC LIMIT 20; }; RETURN time::now() - \$start;")"
participant_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT feed_id, occurred_at FROM probe_feed_involvement_bench WHERE '${HOT_ACTOR_ID}' IN participant_ids ORDER BY occurred_at DESC, feed_id DESC LIMIT 20; }; RETURN time::now() - \$start;")"
combined_loop="$(surreal_sql_root_json "LET \$start = time::now(); FOR \$i IN 0..${BENCH_LOOPS} { SELECT feed_id, occurred_at FROM probe_feed_involvement_bench WHERE (actor_id='${HOT_ACTOR_ID}' OR '${HOT_ACTOR_ID}' IN participant_ids) ORDER BY occurred_at DESC, feed_id DESC LIMIT 20; }; RETURN time::now() - \$start;")"

actor_has_index="$(json_extract "${actor_explain}" has_index_scan)"
actor_index_name="$(json_extract "${actor_explain}" index_name)"
actor_index_rows="$(json_extract "${actor_explain}" index_output_rows)"
actor_index_elapsed_ns="$(json_extract "${actor_explain}" index_elapsed_ns)"
actor_filter_elapsed_ns="$(json_extract "${actor_explain}" filter_elapsed_ns)"
actor_sort_elapsed_ns="$(json_extract "${actor_explain}" sort_elapsed_ns)"
actor_loop_duration="$(json_extract "${actor_loop}" loop_duration)"

participant_has_index="$(json_extract "${participant_explain}" has_index_scan)"
participant_index_name="$(json_extract "${participant_explain}" index_name)"
participant_index_rows="$(json_extract "${participant_explain}" index_output_rows)"
participant_index_elapsed_ns="$(json_extract "${participant_explain}" index_elapsed_ns)"
participant_filter_elapsed_ns="$(json_extract "${participant_explain}" filter_elapsed_ns)"
participant_sort_elapsed_ns="$(json_extract "${participant_explain}" sort_elapsed_ns)"
participant_loop_duration="$(json_extract "${participant_loop}" loop_duration)"

combined_has_index="$(json_extract "${combined_explain}" has_index_scan)"
combined_index_name="$(json_extract "${combined_explain}" index_name)"
combined_index_rows="$(json_extract "${combined_explain}" index_output_rows)"
combined_index_elapsed_ns="$(json_extract "${combined_explain}" index_elapsed_ns)"
combined_filter_elapsed_ns="$(json_extract "${combined_explain}" filter_elapsed_ns)"
combined_sort_elapsed_ns="$(json_extract "${combined_explain}" sort_elapsed_ns)"
combined_loop_duration="$(json_extract "${combined_loop}" loop_duration)"

utc_now="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cat > "${output_file}" <<EOF
# SurrealDB v3 — Feed Involvement-Only Benchmark (Live Docker DB)

Date: ${utc_now}
Compose: \`${compose_file}\`
Endpoint: \`${endpoint_ws}\`
Server version (HTTP \`/version\`): \`${server_version}\`
Namespace/DB: \`${PROBE_NS}/${PROBE_DB}\`

## Benchmark Intent

Evaluate the hot-path feed query shape behind \`involvement_only=true\`:

\`\`\`sql
SELECT feed_id, occurred_at
FROM discovery_feed_item
WHERE actor_id = \$actor_id OR \$actor_id IN participant_ids
ORDER BY occurred_at DESC, feed_id DESC
LIMIT 20
\`\`\`

This probe uses a structurally equivalent table and index setup:
- Table: \`probe_feed_involvement_bench\`
- Indexes: \`idx_probe_actor_latest(actor_id, occurred_at, feed_id)\`, \`idx_probe_time(occurred_at, feed_id)\`
- Seed shape: \`${row_count}\` rows total (participant-heavy + actor-heavy mix)

## Summary

| Scenario | IndexScan used | Index used | Index rows scanned | Index elapsed (ns) | Filter elapsed (ns) | SortTopK elapsed (ns) | Loop duration |
|---|---|---|---:|---:|---:|---:|---|
| Actor-only (\`actor_id = \$actor_id\`) | ${actor_has_index} | ${actor_index_name} | ${actor_index_rows} | ${actor_index_elapsed_ns} | ${actor_filter_elapsed_ns} | ${actor_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${actor_loop_duration} |
| Participant-only (\`\$actor_id IN participant_ids\`) | ${participant_has_index} | ${participant_index_name} | ${participant_index_rows} | ${participant_index_elapsed_ns} | ${participant_filter_elapsed_ns} | ${participant_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${participant_loop_duration} |
| Combined OR path | ${combined_has_index} | ${combined_index_name} | ${combined_index_rows} | ${combined_index_elapsed_ns} | ${combined_filter_elapsed_ns} | ${combined_sort_elapsed_ns} | ${BENCH_LOOPS} loops: ${combined_loop_duration} |

## Decision

- Keep current read path for now, but treat \`involvement_only\` as a monitored lane.
- If OR/participant path becomes a top traffic shape and breaches feed SLO budget, move to Pack C materialization (\`feed_participant_edge\`) so reads avoid array-membership filtering on the hot path.
- Keep triples/relations as enrichment/audit only; do not introduce graph traversal for feed listing.

## Raw Explain (Actor-only)
\`\`\`json
${actor_explain}
\`\`\`

## Raw Explain (Participant-only)
\`\`\`json
${participant_explain}
\`\`\`

## Raw Explain (Combined OR path)
\`\`\`json
${combined_explain}
\`\`\`
EOF

echo "Wrote ${output_file}"
