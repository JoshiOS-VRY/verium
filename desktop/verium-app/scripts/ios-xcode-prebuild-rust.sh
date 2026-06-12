#!/usr/bin/env bash
# Xcode pre-build phase: use libapp.a staged by compile-ios-rust.sh.
# Avoids `tauri ios xcode-script`, which requires a parent `tauri ios build` WebSocket.
set -euo pipefail

APPLE_DIR="${SRCROOT:?}"
LIBAPP_ARM="${APPLE_DIR}/Externals/arm64/${CONFIGURATION}/libapp.a"
LIBAPP_X86="${APPLE_DIR}/Externals/x86_64/${CONFIGURATION}/libapp.a"

if [[ ! -f "${LIBAPP_ARM}" ]]; then
  echo "error: missing ${LIBAPP_ARM}" >&2
  echo "Run npm run ios:deploy from desktop/verium-app (Rust compile step)." >&2
  exit 1
fi

echo "Using prebuilt libapp.a ($(ls -lh "${LIBAPP_ARM}" | awk '{print $5}'))"
mkdir -p "$(dirname "${LIBAPP_X86}")"
cp -f "${LIBAPP_ARM}" "${LIBAPP_X86}"
