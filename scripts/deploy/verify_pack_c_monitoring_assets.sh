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
assert_contains "$stage_a" "increase(gotong_api_feed_involvement_shadow_mismatch_total[30m]) > 3" "stage A shadow mismatch warning threshold mismatch"
assert_contains "$stage_a" "GotongPackCStageAShadowMismatchCritical" "stage A shadow mismatch critical missing"
assert_contains "$stage_a" "increase(gotong_api_feed_involvement_shadow_mismatch_total[30m]) > 10" "stage A shadow mismatch critical threshold mismatch"

assert_contains "$stage_b" "gotong.packc.stage: stage-b" "stage B label mismatch"
assert_contains "$stage_b" "GotongPackCStageBEdgeErrorRatioWarning" "stage B edge_error warning alert missing"
assert_contains "$stage_b" ") > 0.0005" "stage B edge_error warning threshold mismatch"
assert_contains "$stage_b" "GotongPackCStageBEdgeErrorRatioCritical" "stage B edge_error critical alert missing"
assert_contains "$stage_b" ") > 0.002" "stage B edge_error critical threshold mismatch"
assert_contains "$stage_b" "GotongPackCStageBShadowMismatchWarning" "stage B shadow mismatch warning missing"
assert_contains "$stage_b" "increase(gotong_api_feed_involvement_shadow_mismatch_total[30m]) > 1" "stage B shadow mismatch warning threshold mismatch"
assert_contains "$stage_b" "GotongPackCStageBShadowMismatchCritical" "stage B shadow mismatch critical missing"
assert_contains "$stage_b" "increase(gotong_api_feed_involvement_shadow_mismatch_total[30m]) > 5" "stage B shadow mismatch critical threshold mismatch"

assert_contains "$stage_c" "gotong.packc.stage: stage-c" "stage C label mismatch"
assert_contains "$stage_c" "GotongPackCStageCEdgeErrorRatioWarning" "stage C edge_error warning alert missing"
assert_contains "$stage_c" ") > 0.0001" "stage C edge_error warning threshold mismatch"
assert_contains "$stage_c" "GotongPackCStageCEdgeErrorRatioCritical" "stage C edge_error critical alert missing"
assert_contains "$stage_c" ") > 0.0005" "stage C edge_error critical threshold mismatch"
assert_contains "$stage_c" "GotongPackCStageCFallbackLaneNonZero" "stage C fallback-lane alert missing"
assert_contains "$stage_c" "lane=~\"fallback|legacy\"}[5m])) > 0" "stage C fallback lane threshold mismatch"
assert_not_contains "$stage_c" "ShadowMismatch" "stage C should not include shadow mismatch alerts"

assert_contains "$monitoring_readme" "just pack-c-alerts-stage-a" "monitoring README missing stage A just shortcut"
assert_contains "$monitoring_readme" "just pack-c-alerts-stage-b" "monitoring README missing stage B just shortcut"
assert_contains "$monitoring_readme" "just pack-c-alerts-stage-c" "monitoring README missing stage C just shortcut"
assert_contains "$monitoring_readme" "just pack-c-alerts-plan stage=stage-c" "monitoring README missing dry-run just shortcut"

assert_contains "$runbook" "just pack-c-alerts-stage-a" "runbook missing stage A command"
assert_contains "$runbook" "just pack-c-alerts-stage-b" "runbook missing stage B command"
assert_contains "$runbook" "just pack-c-alerts-stage-c" "runbook missing stage C command"
assert_contains "$runbook" "deploy/monitoring/grafana-pack-c-cutover-dashboard.json" "runbook missing dashboard reference"

assert_contains "$docs_index" "Pack C Prometheus Rules" "docs index missing Pack C Prometheus link"
assert_contains "$docs_index" "Pack C Grafana Dashboard" "docs index missing Pack C dashboard link"

echo "Pack C monitoring assets verification: OK"
