#!/usr/bin/env bash
# Regenerate app icons from the Vericonomy logo and sync into the Xcode asset catalog.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOGO="${ROOT}/public/img/vericonomy/vericonomylogo.png"
ICON_OUT="${ROOT}/src-tauri/icons"
IOS_SRC="${ICON_OUT}/ios"
IOS_DST="${ROOT}/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset"

if [[ ! -f "${LOGO}" ]]; then
  echo "error: logo not found at ${LOGO}" >&2
  exit 1
fi

cd "${ROOT}/src-tauri"
npx tauri icon "${LOGO}" -o icons --ios-color "#000000"

if [[ -d "${IOS_DST}" ]]; then
  cp -f "${IOS_SRC}"/*.png "${IOS_DST}/"
  echo "Synced iOS icons to ${IOS_DST}"
else
  echo "warning: ${IOS_DST} not found — run npm run tauri:ios:init first." >&2
fi
