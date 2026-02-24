#!/usr/bin/env bash
set -euo pipefail

# A lightweight Surreal CLI shim which runs the SurrealDB CLI in Docker.
# Useful on macOS where we don't want to vendor host binaries.
#
# Defaults:
#   SURREAL_DOCKER_IMAGE=surrealdb/surrealdb:v3.0.0
#
# Notes:
# - On macOS, a CLI running inside a container cannot reach host services on 127.0.0.1.
#   We rewrite --endpoint values to use host.docker.internal so scripts can keep using
#   ws://127.0.0.1:PORT and http://127.0.0.1:PORT.
# - For `start ... --bind 127.0.0.1:PORT`, we rewrite bind to 0.0.0.0:PORT and publish
#   the port back to 127.0.0.1:PORT on the host.

IMAGE="${SURREAL_DOCKER_IMAGE:-surrealdb/surrealdb:v3.0.0}"
OS_NAME="$(uname -s || true)"

rewrite_endpoint_if_needed() {
  local endpoint="$1"
  if [[ "${OS_NAME}" != "Darwin" ]]; then
    printf "%s" "${endpoint}"
    return 0
  fi

  endpoint="${endpoint//ws:\\/\\/127.0.0.1/ws:\\/\\/host.docker.internal}"
  endpoint="${endpoint//http:\\/\\/127.0.0.1/http:\\/\\/host.docker.internal}"
  endpoint="${endpoint//ws:\\/\\/localhost/ws:\\/\\/host.docker.internal}"
  endpoint="${endpoint//http:\\/\\/localhost/http:\\/\\/host.docker.internal}"
  printf "%s" "${endpoint}"
}

rewrite_bind_if_needed() {
  local bind="$1"
  # When running inside a container, binding to 127.0.0.1 prevents host access via -p.
  bind="${bind//127.0.0.1:/0.0.0.0:}"
  bind="${bind//localhost:/0.0.0.0:}"
  printf "%s" "${bind}"
}

extract_port_from_bind() {
  local bind="$1"
  # Expected forms: 127.0.0.1:8000 / 0.0.0.0:8000 / :8000
  if [[ "${bind}" =~ :([0-9]+)$ ]]; then
    printf "%s" "${BASH_REMATCH[1]}"
    return 0
  fi
  return 1
}

extract_port_from_endpoint() {
  local endpoint="$1"
  # Expected forms: ws://host:8000 / http://host:8000
  if [[ "${endpoint}" =~ :([0-9]+)(/.*)?$ ]]; then
    printf "%s" "${BASH_REMATCH[1]}"
    return 0
  fi
  return 1
}

container_running() {
  local name="$1"
  docker inspect -f '{{.State.Running}}' "${name}" 2>/dev/null | grep -q '^true$'
}

subcommand="${1:-}"
if [[ -z "${subcommand}" ]]; then
  echo "Usage: surreal-docker.sh <command> [args...]" >&2
  exit 2
fi

args=("$@")
rewritten=()
bind_port=""
endpoint_port=""

for ((i=0; i<${#args[@]}; i++)); do
  arg="${args[$i]}"

  case "${arg}" in
    --endpoint)
      rewritten+=("${arg}")
      i=$((i+1))
      endpoint_val="${args[$i]}"
      rewritten_endpoint="$(rewrite_endpoint_if_needed "${endpoint_val}")"
      rewritten+=("${rewritten_endpoint}")
      if [[ -z "${endpoint_port}" ]]; then
        endpoint_port="$(extract_port_from_endpoint "${endpoint_val}" || true)"
      fi
      ;;
    --endpoint=*)
      val="${arg#--endpoint=}"
      rewritten+=("--endpoint=$(rewrite_endpoint_if_needed "${val}")")
      if [[ -z "${endpoint_port}" ]]; then
        endpoint_port="$(extract_port_from_endpoint "${val}" || true)"
      fi
      ;;
    --bind)
      rewritten+=("${arg}")
      i=$((i+1))
      bind_val="$(rewrite_bind_if_needed "${args[$i]}")"
      rewritten+=("${bind_val}")
      if [[ -z "${bind_port}" ]]; then
        bind_port="$(extract_port_from_bind "${bind_val}" || true)"
      fi
      ;;
    --bind=*)
      val="${arg#--bind=}"
      bind_val="$(rewrite_bind_if_needed "${val}")"
      rewritten+=("--bind=${bind_val}")
      if [[ -z "${bind_port}" ]]; then
        bind_port="$(extract_port_from_bind "${bind_val}" || true)"
      fi
      ;;
    *)
      rewritten+=("${arg}")
      ;;
  esac
done

docker_args=(run --rm)
target_port="${bind_port:-${endpoint_port}}"
container_name="${SURREAL_DOCKER_CONTAINER:-}"
if [[ -z "${container_name}" && -n "${target_port}" ]]; then
  container_name="surreal-probe-${target_port}"
fi

case "${subcommand}" in
  start)
    # Ensure the process can be started in the background by the caller.
    if [[ -n "${bind_port}" ]]; then
      docker_args+=(-p "127.0.0.1:${bind_port}:${bind_port}")
    fi
    if [[ -n "${container_name}" ]]; then
      docker_args+=(--name "${container_name}")
    fi
    exec docker "${docker_args[@]}" "${IMAGE}" "${rewritten[@]}"
    ;;
  sql)
    # `sql` commonly reads from stdin.
    if [[ -n "${container_name}" ]] && container_running "${container_name}"; then
      exec docker exec -i "${container_name}" /surreal "${rewritten[@]}"
    fi
    exec docker "${docker_args[@]}" -i "${IMAGE}" "${rewritten[@]}"
    ;;
  is-ready)
    if [[ -n "${container_name}" ]] && container_running "${container_name}"; then
      exec docker exec "${container_name}" /surreal "${rewritten[@]}"
    fi
    exec docker "${docker_args[@]}" "${IMAGE}" "${rewritten[@]}"
    ;;
  *)
    exec docker "${docker_args[@]}" "${IMAGE}" "${rewritten[@]}"
    ;;
esac
