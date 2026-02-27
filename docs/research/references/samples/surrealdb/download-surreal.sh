#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${SCRIPT_DIR}/bin"
mkdir -p "$BIN_DIR"
cd "$BIN_DIR"

VERSION="v3.0.0-beta.4"
BASE_URL="https://github.com/surrealdb/surrealdb/releases/download/${VERSION}"

download_darwin_arm64() {
  local archive="surreal-${VERSION}.darwin-arm64.tgz"
  local url="${BASE_URL}/${archive}"
  echo "Downloading ${archive}..."
  curl -fLO "${url}"
  tar -xzf "${archive}"
  mv surreal "surreal-${VERSION}"
  rm "${archive}"
  echo "Done: surreal-${VERSION}"
}

download_linux_x86_64() {
  local archive="surreal-${VERSION}.linux-amd64.tgz"
  local url="${BASE_URL}/${archive}"
  echo "Downloading ${archive}..."
  curl -fLO "${url}"
  tar -xzf "${archive}"
  mv surreal "surreal-${VERSION}"
  rm "${archive}"
  echo "Done: surreal-${VERSION}"
}

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) download_darwin_arm64 ;;
  Linux-x86_64|Linux-amd64) download_linux_x86_64 ;;
  *)
    echo "Unsupported platform: $(uname -s)-$(uname -m)"
    echo "Download manually from: ${BASE_URL}"
    exit 1
    ;;
esac
