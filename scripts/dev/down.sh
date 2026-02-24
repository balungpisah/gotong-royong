#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

compose_file="${COMPOSE_FILE:-compose.dev.yaml}"

docker compose -f "$compose_file" down
