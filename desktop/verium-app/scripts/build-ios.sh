#!/usr/bin/env bash
# Build the Vericonomy iOS light wallet (remote-RPC mode; no bundled veriumd/vericoind).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"

cd "${ROOT}"

BUILD_DIR="${ROOT}/src-tauri/gen/apple/build"
# Tauri renames the .app into build/arm64-sim (or device target). Stale
# xcarchive or output folders cause "Directory not empty (os error 66)".
if [[ -d "${BUILD_DIR}" ]]; then
  rm -rf \
    "${BUILD_DIR}/arm64-sim" \
    "${BUILD_DIR}/aarch64-sim" \
    "${BUILD_DIR}/vericonomy-wallet_iOS.xcarchive"
fi

if [[ -z "${APPLE_DEVELOPMENT_TEAM:-}" ]]; then
  echo "warning: APPLE_DEVELOPMENT_TEAM is not set — building for simulator without signing." >&2
  "${WITH_XCODE}" npm run tauri:ios:build:sim:inner
else
  "${WITH_XCODE}" npm run tauri:ios:build:inner
fi

echo "iOS build finished. Artifacts are under src-tauri/gen/apple/build/"
