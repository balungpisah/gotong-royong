#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

assert_contains() {
  local file="$1"
  local pattern="$2"
  local message="$3"
  if ! rg -q --fixed-strings "$pattern" "$file"; then
    echo "FAILED: ${message}" >&2
    echo "  file: ${file}" >&2
    echo "  expected to contain: ${pattern}" >&2
    exit 1
  fi
}

require_cmd rg

manifest="deploy/monitoring/prometheusrule-chat-attachment-lifecycle.yaml"
monitoring_readme="deploy/monitoring/README.md"
runbook="docs/deployment/chat-attachment-storage-lifecycle-runbook.md"
docs_index="docs/README.md"
justfile_path="justfile"
debt_file="docs/research/frontend-hot-path-integration-debt.md"

for required_file in \
  "$manifest" \
  "$monitoring_readme" \
  "$runbook" \
  "$docs_index" \
  "$justfile_path" \
  "$debt_file"; do
  if [[ ! -f "$required_file" ]]; then
    echo "FAILED: required file missing: ${required_file}" >&2
    exit 1
  fi
done

assert_contains "$manifest" "gotong.monitoring.domain: chat-attachment" "manifest domain label missing"
assert_contains "$manifest" "GotongChatAttachmentUpload5xxRatioWarning" "upload warning alert missing"
assert_contains "$manifest" "GotongChatAttachmentUpload5xxRatioCritical" "upload critical alert missing"
assert_contains "$manifest" "GotongChatAttachmentUploadP95LatencyWarning" "upload latency alert missing"
assert_contains "$manifest" "max(gotong_api_http_request_duration_seconds{route=\"/v1/chat/attachments/upload\",method=\"POST\",quantile=\"0.95\"}) > 0.75" "upload latency query mismatch"
assert_contains "$manifest" "or vector(0)" "manifest should coerce absent error counters to zero"
assert_contains "$manifest" "GotongChatAttachmentDownload5xxRatioWarning" "download warning alert missing"
assert_contains "$manifest" "GotongChatAttachmentDownload5xxRatioCritical" "download critical alert missing"
assert_contains "$manifest" "GotongChatAttachmentStorageGrowthWarning" "storage growth alert missing"
if rg -q --fixed-strings "gotong_api_http_request_duration_seconds_bucket{route=\"/v1/chat/attachments/upload\",method=\"POST\"}" "$manifest"; then
  echo "FAILED: upload latency alert still uses histogram bucket metric" >&2
  exit 1
fi
assert_contains "$manifest" "docs/deployment/chat-attachment-storage-lifecycle-runbook.md" "runbook annotation missing"

assert_contains "$monitoring_readme" "prometheusrule-chat-attachment-lifecycle.yaml" "monitoring README missing chat attachment manifest"
assert_contains "$monitoring_readme" "just chat-attachment-alerts-apply" "monitoring README missing alerts apply command"
assert_contains "$monitoring_readme" "just chat-attachment-alerts-plan" "monitoring README missing alerts dry-run command"
assert_contains "$monitoring_readme" "just chat-attachment-alerts-verify" "monitoring README missing alerts verify command"

assert_contains "$runbook" "just chat-attachment-alerts-plan" "runbook missing alerts plan command"
assert_contains "$runbook" "just chat-attachment-alerts-apply" "runbook missing alerts apply command"
assert_contains "$runbook" "just chat-attachment-alerts-verify" "runbook missing alerts verify command"
assert_contains "$runbook" "deploy/monitoring/prometheusrule-chat-attachment-lifecycle.yaml" "runbook missing manifest reference"

assert_contains "$docs_index" "Chat Attachment Monitoring Rules" "docs index missing chat attachment monitoring link"
assert_contains "$justfile_path" "chat-attachment-alerts-apply" "justfile missing chat attachment apply target"
assert_contains "$justfile_path" "chat-attachment-alerts-plan" "justfile missing chat attachment plan target"
assert_contains "$justfile_path" "chat-attachment-alerts-verify" "justfile missing chat attachment verify target"
assert_contains "$debt_file" "chat_attachment_prometheus_rules.sh" "debt tracker missing monitoring script reference"

echo "Chat attachment monitoring assets verification: OK"
