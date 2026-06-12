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

flatten_icons() {
  local dir="$1"
  echo "Removing alpha from icons in ${dir} (App Store requires opaque icons)..."
  for f in "${dir}"/*.png; do
    [[ -f "${f}" ]] || continue
    npx --yes sharp-cli -i "${f}" -o "${f}" flatten "#000000"
  done
}

flatten_icons "${IOS_SRC}"

if [[ -d "${IOS_DST}" ]]; then
  cp -f "${IOS_SRC}"/*.png "${IOS_DST}/"
  echo "Synced iOS icons to ${IOS_DST}"
else
  echo "warning: ${IOS_DST} not found — run npm run tauri:ios:init first." >&2
fi
