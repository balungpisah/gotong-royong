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

assert_not_contains() {
  local file="$1"
  local pattern="$2"
  local message="$3"
  if rg -q --fixed-strings "$pattern" "$file"; then
    echo "FAILED: ${message}" >&2
    echo "  file: ${file}" >&2
    echo "  unexpected content: ${pattern}" >&2
    exit 1
  fi
}

require_cmd jq
require_cmd rg

stage_a="deploy/monitoring/prometheusrule-pack-c-stage-a.yaml"
stage_b="deploy/monitoring/prometheusrule-pack-c-stage-b.yaml"
stage_c="deploy/monitoring/prometheusrule-pack-c-stage-c.yaml"
dashboard="deploy/monitoring/grafana-pack-c-cutover-dashboard.json"
monitoring_readme="deploy/monitoring/README.md"
runbook="docs/deployment/feed-involvement-fallback-removal-runbook.md"
docs_index="docs/README.md"

for required_file in \
  "$stage_a" \
  "$stage_b" \
  "$stage_c" \
  "$dashboard" \
  "$monitoring_readme" \
  "$runbook" \
  "$docs_index"; do
  if [[ ! -f "$required_file" ]]; then
    echo "FAILED: required file missing: ${required_file}" >&2
    exit 1
  fi
done

jq empty "$dashboard"

assert_contains "$stage_a" "gotong.packc.stage: stage-a" "stage A label mismatch"
assert_contains "$stage_a" "GotongPackCStageAEdgeErrorRatioWarning" "stage A edge_error warning alert missing"
assert_contains "$stage_a" ") > 0.001" "stage A edge_error warning threshold mismatch"
assert_contains "$stage_a" "GotongPackCStageAEdgeErrorRatioCritical" "stage A edge_error critical alert missing"
assert_contains "$stage_a" ") > 0.005" "stage A edge_error critical threshold mismatch"
assert_contains "$stage_a" "GotongPackCStageAShadowMismatchWarning" "stage A shadow mismatch warning missing"
assert_contains "$stage_a" "(sum(increase(gotong_api_feed_involvement_shadow_mismatch_total[30m])) or vector(0)) > 3" "stage A shadow mismatch warning threshold mismatch"
assert_contains "$stage_a" "GotongPackCStageAShadowMismatchCritical" "stage A shadow mismatch critical missing"
assert_contains "$stage_a" "(sum(increase(gotong_api_feed_involvement_shadow_mismatch_total[30m])) or vector(0)) > 10" "stage A shadow mismatch critical threshold mismatch"
assert_contains "$stage_a" "max(gotong_api_http_request_duration_seconds{route=\"/v1/feed\",method=\"GET\",quantile=\"0.95\"}) > 0.18" "stage A feed latency query mismatch"
assert_not_contains "$stage_a" "gotong_api_http_request_duration_seconds_bucket{route=\"/v1/feed\",method=\"GET\"}" "stage A still uses histogram bucket metric"

assert_contains "$stage_b" "gotong.packc.stage: stage-b" "stage B label mismatch"
assert_contains "$stage_b" "GotongPackCStageBEdgeErrorRatioWarning" "stage B edge_error warning alert missing"
assert_contains "$stage_b" ") > 0.0005" "stage B edge_error warning threshold mismatch"
assert_contains "$stage_b" "GotongPackCStageBEdgeErrorRatioCritical" "stage B edge_error critical alert missing"
assert_contains "$stage_b" ") > 0.002" "stage B edge_error critical threshold mismatch"
assert_contains "$stage_b" "GotongPackCStageBShadowMismatchWarning" "stage B shadow mismatch warning missing"
assert_contains "$stage_b" "(sum(increase(gotong_api_feed_involvement_shadow_mismatch_total[30m])) or vector(0)) > 1" "stage B shadow mismatch warning threshold mismatch"
assert_contains "$stage_b" "GotongPackCStageBShadowMismatchCritical" "stage B shadow mismatch critical missing"
assert_contains "$stage_b" "(sum(increase(gotong_api_feed_involvement_shadow_mismatch_total[30m])) or vector(0)) > 5" "stage B shadow mismatch critical threshold mismatch"
assert_contains "$stage_b" "max(gotong_api_http_request_duration_seconds{route=\"/v1/feed\",method=\"GET\",quantile=\"0.95\"}) > 0.18" "stage B feed latency query mismatch"
assert_not_contains "$stage_b" "gotong_api_http_request_duration_seconds_bucket{route=\"/v1/feed\",method=\"GET\"}" "stage B still uses histogram bucket metric"

assert_contains "$stage_c" "gotong.packc.stage: stage-c" "stage C label mismatch"
assert_contains "$stage_c" "GotongPackCStageCEdgeErrorRatioWarning" "stage C edge_error warning alert missing"
assert_contains "$stage_c" ") > 0.0001" "stage C edge_error warning threshold mismatch"
assert_contains "$stage_c" "GotongPackCStageCEdgeErrorRatioCritical" "stage C edge_error critical alert missing"
assert_contains "$stage_c" ") > 0.0005" "stage C edge_error critical threshold mismatch"
assert_contains "$stage_c" "max(gotong_api_http_request_duration_seconds{route=\"/v1/feed\",method=\"GET\",quantile=\"0.95\"}) > 0.18" "stage C feed latency query mismatch"
assert_not_contains "$stage_c" "gotong_api_http_request_duration_seconds_bucket{route=\"/v1/feed\",method=\"GET\"}" "stage C still uses histogram bucket metric"
assert_contains "$stage_c" "GotongPackCStageCFallbackLaneNonZero" "stage C fallback-lane alert missing"
assert_contains "$stage_c" "(sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~\"feed|search\",lane=~\"fallback|legacy\"}[5m])) or vector(0)) > 0" "stage C fallback lane threshold mismatch"
assert_not_contains "$stage_c" "ShadowMismatch" "stage C should not include shadow mismatch alerts"

assert_contains "$monitoring_readme" "just pack-c-alerts-stage-a" "monitoring README missing stage A just shortcut"
assert_contains "$monitoring_readme" "just pack-c-alerts-stage-b" "monitoring README missing stage B just shortcut"
assert_contains "$monitoring_readme" "just pack-c-alerts-stage-c" "monitoring README missing stage C just shortcut"
assert_contains "$monitoring_readme" "just pack-c-alerts-plan stage=stage-c" "monitoring README missing dry-run just shortcut"
assert_contains "$monitoring_readme" "just pack-c-stage-a-end-to-end" "monitoring README missing stage A end-to-end shortcut"
assert_contains "$monitoring_readme" "just pack-c-stage-b-end-to-end" "monitoring README missing stage B end-to-end shortcut"
assert_contains "$monitoring_readme" "just pack-c-stage-c-end-to-end" "monitoring README missing stage C end-to-end shortcut"
assert_contains "$monitoring_readme" "just pack-c-stage-end-to-end-dry-run stage-b" "monitoring README missing generic dry-run end-to-end shortcut"

assert_contains "$runbook" "just pack-c-alerts-stage-a" "runbook missing stage A command"
assert_contains "$runbook" "just pack-c-alerts-stage-b" "runbook missing stage B command"
assert_contains "$runbook" "just pack-c-alerts-stage-c" "runbook missing stage C command"
assert_contains "$runbook" "just pack-c-stage-a-go-no-go" "runbook missing stage A go/no-go command"
assert_contains "$runbook" "just pack-c-stage-b-go-no-go" "runbook missing stage B go/no-go command"
assert_contains "$runbook" "just pack-c-stage-c-go-no-go" "runbook missing stage C go/no-go command"
assert_contains "$runbook" "just pack-c-stage-a-end-to-end" "runbook missing stage A end-to-end command"
assert_contains "$runbook" "just pack-c-stage-b-end-to-end" "runbook missing stage B end-to-end command"
assert_contains "$runbook" "just pack-c-stage-c-end-to-end" "runbook missing stage C end-to-end command"
assert_contains "$runbook" "just pack-c-stage-end-to-end-dry-run stage-b" "runbook missing dry-run end-to-end command"
assert_contains "$runbook" "deploy/monitoring/grafana-pack-c-cutover-dashboard.json" "runbook missing dashboard reference"

assert_contains "$docs_index" "Pack C Prometheus Rules" "docs index missing Pack C Prometheus link"
assert_contains "$docs_index" "Pack C Grafana Dashboard" "docs index missing Pack C dashboard link"
assert_contains "$docs_index" "Pack C Stage A Go/No-Go Report" "docs index missing Pack C Stage A go/no-go link"
assert_contains "$docs_index" "Pack C Stage B Go/No-Go Report" "docs index missing Pack C Stage B go/no-go link"
assert_contains "$docs_index" "Pack C Stage C Go/No-Go Report" "docs index missing Pack C Stage C go/no-go link"

echo "Pack C monitoring assets verification: OK"
