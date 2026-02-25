#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ROUTES_FILE="crates/api/src/routes/mod.rs"
WORKER_FILE="crates/worker/src/main.rs"
DOC_FILE="docs/architecture/hot-path-api-shapes.md"

pass() {
    printf '[OK] %s\n' "$1"
}

fail() {
    printf '[FAIL] %s\n' "$1" >&2
    exit 1
}

check_pattern() {
    local file="$1"
    local pattern="$2"
    local message="$3"
    if rg -q --fixed-strings "$pattern" "$file"; then
        pass "$message"
    else
        fail "$message (missing '$pattern' in $file)"
    fi
}

echo "Checking ontology enrichment guardrails..."

check_pattern "$ROUTES_FILE" "ontology_note_create" "Idempotent ontology note create key exists"
check_pattern "$ROUTES_FILE" "format!(\"action:{predicate}\")" "HasAction triples normalize to action:*"
check_pattern "$ROUTES_FILE" "enqueue_async_enrichment" "Async enrichment is explicitly gated"
check_pattern "$ROUTES_FILE" "if enqueue_async_enrichment {" "Async enrichment only enqueues when needed"
check_pattern "$ROUTES_FILE" "\"feedback_enriched_at_ms\"" "Feedback timestamp field is present in API writes"
check_pattern "$WORKER_FILE" "\"tags_enriched_at_ms\"" "Tags timestamp field is present in worker writes"
check_pattern "$WORKER_FILE" "JobType::OntologyNoteEnrich" "Worker handles ontology enrichment job type"
check_pattern "$DOC_FILE" "tags_enriched_at_ms" "Docs include tag enrichment timestamp contract"
check_pattern "$DOC_FILE" "feedback_enriched_at_ms" "Docs include feedback enrichment timestamp contract"

if [[ "${1:-}" == "--with-tests" ]]; then
    echo "Running test suite..."
    cargo test -q
    pass "cargo test passed"
fi

echo "Ontology enrichment guardrail checks passed."
