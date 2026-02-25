#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

compose_file="${COMPOSE_FILE:-compose.dev.yaml}"
output_file="${1:-${ROOT_DIR}/docs/research/surrealdb-live-db-probe-latest.md}"

: "${SURREAL_PORT:=8000}"
: "${SURREAL_USER:=root}"
: "${SURREAL_PASS:=root}"
: "${PROBE_NS:=gotong_probe}"
: "${PROBE_DB:=bench}"

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

surreal_sql_root() {
  local sql="$1"
  docker compose -f "${compose_file}" exec -T surrealdb /surreal sql \
    --endpoint "${endpoint_ws}" \
    --user "${SURREAL_USER}" \
    --pass "${SURREAL_PASS}" \
    --ns "${PROBE_NS}" \
    --db "${PROBE_DB}" \
    --hide-welcome <<SQL
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

http_signup() {
  local email="$1"
  local pass="$2"
  local access_name="$3"
  curl -fsS "${endpoint_http}/signup" \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    -d "{\"ns\":\"${PROBE_NS}\",\"db\":\"${PROBE_DB}\",\"ac\":\"${access_name}\",\"email\":\"${email}\",\"pass\":\"${pass}\"}" |
    python3 -c 'import sys,json; print(json.load(sys.stdin)["token"])'
}

http_signin() {
  local email="$1"
  local pass="$2"
  local access_name="$3"
  curl -fsS "${endpoint_http}/signin" \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    -d "{\"ns\":\"${PROBE_NS}\",\"db\":\"${PROBE_DB}\",\"ac\":\"${access_name}\",\"email\":\"${email}\",\"pass\":\"${pass}\"}" |
    python3 -c 'import sys,json; print(json.load(sys.stdin)["token"])'
}

live_ws_capture() {
  local live_sql="$1"
  local followup_sql="$2"
  local auth_mode="$3" # root | token
  local token="${4:-}"

  local tmp_dir
  tmp_dir="$(mktemp -d)"
  local fifo="${tmp_dir}/live.fifo"
  local out="${tmp_dir}/live.out"
  mkfifo "${fifo}"

  if [[ "${auth_mode}" == "root" ]]; then
    (docker compose -f "${compose_file}" exec -T surrealdb /surreal sql \
      --endpoint "${endpoint_ws}" \
      --user "${SURREAL_USER}" \
      --pass "${SURREAL_PASS}" \
      --ns "${PROBE_NS}" \
      --db "${PROBE_DB}" \
      --hide-welcome <"${fifo}" >"${out}" 2>&1 || true) &
  else
    (docker compose -f "${compose_file}" exec -T surrealdb /surreal sql \
      --endpoint "${endpoint_ws}" \
      --token "${token}" \
      --ns "${PROBE_NS}" \
      --db "${PROBE_DB}" \
      --hide-welcome <"${fifo}" >"${out}" 2>&1 || true) &
  fi

  local pid=$!
  exec 3>"${fifo}"
  echo "${live_sql}" >&3
  sleep 0.8

  if [[ -n "${followup_sql}" ]]; then
    surreal_sql_root_json "${followup_sql}" >/dev/null
  fi

  sleep 1.2
  exec 3>&-
  wait "${pid}" || true

  cat "${out}"
  rm -rf "${tmp_dir}"
}

ensure_dev_db_up

server_version="$(curl -fsS "${endpoint_http}/version" 2>/dev/null || true)"
if [[ -z "${server_version}" ]]; then
  server_version="(unknown; /version failed)"
fi

# Clean prior probe artifacts (idempotent).
surreal_sql_root_json "REMOVE TABLE probe_idem;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_ts;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_live;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_diff;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_user;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_private;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_graph_user;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_graph_entity;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_follows;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_search;" >/dev/null || true
surreal_sql_root_json "REMOVE TABLE probe_vector;" >/dev/null || true
surreal_sql_root_json "REMOVE ACCESS probe_account ON DATABASE;" >/dev/null || true
surreal_sql_root_json "REMOVE ANALYZER probe_simple;" >/dev/null || true

## Pattern 1: Unique composite index (idempotency)
surreal_sql_root_json $'DEFINE TABLE probe_idem SCHEMAFULL;\nDEFINE FIELD entity_id ON TABLE probe_idem TYPE string;\nDEFINE FIELD request_id ON TABLE probe_idem TYPE string;\nDEFINE INDEX uniq_entity_request ON TABLE probe_idem FIELDS entity_id, request_id UNIQUE;' >/dev/null
idem_first="$(surreal_sql_root_json "CREATE probe_idem CONTENT { entity_id: 'thread:alpha', request_id: 'req-123' };")"
idem_dup="$(surreal_sql_root_json "CREATE probe_idem CONTENT { entity_id: 'thread:alpha', request_id: 'req-123' };")"
idem_rows="$(surreal_sql_root_json "SELECT count() FROM probe_idem WHERE entity_id='thread:alpha' AND request_id='req-123' GROUP ALL;")"

idem_pass="FAIL"
if echo "${idem_dup}" | grep -q "already contains" && echo "${idem_rows}" | grep -q "\"count\":1"; then
  idem_pass="PASS"
fi

## Pattern 2: Deterministic ordering + catch-up query
surreal_sql_root_json $'DEFINE TABLE probe_ts SCHEMAFULL;\nDEFINE FIELD thread_id ON TABLE probe_ts TYPE string;\nDEFINE FIELD message_id ON TABLE probe_ts TYPE string;\nDEFINE FIELD created_at ON TABLE probe_ts TYPE datetime;\nDEFINE INDEX idx_probe_ts_order ON TABLE probe_ts FIELDS thread_id, created_at, message_id;' >/dev/null
surreal_sql_root_json "CREATE probe_ts CONTENT { thread_id: 'thread:order', message_id: 'msg-003', created_at: d'2026-02-15T03:30:00Z' };" >/dev/null
surreal_sql_root_json "CREATE probe_ts CONTENT { thread_id: 'thread:order', message_id: 'msg-001', created_at: d'2026-02-15T03:30:00Z' };" >/dev/null
surreal_sql_root_json "CREATE probe_ts CONTENT { thread_id: 'thread:order', message_id: 'msg-002', created_at: d'2026-02-15T03:30:00Z' };" >/dev/null

ordering_json="$(surreal_sql_root_json "SELECT message_id, created_at FROM probe_ts WHERE thread_id='thread:order' ORDER BY created_at ASC, message_id ASC;")"
catchup_json="$(surreal_sql_root_json "SELECT message_id, created_at FROM probe_ts WHERE thread_id='thread:order' AND (created_at > d'2026-02-15T03:30:00Z' OR (created_at = d'2026-02-15T03:30:00Z' AND message_id > 'msg-001')) ORDER BY created_at ASC, message_id ASC;")"
ordering_ids="$(printf "%s" "${ordering_json}" | python3 -c "import json,sys; rows=json.load(sys.stdin)[0]; print(','.join([r['message_id'] for r in rows]))")"
catchup_ids="$(printf "%s" "${catchup_json}" | python3 -c "import json,sys; rows=json.load(sys.stdin)[0]; print(','.join([r['message_id'] for r in rows]))")"

ordering_pass="FAIL"
if [[ "${ordering_ids}" == "msg-001,msg-002,msg-003" && "${catchup_ids}" == "msg-002,msg-003" ]]; then
  ordering_pass="PASS"
fi

## Pattern 3: EXPLAIN uses index scan for hot query shape
explain_plan="$(surreal_sql_root_json "EXPLAIN SELECT message_id, created_at FROM probe_ts WHERE thread_id='thread:order' ORDER BY created_at ASC, message_id ASC LIMIT 20;")"
explain_pass="FAIL"
if echo "${explain_plan}" | grep -q "IndexScan"; then
  explain_pass="PASS"
fi

# Observation: in SurrealQL, ORDER BY idioms must appear in the projection list.
order_idiom_error="$(
  docker compose -f "${compose_file}" exec -T surrealdb /surreal sql \
    --endpoint "${endpoint_ws}" \
    --user "${SURREAL_USER}" \
    --pass "${SURREAL_PASS}" \
    --ns "${PROBE_NS}" \
    --db "${PROBE_DB}" \
    --json \
    --hide-welcome <<<"SELECT message_id FROM probe_ts WHERE thread_id='thread:order' ORDER BY created_at ASC, message_id ASC;" 2>&1 || true
)"
order_idiom_error="$(printf "%s" "${order_idiom_error}" | sed '/^onnxruntime cpuid_info warning:/d' | head -n 6)"

## Pattern 4: Live stream works over WS
surreal_sql_root_json $'DEFINE TABLE probe_live SCHEMAFULL;\nDEFINE FIELD thread_id ON TABLE probe_live TYPE string;\nDEFINE FIELD body ON TABLE probe_live TYPE string;\nDEFINE FIELD created_at ON TABLE probe_live TYPE datetime;' >/dev/null
live_ws_output="$(live_ws_capture "LIVE SELECT * FROM probe_live WHERE thread_id='t-live';" "CREATE probe_live SET thread_id='t-live', body='hello-live', created_at=time::now();" root)"
live_ws_pass="FAIL"
if echo "${live_ws_output}" | grep -q "action: 'CREATE'"; then
  live_ws_pass="PASS"
fi

## Pattern 5: Live query does not stream over HTTP (expected)
live_http_output="$(printf "LIVE SELECT * FROM probe_live WHERE thread_id='t-live';\n" | docker compose -f "${compose_file}" exec -T surrealdb /surreal sql --endpoint "${endpoint_http}" --user "${SURREAL_USER}" --pass "${SURREAL_PASS}" --ns "${PROBE_NS}" --db "${PROBE_DB}" --hide-welcome 2>&1 || true)"
live_http_pass="FAIL"
if echo "${live_http_output}" | grep -q "Unable to perform the realtime query"; then
  live_http_pass="PASS"
fi

## Pattern 6: Live DIFF payload contract
surreal_sql_root_json $'DEFINE TABLE probe_diff SCHEMAFULL;\nDEFINE FIELD thread_id ON TABLE probe_diff TYPE string;\nDEFINE FIELD body ON TABLE probe_diff TYPE string;' >/dev/null
tmp_dir_diff="$(mktemp -d)"
fifo_diff="${tmp_dir_diff}/diff.fifo"
out_diff="${tmp_dir_diff}/diff.out"
mkfifo "${fifo_diff}"
(docker compose -f "${compose_file}" exec -T surrealdb /surreal sql \
  --endpoint "${endpoint_ws}" \
  --user "${SURREAL_USER}" \
  --pass "${SURREAL_PASS}" \
  --ns "${PROBE_NS}" \
  --db "${PROBE_DB}" \
  --hide-welcome <"${fifo_diff}" >"${out_diff}" 2>&1 || true) &
diff_pid=$!
exec 3>"${fifo_diff}"
echo "LIVE SELECT DIFF FROM probe_diff WHERE thread_id='t-diff';" >&3
sleep 0.8
surreal_sql_root_json "CREATE probe_diff SET thread_id='t-diff', body='hello';" >/dev/null
surreal_sql_root_json "UPDATE probe_diff SET body='hello edited' WHERE thread_id='t-diff' AND body='hello';" >/dev/null
sleep 1.2
exec 3>&-
wait "${diff_pid}" || true
live_diff_output="$(cat "${out_diff}")"
rm -rf "${tmp_dir_diff}"
live_diff_pass="FAIL"
if echo "${live_diff_output}" | grep -q "op: 'change'" && echo "${live_diff_output}" | grep -q "/body"; then
  live_diff_pass="PASS"
fi

## Pattern 7: Permission-filtered LIVE SELECT works (record access)
surreal_sql_root_json $'DEFINE TABLE probe_user SCHEMAFULL;\nDEFINE FIELD email ON TABLE probe_user TYPE string;\nDEFINE FIELD pass ON TABLE probe_user TYPE string;\nDEFINE ACCESS probe_account ON DATABASE TYPE RECORD SIGNUP ( CREATE probe_user SET email = $email, pass = $pass ) SIGNIN ( SELECT * FROM probe_user WHERE email = $email AND pass = $pass );\nDEFINE TABLE probe_private SCHEMAFULL PERMISSIONS FOR select WHERE owner = $auth.id, FOR create WHERE owner = $auth.id, FOR update WHERE owner = $auth.id, FOR delete WHERE owner = $auth.id;\nDEFINE FIELD owner ON TABLE probe_private TYPE record<probe_user>;\nDEFINE FIELD body ON TABLE probe_private TYPE string;' >/dev/null

alice_email="alice-probe-$(date +%s)@example.com"
bob_email="bob-probe-$(date +%s)@example.com"
alice_token="$(http_signup "${alice_email}" "secret123" "probe_account")"
bob_token="$(http_signup "${bob_email}" "secret123" "probe_account")"

tmp_dir_perm="$(mktemp -d)"
fifo_perm="${tmp_dir_perm}/perm.fifo"
out_perm="${tmp_dir_perm}/perm.out"
mkfifo "${fifo_perm}"
(docker compose -f "${compose_file}" exec -T surrealdb /surreal sql \
  --endpoint "${endpoint_ws}" \
  --token "${alice_token}" \
  --ns "${PROBE_NS}" \
  --db "${PROBE_DB}" \
  --hide-welcome <"${fifo_perm}" >"${out_perm}" 2>&1 || true) &
perm_pid=$!
exec 3>"${fifo_perm}"
echo "LIVE SELECT * FROM probe_private;" >&3
sleep 0.8

# While the listener is running, create rows for bob and alice.
# NOTE: owner uses $auth (record), not $auth.id, for record<probe_user> fields.
docker compose -f "${compose_file}" exec -T surrealdb /surreal sql --endpoint "${endpoint_ws}" --token "${bob_token}" --ns "${PROBE_NS}" --db "${PROBE_DB}" --hide-welcome --json <<'SQL' >/dev/null
CREATE probe_private SET owner=$auth, body='bob private';
SQL
docker compose -f "${compose_file}" exec -T surrealdb /surreal sql --endpoint "${endpoint_ws}" --token "${alice_token}" --ns "${PROBE_NS}" --db "${PROBE_DB}" --hide-welcome --json <<'SQL' >/dev/null
CREATE probe_private SET owner=$auth, body='alice private';
SQL

sleep 1.2
exec 3>&-
wait "${perm_pid}" || true
perm_output="$(cat "${out_perm}")"
rm -rf "${tmp_dir_perm}"
perm_pass="FAIL"
if echo "${perm_output}" | grep -q "alice private" && ! echo "${perm_output}" | grep -q "bob private"; then
  perm_pass="PASS"
fi

## Pattern 8: Graph traversal mechanics (relations)
surreal_sql_root_json $'DEFINE TABLE probe_graph_user SCHEMAFULL;\nDEFINE FIELD name ON TABLE probe_graph_user TYPE string;\nDEFINE TABLE probe_graph_entity SCHEMAFULL;\nDEFINE FIELD label ON TABLE probe_graph_entity TYPE string;\nDEFINE TABLE probe_follows TYPE RELATION IN probe_graph_user OUT probe_graph_entity;' >/dev/null
surreal_sql_root_json "CREATE probe_graph_user:alice SET name='Alice'; CREATE probe_graph_entity:rt05 SET label='RT 05'; RELATE probe_graph_user:alice->probe_follows->probe_graph_entity:rt05 SET created_at=time::now();" >/dev/null
graph_query="$(surreal_sql_root_json "SELECT ->probe_follows->probe_graph_entity.* FROM probe_graph_user:alice;")"
graph_pass="FAIL"
if echo "${graph_query}" | grep -q "RT 05"; then
  graph_pass="PASS"
fi

## Pattern 9: Full-text search mechanics (FULLTEXT index)
surreal_sql_root_json "DEFINE ANALYZER probe_simple TOKENIZERS class, punct FILTERS lowercase, ascii;" >/dev/null
surreal_sql_root_json "CREATE probe_search:1 SET text='Graph databases are great.'; CREATE probe_search:2 SET text='Relational databases store tables.'; CREATE probe_search:3 SET text='This document mentions graphs.';" >/dev/null
surreal_sql_root_json "DEFINE INDEX idx_probe_search_text ON TABLE probe_search FIELDS text FULLTEXT ANALYZER probe_simple BM25;" >/dev/null
fts_query="$(surreal_sql_root_json "SELECT id, search::score(1) as score FROM probe_search WHERE text @1@ 'graph' ORDER BY score DESC LIMIT 5;")"
fts_pass="FAIL"
if echo "${fts_query}" | grep -q "search::score" || echo "${fts_query}" | grep -q "\"score\":"; then
  fts_pass="PASS"
fi

## Pattern 10: Vector search mechanics (HNSW index + KNN operator)
surreal_sql_root_json "CREATE probe_vector:1 SET text='Graph note', embedding=[0.10,0.20,0.30]; CREATE probe_vector:2 SET text='Table note', embedding=[0.05,0.10,0.00]; CREATE probe_vector:3 SET text='Another graph note', embedding=[0.20,0.10,0.25];" >/dev/null
surreal_sql_root_json "DEFINE INDEX idx_probe_vector_embedding ON TABLE probe_vector FIELDS embedding HNSW DIMENSION 3 DIST COSINE;" >/dev/null
vector_query="$(surreal_sql_root_json "SELECT id, vector::distance::knn() as dist FROM probe_vector WHERE embedding <|2,100|> [0.12, 0.18, 0.27] ORDER BY dist;")"
vector_pass="FAIL"
if printf "%s" "${vector_query}" | python3 -c "import json,sys; rows=json.load(sys.stdin)[0]; ok=(len(rows)==2 and all(r.get('dist') is not None for r in rows)); raise SystemExit(0 if ok else 1)"; then
  vector_pass="PASS"
fi
vector_explain="$(surreal_sql_root_json "SELECT id FROM probe_vector WHERE embedding <|2,100|> [0.12, 0.18, 0.27] EXPLAIN FULL;")"

cat > "${output_file}" <<REPORT
# SurrealDB v3 — Live Docker DB Probe (Gotong)

Date: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
Compose: \`${compose_file}\`
Endpoint: \`${endpoint_ws}\`
Server version (HTTP \`/version\`): \`${server_version}\`
Namespace/DB: \`${PROBE_NS}/${PROBE_DB}\`

## Result Summary

| Pattern | Result | Notes |
|---|---|---|
| Unique composite index (idempotency) | ${idem_pass} | Duplicate insert blocked; count stays 1 |
| Deterministic ordering + catch-up cursor | ${ordering_pass} | Stable \`(created_at,message_id)\` cursor logic |
| \`EXPLAIN\` shows index scan | ${explain_pass} | Verifies index-backed filtering; order still uses TopK sort |
| \`LIVE SELECT\` over WS | ${live_ws_pass} | WS streams \`CREATE\` actions |
| \`LIVE SELECT\` over HTTP fails (expected) | ${live_http_pass} | Confirms WS-only realtime in this env |
| \`LIVE SELECT DIFF\` payload | ${live_diff_pass} | Includes \`op: 'change'\` for field updates |
| Permission-filtered live subscription | ${perm_pass} | Alice sees only her rows |
| Relation traversal mechanics | ${graph_pass} | 1-hop traversal returns nested objects |
| Full-text index + \`@1@\` query | ${fts_pass} | Uses \`FULLTEXT\` + \`search::score(1)\` |
| Vector KNN + HNSW index | ${vector_pass} | Uses \`<|k,ef|>\` + \`vector::distance::knn()\` |

## Key Outputs (selected)

### Unique composite index (duplicate error)
\`\`\`json
${idem_dup}
\`\`\`

### Ordering query output
\`\`\`json
${ordering_json}
\`\`\`

### Catch-up query output
\`\`\`json
${catchup_json}
\`\`\`

### EXPLAIN plan
\`\`\`json
${explain_plan}
\`\`\`

### ORDER BY projection requirement (error excerpt)
\`\`\`text
${order_idiom_error}
\`\`\`

### LIVE SELECT over WS (excerpt)
\`\`\`text
$(echo "${live_ws_output}" | sed -n '1,20p')
\`\`\`

### LIVE SELECT over HTTP (excerpt)
\`\`\`text
$(echo "${live_http_output}" | sed -n '1,10p')
\`\`\`

### LIVE SELECT DIFF (excerpt)
\`\`\`text
$(echo "${live_diff_output}" | sed -n '1,30p')
\`\`\`

### Permission-filtered LIVE SELECT (excerpt)
\`\`\`text
$(echo "${perm_output}" | sed -n '1,30p')
\`\`\`

### Graph traversal result
\`\`\`json
${graph_query}
\`\`\`

### Full-text search result
\`\`\`json
${fts_query}
\`\`\`

### Vector search result
\`\`\`json
${vector_query}
\`\`\`

### Vector EXPLAIN FULL (note: may not show vector index operator yet)
\`\`\`json
${vector_explain}
\`\`\`
REPORT

echo "Wrote ${output_file}"
