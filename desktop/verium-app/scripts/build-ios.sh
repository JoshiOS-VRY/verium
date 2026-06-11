#!/usr/bin/env bash
# Build the Vericonomy iOS light wallet (remote-RPC mode; no bundled veriumd/vericoind).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"

cd "${ROOT}"

echo "Building frontend…"
npm run build

# Tauri embeds dist/ into libapp.a at Rust compile time. Without this, Xcode can
# reuse a stale static library and the phone keeps showing an old UI bundle.
echo "Clearing stale iOS Rust codegen caches…"
rm -rf "${ROOT}/src-tauri/target/aarch64-apple-ios"/*/build/vericonomy-wallet-*/out/tauri-codegen-assets \
  "${ROOT}/src-tauri/gen/apple/Externals/arm64/debug/libapp.a" \
  "${ROOT}/src-tauri/gen/apple/Externals/arm64/release/libapp.a" 2>/dev/null || true

if [[ -d "${ROOT}/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset" ]]; then
  ICON_SRC="${ROOT}/src-tauri/icons/ios"
  ICON_DST="${ROOT}/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset"
  if [[ -d "${ICON_SRC}" ]]; then
    cp -f "${ICON_SRC}"/*.png "${ICON_DST}/"
  fi
fi

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
echo ""
echo "Install on a connected iPhone:"
echo "  ./scripts/install-ios-device.sh JQT"
echo ""
echo "Note: npm run tauri:ios:run (without -r) uses a DEBUG libapp.a."
echo "      For device testing, prefer ./scripts/build-ios.sh + install-ios-device.sh"
echo "      or: npm run tauri:ios:run:release -- JQT"
