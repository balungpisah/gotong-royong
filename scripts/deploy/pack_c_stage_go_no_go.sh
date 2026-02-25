#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

STAGE=""
PROM_URL="http://127.0.0.1:9090"
WINDOW="30m"
STEP="60s"
OUTPUT_FILE=""
DRY_RUN=0

usage() {
  cat <<'EOF'
usage: scripts/deploy/pack_c_stage_go_no_go.sh --stage <stage-a|stage-b|stage-c> [--prom-url <http://prometheus:9090>] [--window <duration>] [--step <duration>] [--output <report-path>] [--dry-run]

examples:
  scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-b
  scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-c --prom-url http://127.0.0.1:9090 --window 60m --step 60s
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --stage)
      shift
      STAGE="${1:-}"
      ;;
    --prom-url)
      shift
      PROM_URL="${1:-}"
      ;;
    --window)
      shift
      WINDOW="${1:-}"
      ;;
    --step)
      shift
      STEP="${1:-}"
      ;;
    --output)
      shift
      OUTPUT_FILE="${1:-}"
      ;;
    --dry-run)
      DRY_RUN=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
  shift
done

case "${STAGE}" in
  stage-a|stage-b|stage-c) ;;
  *)
    echo "--stage is required and must be one of: stage-a|stage-b|stage-c" >&2
    usage
    exit 1
    ;;
esac

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

require_cmd date
require_cmd python3

if [[ -z "${OUTPUT_FILE}" ]]; then
  OUTPUT_FILE="docs/research/pack-c-${STAGE}-go-no-go-latest.md"
fi

NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
mkdir -p "$(dirname "${OUTPUT_FILE}")"

set +e
python3 - "${STAGE}" "${PROM_URL}" "${WINDOW}" "${STEP}" "${NOW_UTC}" "${OUTPUT_FILE}" "${DRY_RUN}" <<'PY'
import json
import math
import pathlib
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from typing import Optional


def parse_duration_to_seconds(value: str) -> int:
    text = value.strip().lower()
    match = re.fullmatch(r"(\d+)([smhd])", text)
    if not match:
        raise ValueError(f"invalid duration: {value!r} (expected <int><s|m|h|d>)")
    amount = int(match.group(1))
    unit = match.group(2)
    multiplier = {"s": 1, "m": 60, "h": 3600, "d": 86400}[unit]
    return amount * multiplier


def duration_label(seconds: Optional[int]) -> str:
    if seconds is None:
        return "n/a"
    if seconds % 3600 == 0:
        return f"{seconds // 3600}h"
    if seconds % 60 == 0:
        return f"{seconds // 60}m"
    return f"{seconds}s"


def fmt_num(value: Optional[float]) -> str:
    if value is None:
        return "n/a"
    return f"{value:.6f}"


def fmt_threshold(value: Optional[float], seconds: Optional[int]) -> str:
    if value is None:
        return "n/a"
    if seconds is None:
        return f">{value:g}"
    return f">{value:g} for {duration_label(seconds)}"


def max_streak(values: list[float], threshold: float) -> int:
    current = 0
    best = 0
    for value in values:
        if value > threshold:
            current += 1
            best = max(best, current)
        else:
            current = 0
    return best


def query_range(prom_url: str, promql: str, start_ts: int, end_ts: int, step: int) -> list[float]:
    base = prom_url.rstrip("/")
    params = urllib.parse.urlencode(
        {
            "query": promql,
            "start": str(start_ts),
            "end": str(end_ts),
            "step": str(step),
        }
    )
    url = f"{base}/api/v1/query_range?{params}"
    request = urllib.request.Request(url, headers={"Accept": "application/json"})
    with urllib.request.urlopen(request, timeout=20) as response:
        body = response.read().decode("utf-8")
    payload = json.loads(body)
    if payload.get("status") != "success":
        raise RuntimeError(f"prometheus query failed: {payload.get('error', 'unknown error')}")
    matrix = payload.get("data", {}).get("result", [])
    if not matrix:
        return []
    by_ts: dict[float, float] = {}
    for series in matrix:
        for ts_text, value_text in series.get("values", []):
            ts = float(ts_text)
            try:
                value = float(value_text)
            except ValueError:
                continue
            if not math.isfinite(value):
                continue
            by_ts[ts] = by_ts.get(ts, 0.0) + value
    return [value for _, value in sorted(by_ts.items(), key=lambda item: item[0])]


stage = sys.argv[1]
prom_url = sys.argv[2]
window_raw = sys.argv[3]
step_raw = sys.argv[4]
now_utc = sys.argv[5]
output_file = pathlib.Path(sys.argv[6])
dry_run = sys.argv[7] == "1"

window_seconds = parse_duration_to_seconds(window_raw)
step_seconds = parse_duration_to_seconds(step_raw)
if step_seconds <= 0:
    raise ValueError("step must be > 0")
if window_seconds < step_seconds:
    raise ValueError("window must be >= step")

end_ts = int(time.time())
start_ts = end_ts - window_seconds

queries = {
    "edge_error_ratio": """(
  (sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search",lane="edge_error"}[5m])) or vector(0))
  /
  clamp_min((sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m])) or vector(0)), 1)
)""",
    "edge_partial_ratio": """(
  (sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search",lane="edge_partial"}[5m])) or vector(0))
  /
  clamp_min((sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~"feed|search"}[5m])) or vector(0)), 1)
)""",
    "shadow_mismatch_growth_30m": "(sum(increase(gotong_api_feed_involvement_shadow_mismatch_total[30m])) or vector(0))",
    "feed_p95_latency_seconds": "max(gotong_api_http_request_duration_seconds{route=\"/v1/feed\",method=\"GET\",quantile=\"0.95\"})",
    "feed_5xx_ratio": """(
  (sum(rate(gotong_api_http_errors_total{route="/v1/feed",method="GET"}[5m])) or vector(0))
  /
  clamp_min((sum(rate(gotong_api_http_requests_total{route="/v1/feed",method="GET"}[5m])) or vector(0)), 1)
)""",
    "fallback_legacy_lane_rps": "(sum(rate(gotong_api_feed_involvement_lane_requests_total{endpoint=~\"feed|search\",lane=~\"fallback|legacy\"}[5m])) or vector(0))",
}

stage_profiles = {
    "stage-a": [
        {"key": "edge_error_ratio", "name": "Edge error ratio", "warn": 0.001, "warn_for": 600, "crit": 0.005, "crit_for": 300},
        {"key": "edge_partial_ratio", "name": "Edge partial ratio", "warn": 0.003, "warn_for": 600, "crit": 0.01, "crit_for": 300},
        {"key": "shadow_mismatch_growth_30m", "name": "Shadow mismatch increase[30m]", "warn": 3.0, "warn_for": 300, "crit": 10.0, "crit_for": 300},
        {"key": "feed_p95_latency_seconds", "name": "Feed p95 latency (seconds)", "warn": 0.18, "warn_for": 900, "crit": 0.25, "crit_for": 600},
        {"key": "feed_5xx_ratio", "name": "Feed 5xx ratio", "warn": 0.01, "warn_for": 600, "crit": 0.02, "crit_for": 300},
    ],
    "stage-b": [
        {"key": "edge_error_ratio", "name": "Edge error ratio", "warn": 0.0005, "warn_for": 600, "crit": 0.002, "crit_for": 300},
        {"key": "edge_partial_ratio", "name": "Edge partial ratio", "warn": 0.001, "warn_for": 600, "crit": 0.005, "crit_for": 300},
        {"key": "shadow_mismatch_growth_30m", "name": "Shadow mismatch increase[30m]", "warn": 1.0, "warn_for": 300, "crit": 5.0, "crit_for": 300},
        {"key": "feed_p95_latency_seconds", "name": "Feed p95 latency (seconds)", "warn": 0.18, "warn_for": 900, "crit": 0.25, "crit_for": 600},
        {"key": "feed_5xx_ratio", "name": "Feed 5xx ratio", "warn": 0.01, "warn_for": 600, "crit": 0.02, "crit_for": 300},
    ],
    "stage-c": [
        {"key": "edge_error_ratio", "name": "Edge error ratio", "warn": 0.0001, "warn_for": 600, "crit": 0.0005, "crit_for": 300},
        {"key": "edge_partial_ratio", "name": "Edge partial ratio", "warn": 0.0005, "warn_for": 600, "crit": 0.002, "crit_for": 300},
        {"key": "feed_p95_latency_seconds", "name": "Feed p95 latency (seconds)", "warn": 0.18, "warn_for": 900, "crit": 0.25, "crit_for": 600},
        {"key": "feed_5xx_ratio", "name": "Feed 5xx ratio", "warn": 0.01, "warn_for": 600, "crit": 0.02, "crit_for": 300},
        {"key": "fallback_legacy_lane_rps", "name": "Fallback/legacy lane RPS", "warn": None, "warn_for": None, "crit": 0.0, "crit_for": 900},
    ],
}

rows = []
query_error = None
for metric in stage_profiles[stage]:
    warn_required = math.ceil(metric["warn_for"] / step_seconds) if metric["warn"] is not None else None
    crit_required = math.ceil(metric["crit_for"] / step_seconds) if metric["crit"] is not None else None
    row = {
        "signal": metric["name"],
        "latest": None,
        "max": None,
        "warn_threshold": fmt_threshold(metric["warn"], metric["warn_for"]),
        "crit_threshold": fmt_threshold(metric["crit"], metric["crit_for"]),
        "warn_streak": 0,
        "warn_required": warn_required,
        "crit_streak": 0,
        "crit_required": crit_required,
        "status": "SKIPPED" if dry_run else "PASS",
        "notes": "dry-run (no Prometheus query executed)" if dry_run else "within thresholds",
    }

    if dry_run:
        rows.append(row)
        continue

    try:
        values = query_range(prom_url, queries[metric["key"]], start_ts, end_ts, step_seconds)
    except (urllib.error.URLError, RuntimeError, TimeoutError) as exc:
        query_error = str(exc)
        break

    if not values:
        row["status"] = "WARN"
        row["notes"] = "no_data (check scrape and traffic coverage)"
        rows.append(row)
        continue

    row["latest"] = values[-1]
    row["max"] = max(values)

    notes: list[str] = []
    if metric["warn"] is not None:
        warn_streak = max_streak(values, metric["warn"])
        row["warn_streak"] = warn_streak
        if warn_required is not None and warn_streak >= warn_required:
            notes.append(f"warning sustained {warn_streak} points")
    if metric["crit"] is not None:
        crit_streak = max_streak(values, metric["crit"])
        row["crit_streak"] = crit_streak
        if crit_required is not None and crit_streak >= crit_required:
            row["status"] = "FAIL"
            notes.append(f"critical sustained {crit_streak} points")
        elif crit_streak > 0:
            row["status"] = "WARN"
            notes.append(f"critical spikes {crit_streak} points (below sustain window)")

    if row["status"] != "FAIL" and metric["warn"] is not None and warn_required is not None:
        if row["warn_streak"] >= warn_required:
            row["status"] = "WARN"
        elif row["warn_streak"] > 0 and row["status"] == "PASS":
            notes.append(f"warning spikes {row['warn_streak']} points (below sustain window)")

    if notes:
        row["notes"] = "; ".join(notes)
    rows.append(row)

stage_title = stage.replace("-", " ").title()
if query_error is not None:
    decision = "ERROR"
elif dry_run:
    decision = "DRY_RUN"
elif any(row["status"] == "FAIL" for row in rows):
    decision = "NO_GO"
elif any(row["status"] == "WARN" for row in rows):
    decision = "HOLD"
else:
    decision = "GO"

fail_count = sum(1 for row in rows if row["status"] == "FAIL")
warn_count = sum(1 for row in rows if row["status"] == "WARN")
pass_count = sum(1 for row in rows if row["status"] == "PASS")

if decision == "NO_GO":
    recommendation = "Do not advance stage. Roll back fallback switch if this persists."
elif decision == "HOLD":
    recommendation = "Hold progression and continue observation or remediate warning signals."
elif decision == "GO":
    recommendation = "Gate is clear to advance to the next rollout stage."
elif decision == "DRY_RUN":
    recommendation = "Dry-run only. Execute without --dry-run against Prometheus for a real decision."
else:
    recommendation = "Prometheus query failed. Fix connectivity/auth and rerun."

report_lines = [
    f"# Pack C {stage_title} Go/No-Go Report",
    "",
    f"Date: {now_utc}",
    f"Stage: `{stage}`",
    f"Prometheus URL: `{prom_url}`",
    f"Window: `{window_raw}`",
    f"Step: `{step_raw}`",
    f"Mode: `{'dry-run' if dry_run else 'live'}`",
    "",
    "## Decision Summary",
    "",
    "| Decision | PASS | WARN | FAIL |",
    "|---|---:|---:|---:|",
    f"| {decision} | {pass_count} | {warn_count} | {fail_count} |",
    "",
    f"Recommendation: {recommendation}",
    "",
]

if query_error is not None:
    report_lines.extend(
        [
            "## Query Error",
            "",
            f"`{query_error}`",
            "",
        ]
    )

report_lines.extend(
    [
        "## Signal Evaluation",
        "",
        "| Signal | Latest | Max | Warn threshold | Crit threshold | Warn streak | Crit streak | Status | Notes |",
        "|---|---:|---:|---|---|---:|---:|---|---|",
    ]
)

for row in rows:
    warn_streak_text = "n/a" if row["warn_required"] is None else f"{row['warn_streak']}/{row['warn_required']}"
    crit_streak_text = "n/a" if row["crit_required"] is None else f"{row['crit_streak']}/{row['crit_required']}"
    report_lines.append(
        "| {signal} | {latest} | {max_value} | {warn_threshold} | {crit_threshold} | {warn_streak} | {crit_streak} | {status} | {notes} |".format(
            signal=row["signal"],
            latest=fmt_num(row["latest"]),
            max_value=fmt_num(row["max"]),
            warn_threshold=row["warn_threshold"],
            crit_threshold=row["crit_threshold"],
            warn_streak=warn_streak_text,
            crit_streak=crit_streak_text,
            status=row["status"],
            notes=row["notes"],
        )
    )

report_lines.extend(
    [
        "",
        "Policy reference:",
        "- `docs/deployment/feed-involvement-fallback-alert-thresholds.md`",
        "- `docs/deployment/feed-involvement-fallback-removal-runbook.md`",
        "",
        "Notes:",
        "- GO requires zero sustained critical and warning breaches for this observation window.",
        "- NO_GO is emitted when any critical threshold is sustained for its configured `for` duration.",
        "- HOLD blocks progression without forcing immediate rollback.",
    ]
)

output_file.write_text("\n".join(report_lines) + "\n")

if decision in {"GO", "DRY_RUN"}:
    sys.exit(0)
sys.exit(1)
PY
result=$?
set -e

echo "Wrote ${OUTPUT_FILE}"
exit "${result}"
