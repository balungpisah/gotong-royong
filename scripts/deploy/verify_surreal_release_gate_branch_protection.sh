#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

REPO=""
BRANCH="main"
REQUIRED_CHECK="CI / surreal-release-gates"
OUTPUT_FILE="docs/research/branch-protection-surreal-release-gates-latest.md"
DRY_RUN=0

usage() {
  cat <<'EOF'
Usage:
  scripts/deploy/verify_surreal_release_gate_branch_protection.sh --repo <owner/repo> [--branch <branch>] [--required-check <name>] [--output <path>] [--dry-run]

Examples:
  scripts/deploy/verify_surreal_release_gate_branch_protection.sh --repo my-org/gotong-royong --branch main
  scripts/deploy/verify_surreal_release_gate_branch_protection.sh --repo my-org/gotong-royong --dry-run
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      shift
      REPO="${1:-}"
      ;;
    --branch)
      shift
      BRANCH="${1:-}"
      ;;
    --required-check)
      shift
      REQUIRED_CHECK="${1:-}"
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

if [[ -z "${REPO}" ]]; then
  echo "--repo is required (format: owner/repo)" >&2
  usage
  exit 1
fi

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

require_cmd date
require_cmd python3

NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
mkdir -p "$(dirname "${OUTPUT_FILE}")"

if [[ "${DRY_RUN}" == "1" ]]; then
  cat > "${OUTPUT_FILE}" <<EOF
# Surreal Release Gate Branch Protection Check

Date: ${NOW_UTC}
Repository: \`${REPO}\`
Branch: \`${BRANCH}\`
Required check: \`${REQUIRED_CHECK}\`
Mode: \`dry-run\`
Status: \`DRY_RUN\`

## Next command

\`\`\`bash
scripts/deploy/verify_surreal_release_gate_branch_protection.sh --repo ${REPO} --branch ${BRANCH}
\`\`\`

## Context

- This check verifies that GitHub branch protection requires \`${REQUIRED_CHECK}\`.
- Related debt tracker: \`docs/research/frontend-hot-path-integration-debt.md\` (CHAT-API-004)
- Related runbook: \`docs/deployment/chat-attachment-storage-lifecycle-runbook.md\`
EOF
  echo "Wrote ${OUTPUT_FILE}"
  exit 0
fi

require_cmd gh

PROTECTION_JSON="$(mktemp)"
set +e
gh api \
  -H "Accept: application/vnd.github+json" \
  "repos/${REPO}/branches/${BRANCH}/protection" > "${PROTECTION_JSON}" 2> /tmp/verify_surreal_release_gate_branch_protection.err
api_exit=$?
set -e

if [[ ${api_exit} -ne 0 ]]; then
  error_text="$(cat /tmp/verify_surreal_release_gate_branch_protection.err)"
  status="ERROR"
  guidance="- Ensure \`gh auth status\` is logged in and has permission to read branch protection rules."
  if [[ "${error_text}" == *"Branch not protected"* ]]; then
    status="FAIL"
    guidance="- Branch protection is not configured for this branch yet."
  fi
  cat > "${OUTPUT_FILE}" <<EOF
# Surreal Release Gate Branch Protection Check

Date: ${NOW_UTC}
Repository: \`${REPO}\`
Branch: \`${BRANCH}\`
Required check: \`${REQUIRED_CHECK}\`
Mode: \`live\`
Status: \`${status}\`

## Error

\`\`\`
${error_text}
\`\`\`

## Context

${guidance}
- Related debt tracker: \`docs/research/frontend-hot-path-integration-debt.md\` (CHAT-API-004)
EOF
  rm -f "${PROTECTION_JSON}" /tmp/verify_surreal_release_gate_branch_protection.err
  if [[ "${status}" == "FAIL" ]]; then
    echo "branch protection is not configured; see ${OUTPUT_FILE}" >&2
  else
    echo "branch protection query failed; see ${OUTPUT_FILE}" >&2
  fi
  exit 1
fi

python_output="$(python3 - "${PROTECTION_JSON}" "${REQUIRED_CHECK}" <<'PY'
import json
import pathlib
import sys

payload_path = pathlib.Path(sys.argv[1])
required = sys.argv[2]

payload = json.loads(payload_path.read_text(encoding="utf-8"))
required_status = payload.get("required_status_checks")
contexts = []
if isinstance(required_status, dict):
    contexts = required_status.get("contexts") or []
if not isinstance(contexts, list):
    contexts = []
contexts = [str(value) for value in contexts]

enabled = required_status is not None
present = required in contexts
status = "PASS" if enabled and present else "FAIL"

print(status)
print("true" if enabled else "false")
print("true" if present else "false")
print(json.dumps(contexts))
PY
)"

status="$(echo "${python_output}" | sed -n '1p')"
required_checks_enabled="$(echo "${python_output}" | sed -n '2p')"
required_check_present="$(echo "${python_output}" | sed -n '3p')"
contexts_json="$(echo "${python_output}" | sed -n '4p')"

python3 - "${contexts_json}" > /tmp/verify_surreal_release_gate_branch_protection_contexts.txt <<'PY'
import json
import sys

contexts = json.loads(sys.argv[1])
if not contexts:
    print("- (none)")
else:
    for context in contexts:
        print(f"- `{context}`")
PY

contexts_list="$(cat /tmp/verify_surreal_release_gate_branch_protection_contexts.txt)"

cat > "${OUTPUT_FILE}" <<EOF
# Surreal Release Gate Branch Protection Check

Date: ${NOW_UTC}
Repository: \`${REPO}\`
Branch: \`${BRANCH}\`
Required check: \`${REQUIRED_CHECK}\`
Mode: \`live\`
Status: \`${status}\`

## Evaluation

- Required status checks enabled: \`${required_checks_enabled}\`
- Required check present: \`${required_check_present}\`

## Branch required checks

${contexts_list}

## Context

- Related debt tracker: \`docs/research/frontend-hot-path-integration-debt.md\` (CHAT-API-004)
- Related runbook: \`docs/deployment/chat-attachment-storage-lifecycle-runbook.md\`
EOF

rm -f "${PROTECTION_JSON}" /tmp/verify_surreal_release_gate_branch_protection.err /tmp/verify_surreal_release_gate_branch_protection_contexts.txt

if [[ "${status}" != "PASS" ]]; then
  echo "required status check missing; see ${OUTPUT_FILE}" >&2
  exit 1
fi

echo "Wrote ${OUTPUT_FILE}"
