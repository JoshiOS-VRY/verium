#!/usr/bin/env bash
# Regenerate iOS app icons: premium gradient canvas + glowing logo composite.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOGO="${IOS_ICON_LOGO:-${ROOT}/public/img/vericonomy/ios-app-icon.png}"
ICON_OUT="${ROOT}/src-tauri/icons"
IOS_SRC="${ICON_OUT}/ios"
IOS_DST="${ROOT}/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset"
IOS_ICON_CANVAS=1024
IOS_ICON_SCALE="${IOS_ICON_SCALE:-0.9}"
IOS_BG="${IOS_BG:-#020617}"
SCALED_MASTER="${ICON_OUT}/.ios-icon-source.png"
BG_SVG="${ICON_OUT}/.ios-icon-bg.svg"
BG_PNG="${ICON_OUT}/.ios-icon-bg.png"
LOGO_KEYED="${ICON_OUT}/.ios-icon-logo-keyed.png"

if [[ ! -f "${LOGO}" ]]; then
  echo "error: iOS logo not found at ${LOGO}" >&2
  exit 1
fi

LOGO_W="$(sips -g pixelWidth "${LOGO}" 2>/dev/null | awk '/pixelWidth/ {print $2}')"
LOGO_H="$(sips -g pixelHeight "${LOGO}" 2>/dev/null | awk '/pixelHeight/ {print $2}')"
if [[ "${LOGO_W}" != "${LOGO_H}" ]] || [[ "${LOGO_W}" -lt 512 ]]; then
  echo "error: iOS logo must be a square PNG at least 512×512 (got ${LOGO_W}×${LOGO_H})." >&2
  exit 1
fi

LOGO_SIZE="$(python3 -c "print(int(${IOS_ICON_CANVAS} * ${IOS_ICON_SCALE}))")"
PAD="$(( (IOS_ICON_CANVAS - LOGO_SIZE) / 2 ))"

echo "Compositing iOS icons: logo ${LOGO_SIZE}px on premium gradient (${IOS_ICON_SCALE} scale)…"
mkdir -p "${ICON_OUT}" "${IOS_SRC}"
rm -f "${SCALED_MASTER}" "${BG_SVG}" "${BG_PNG}" "${LOGO_KEYED}" "${IOS_SRC}"/*.png

# Option 5 — Apple Design Award radial depth (cyan core → electric blue → deep navy).
cat > "${BG_SVG}" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" width="${IOS_ICON_CANVAS}" height="${IOS_ICON_CANVAS}" viewBox="0 0 ${IOS_ICON_CANVAS} ${IOS_ICON_CANVAS}">
  <defs>
    <radialGradient id="bg" cx="50%" cy="35%" r="72%" fx="50%" fy="32%">
      <stop offset="0%" stop-color="#61D7FF"/>
      <stop offset="28%" stop-color="#1974F6"/>
      <stop offset="55%" stop-color="#0B2555"/>
      <stop offset="100%" stop-color="#020617"/>
    </radialGradient>
    <radialGradient id="ambient" cx="50%" cy="32%" r="38%">
      <stop offset="0%" stop-color="#DDF8FF" stop-opacity="0.18"/>
      <stop offset="45%" stop-color="#61D7FF" stop-opacity="0.08"/>
      <stop offset="100%" stop-color="#61D7FF" stop-opacity="0"/>
    </radialGradient>
  </defs>
  <rect width="${IOS_ICON_CANVAS}" height="${IOS_ICON_CANVAS}" fill="url(#bg)"/>
  <rect width="${IOS_ICON_CANVAS}" height="${IOS_ICON_CANVAS}" fill="url(#ambient)"/>
</svg>
SVG

npx --yes sharp-cli -i "${BG_SVG}" -o "${BG_PNG}" resize "${IOS_ICON_CANVAS}" "${IOS_ICON_CANVAS}"

SHARP_RUNTIME="${ICON_OUT}/.sharp-runtime"
if [[ ! -d "${SHARP_RUNTIME}/node_modules/sharp" ]]; then
  echo "Installing sharp for icon compositing (one-time)…"
  mkdir -p "${SHARP_RUNTIME}"
  npm install --prefix "${SHARP_RUNTIME}" --no-save sharp >/dev/null
fi

# Key black backdrop out of the rendered logo, then composite with bloom on the gradient.
LOGO="${LOGO}" BG_PNG="${BG_PNG}" OUT="${SCALED_MASTER}" LOGO_SIZE="${LOGO_SIZE}" PAD="${PAD}" \
  NODE_PATH="${SHARP_RUNTIME}/node_modules" node "${ROOT}/scripts/composite-ios-icon.cjs"

rm -f "${BG_SVG}" "${BG_PNG}" "${LOGO_KEYED}"

write_ios_icon() {
  local px="$1"
  local filename="$2"
  local out="${IOS_SRC}/${filename}"
  npx --yes sharp-cli -i "${SCALED_MASTER}" -o "${out}" resize "${px}" "${px}"
  npx --yes sharp-cli -i "${out}" -o "${out}" flatten "${IOS_BG}"
}

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
  touch "${IOS_DST}/Contents.json"
  echo "Synced iOS icons to ${IOS_DST}"
else
  echo "warning: ${IOS_DST} not found — run npm run tauri:ios:init first." >&2
fi

STAMP_FILE="${ROOT}/src-tauri/gen/apple/.ios-icon-stamp"
NEW_FP="$(shasum -a 256 "${IOS_SRC}/AppIcon-512@2x.png" | awk '{print $1}')"
OLD_FP="$(cat "${STAMP_FILE}" 2>/dev/null || true)"
printf '%s\n' "${NEW_FP}" > "${STAMP_FILE}"
if [[ "${NEW_FP}" != "${OLD_FP}" ]]; then
  echo "AppIcon fingerprint changed (${OLD_FP:-none} → ${NEW_FP})"
  echo "  Run a full ios:archive (not ios:export alone) so Xcode recompiles Assets.car."
fi
