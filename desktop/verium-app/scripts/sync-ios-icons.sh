#!/usr/bin/env bash
# Regenerate iOS app icons from the Vericonomy logo and sync into the Xcode asset catalog.
# Icons are resized directly from a padded 1024px master (tauri icon crops to content bounds).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOGO="${ROOT}/public/img/vericonomy/vericonomylogo.png"
ICON_OUT="${ROOT}/src-tauri/icons"
IOS_SRC="${ICON_OUT}/ios"
IOS_DST="${ROOT}/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset"
# iOS logo-based icons need generous inset — Apple's squircle mask makes edge-to-edge
# artwork look oversized on the home screen (~32% smaller than full-bleed reads well).
IOS_ICON_CANVAS=1024
IOS_LOGO_SCALE="${IOS_LOGO_SCALE:-0.68}"
SCALED_LOGO="${ICON_OUT}/.ios-icon-source.png"

if [[ ! -f "${LOGO}" ]]; then
  echo "error: logo not found at ${LOGO}" >&2
  exit 1
fi

LOGO_SIZE="$(python3 -c "print(int(${IOS_ICON_CANVAS} * ${IOS_LOGO_SCALE}))")"
PAD="$(( (IOS_ICON_CANVAS - LOGO_SIZE) / 2 ))"

echo "Preparing iOS icon source: logo ${LOGO_SIZE}px centered on ${IOS_ICON_CANVAS}px canvas…"
mkdir -p "${ICON_OUT}" "${IOS_SRC}"
npx --yes sharp-cli -i "${LOGO}" -o "${SCALED_LOGO}.tmp.png" resize "${LOGO_SIZE}" "${LOGO_SIZE}"
npx --yes sharp-cli -i "${SCALED_LOGO}.tmp.png" -o "${SCALED_LOGO}" \
  extend "${PAD}" "${PAD}" "${PAD}" "${PAD}" --background "#000000"
rm -f "${SCALED_LOGO}.tmp.png"

write_ios_icon() {
  local px="$1"
  local filename="$2"
  local out="${IOS_SRC}/${filename}"
  npx --yes sharp-cli -i "${SCALED_LOGO}" -o "${out}" resize "${px}" "${px}"
  npx --yes sharp-cli -i "${out}" -o "${out}" flatten "#000000"
}

echo "Generating iOS icon sizes from padded master…"
# iPhone
write_ios_icon 40  "AppIcon-20x20@2x.png"
write_ios_icon 60  "AppIcon-20x20@3x.png"
write_ios_icon 58  "AppIcon-29x29@2x-1.png"
write_ios_icon 87  "AppIcon-29x29@3x.png"
write_ios_icon 80  "AppIcon-40x40@2x.png"
write_ios_icon 120 "AppIcon-40x40@3x.png"
write_ios_icon 120 "AppIcon-60x60@2x.png"
write_ios_icon 180 "AppIcon-60x60@3x.png"
# iPad
write_ios_icon 20  "AppIcon-20x20@1x.png"
write_ios_icon 40  "AppIcon-20x20@2x-1.png"
write_ios_icon 29  "AppIcon-29x29@1x.png"
write_ios_icon 58  "AppIcon-29x29@2x.png"
write_ios_icon 40  "AppIcon-40x40@1x.png"
write_ios_icon 80  "AppIcon-40x40@2x-1.png"
write_ios_icon 76  "AppIcon-76x76@1x.png"
write_ios_icon 152 "AppIcon-76x76@2x.png"
write_ios_icon 167 "AppIcon-83.5x83.5@2x.png"
# App Store / marketing
write_ios_icon 1024 "AppIcon-512@2x.png"

if [[ -d "${IOS_DST}" ]]; then
  cp -f "${IOS_SRC}"/*.png "${IOS_DST}/"
  echo "Synced iOS icons to ${IOS_DST}"
else
  echo "warning: ${IOS_DST} not found — run npm run tauri:ios:init first." >&2
fi
