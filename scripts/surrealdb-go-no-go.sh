#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../" && pwd)"
PROBE_SCRIPT="${ROOT_DIR}/docs/research/samples/surrealdb/pattern_probe.sh"

: "${SURREAL_BIN:=surreal}"
: "${LOCKED_TARGET_VERSION:=3.0.0-beta.4}"

if ! command -v "${SURREAL_BIN}" >/dev/null 2>&1; then
  echo "Surreal binary not found: ${SURREAL_BIN}" >&2
  echo "Set SURREAL_BIN to a valid Surreal executable." >&2
  exit 1
fi

if [[ ! -x "${PROBE_SCRIPT}" ]]; then
  echo "Probe script is not executable: ${PROBE_SCRIPT}" >&2
  exit 1
fi

OUTPUT_FILE="${1:-${ROOT_DIR}/docs/research/surrealdb-go-no-go-latest.md}"
export SURREAL_BIN
export LOCKED_TARGET_VERSION

"${PROBE_SCRIPT}" "${OUTPUT_FILE}"

python3 - <<'PY' "${OUTPUT_FILE}" "${LOCKED_TARGET_VERSION}"
import pathlib
import re
import sys

report_path, expected_version = sys.argv[1], sys.argv[2]
text = pathlib.Path(report_path).read_text()

failures = []
summary = re.findall(r"^\|\s*[^|]+\|\s*(PASS|FAIL)\s*\|", text, re.M)
if not summary:
    failures.append("no go/no-go table entries found")
else:
    for idx, status in enumerate(summary, 1):
        if status != "PASS":
            failures.append(f"pattern #{idx} failed")

if f"matches locked target {expected_version}" not in text:
    failures.append(f"locked target {expected_version} was not confirmed in probe output")

if failures:
    print("SurrealDB go/no-go checks failed:", file=sys.stderr)
    for failure in failures:
        print(f"- {failure}", file=sys.stderr)
    print("\nLatest report:", file=sys.stderr)
    print(text, file=sys.stderr)
    sys.exit(1)

print("SurrealDB go/no-go checks passed")
print(f"Report: {report_path}")
PY
