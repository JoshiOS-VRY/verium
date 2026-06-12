#!/usr/bin/env bash
# Release device build via xcodebuild (exits when done — no tauri ios build WebSocket hang).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"
APPLE_DIR="${ROOT}/src-tauri/gen/apple"
PROJECT="${APPLE_DIR}/vericonomy-wallet.xcodeproj"
SCHEME="vericonomy-wallet_iOS"

bash "${ROOT}/scripts/patch-ios-xcode-project.sh"

echo "==> xcodebuild release (iphoneos)…"
"${WITH_XCODE}" xcodebuild \
  -project "${PROJECT}" \
  -scheme "${SCHEME}" \
  -configuration release \
  -destination 'generic/platform=iOS' \
  -allowProvisioningUpdates \
  build

echo "==> Xcode device build finished."
