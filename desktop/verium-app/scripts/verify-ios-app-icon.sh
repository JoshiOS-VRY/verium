#!/usr/bin/env bash
# Verify an exported App Store IPA embeds the same AppIcon as the Xcode asset catalog.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"
# shellcheck source=ios-icon-cache.sh
source "${ROOT}/scripts/ios-icon-cache.sh"

IPA="${1:-${ROOT}/src-tauri/gen/apple/build/app-store/Vericonomy Wallet.ipa}"
SOURCE_120="${ROOT}/src-tauri/icons/ios/AppIcon-60x60@2x.png"
CATALOG="${ROOT}/src-tauri/gen/apple/Assets.xcassets"

if [[ ! -f "${IPA}" ]]; then
  echo "error: IPA not found at ${IPA}" >&2
  echo "  Run: npm run ios:archive" >&2
  exit 1
fi

if [[ ! -f "${SOURCE_120}" ]]; then
  echo "error: source icon missing — run: npm run icons:ios" >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

unzip -q "${IPA}" -d "${TMP}/ipa"
APP="$(find "${TMP}/ipa/Payload" -maxdepth 1 -name '*.app' -print -quit)"
if [[ -z "${APP}" ]]; then
  echo "error: no .app bundle inside IPA" >&2
  exit 1
fi

IPA_120="${APP}/AppIcon60x60@2x.png"
IPA_CAR="${APP}/Assets.car"
BUILD_VERSION="$(/usr/libexec/PlistBuddy -c 'Print CFBundleVersion' "${APP}/Info.plist" 2>/dev/null || echo unknown)"
BUILD_SHORT="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "${APP}/Info.plist" 2>/dev/null || echo unknown)"
echo "IPA build: ${BUILD_SHORT} (${BUILD_VERSION})"
if [[ ! -f "${IPA_120}" ]]; then
  echo "error: ${IPA_120} missing from IPA" >&2
  exit 1
fi
if [[ ! -f "${IPA_CAR}" ]]; then
  echo "error: Assets.car missing from IPA" >&2
  exit 1
fi

normalize_png() {
  sips -s format png "$1" --out "$2" >/dev/null
}

normalize_png "${SOURCE_120}" "${TMP}/source-120.png"
normalize_png "${IPA_120}" "${TMP}/ipa-120.png"

SOURCE_HASH="$(shasum -a 256 "${TMP}/source-120.png" | awk '{print $1}')"
IPA_HASH="$(shasum -a 256 "${TMP}/ipa-120.png" | awk '{print $1}')"

if [[ "${SOURCE_HASH}" != "${IPA_HASH}" ]]; then
  echo "error: IPA AppIcon does not match the current asset catalog." >&2
  echo "  catalog AppIcon-60x60@2x: ${SOURCE_HASH}" >&2
  echo "  IPA AppIcon60x60@2x:      ${IPA_HASH}" >&2
  echo "" >&2
  echo "Xcode likely reused a cached Assets.car. Fix:" >&2
  echo "  IOS_FORCE_ICON_REBUILD=1 npm run ios:archive" >&2
  exit 1
fi

echo "OK: IPA phone icon matches catalog (AppIcon-60x60@2x)."

echo "==> Verifying App Store marketing icon (1024 — what Transporter previews)…"
mkdir -p "${TMP}/fresh"
"${WITH_XCODE}" xcrun actool "${CATALOG}" \
  --compile "${TMP}/fresh" \
  --platform iphoneos \
  --minimum-deployment-target 14.0 \
  --app-icon AppIcon \
  --output-partial-info-plist "${TMP}/partial.plist" \
  >/dev/null

FRESH_CAR="$(find "${TMP}/fresh" -name '*.car' -print -quit)"
if [[ -z "${FRESH_CAR}" ]]; then
  echo "error: actool did not produce Assets.car for comparison." >&2
  exit 1
fi

MARKETING_OK="$("${WITH_XCODE}" python3 -c "
import json, subprocess, sys
fresh_car = sys.argv[1]
ipa_car = sys.argv[2]

def marketing_sha(path):
    out = subprocess.check_output(['assetutil', '--info', path], text=True)
    data = json.loads(out)
    for item in data:
        if item.get('AssetType') == 'Icon Image' and item.get('PixelWidth') == 1024:
            return item.get('SHA1Digest')
    return None

fresh = marketing_sha(fresh_car)
ipa = marketing_sha(ipa_car)
if not fresh or not ipa:
    sys.exit(2)
sys.exit(0 if fresh == ipa else 1)
" "${FRESH_CAR}" "${IPA_CAR}")"
VERIFY_EXIT=$?
if [[ ${VERIFY_EXIT} -ne 0 ]]; then
  echo "error: IPA marketing icon (1024) does not match a fresh compile of AppIcon.appiconset." >&2
  echo "  Transporter will keep showing a stale icon until this is fixed." >&2
  echo "  Run: IOS_FORCE_ICON_REBUILD=1 npm run ios:archive" >&2
  exit 1
fi

ios_icon_mark_archived "${ROOT}"

PREVIEW_DIR="${ROOT}/src-tauri/gen/apple/build/app-store"
mkdir -p "${PREVIEW_DIR}"
cp -f "${ROOT}/src-tauri/icons/ios/AppIcon-512@2x.png" "${PREVIEW_DIR}/icon-from-catalog-1024.png"
cp -f "${TMP}/ipa-120.png" "${PREVIEW_DIR}/icon-from-ipa-phone.png"

echo "OK: IPA marketing icon matches catalog (App Store / Transporter preview)."
echo "    IPA: ${IPA}"
echo "    Preview PNGs: ${PREVIEW_DIR}/icon-from-catalog-1024.png"
echo "                  ${PREVIEW_DIR}/icon-from-ipa-phone.png"
echo ""
echo "Transporter shows the icon baked into each build at upload time."
echo "Older builds (e.g. 13–15) keep their old icon forever."
echo "Upload this IPA as a NEW CFBundleVersion (${BUILD_VERSION}) to refresh Transporter."
