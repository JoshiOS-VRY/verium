#!/usr/bin/env bash
# Build an App Store–ready IPA for TestFlight upload (Transporter or Xcode Organizer).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"
APPLE_DIR="${ROOT}/src-tauri/gen/apple"
PROJECT="${APPLE_DIR}/vericonomy-wallet.xcodeproj/project.pbxproj"
XCODE_PROJECT="${APPLE_DIR}/vericonomy-wallet.xcodeproj"
SCHEME="vericonomy-wallet_iOS"
BUILD_DIR="${APPLE_DIR}/build"
ARCHIVE_PATH="${BUILD_DIR}/vericonomy-wallet_iOS.xcarchive"
EXPORT_DIR="${BUILD_DIR}/app-store"
EXPORT_PLIST="${BUILD_DIR}/ExportOptions.plist"

is_placeholder_team() {
  local team
  team="$(echo "${1}" | tr '[:upper:]' '[:lower:]')"
  [[ -z "${team}" ]] || [[ "${team}" == "your_team_id" ]] || [[ "${team}" == "xxxxxxxxxx" ]] \
    || [[ "${team}" == "your-team-id" ]] || [[ "${team}" == *"your_team"* ]]
}

read_team_from_xcode_project() {
  if [[ ! -f "${PROJECT}" ]]; then
    return 1
  fi
  grep -m1 'DEVELOPMENT_TEAM = ' "${PROJECT}" | sed -E 's/.*DEVELOPMENT_TEAM = "?([^";]+)"?;.*/\1/'
}

resolve_team_id() {
  local from_env="${APPLE_DEVELOPMENT_TEAM:-}"
  if [[ -n "${from_env}" ]] && ! is_placeholder_team "${from_env}"; then
    echo "${from_env}"
    return 0
  fi
  local from_project
  from_project="$(read_team_from_xcode_project || true)"
  if [[ -n "${from_project}" ]] && ! is_placeholder_team "${from_project}"; then
    echo "${from_project}"
    return 0
  fi
  return 1
}

TEAM_ID="$(resolve_team_id || true)"
if [[ -z "${TEAM_ID}" ]]; then
  echo "error: could not determine Apple Developer team ID." >&2
  echo "  Do not use the docs placeholder YOUR_TEAM_ID." >&2
  echo "  Find your team ID in Xcode → Settings → Accounts, or App Store Connect → Membership." >&2
  echo "  Then: export APPLE_DEVELOPMENT_TEAM=396ZMFA3PP  # example" >&2
  exit 1
fi

if is_placeholder_team "${APPLE_DEVELOPMENT_TEAM:-}"; then
  echo "warning: APPLE_DEVELOPMENT_TEAM was a placeholder; using team ${TEAM_ID} from Xcode project." >&2
fi

export APPLE_DEVELOPMENT_TEAM="${TEAM_ID}"

if [[ "${EXPORT_ONLY:-}" != "1" ]]; then
  export SKIP_IOS_INSTALL=1
  bash "${ROOT}/scripts/build-ios.sh"

  bash "${ROOT}/scripts/patch-ios-xcode-project.sh"

  echo "==> xcodebuild archive (App Store)…"
  "${WITH_XCODE}" xcodebuild \
    -project "${XCODE_PROJECT}" \
    -scheme "${SCHEME}" \
    -configuration release \
    -destination 'generic/platform=iOS' \
    -archivePath "${ARCHIVE_PATH}" \
    -allowProvisioningUpdates \
    DEVELOPMENT_TEAM="${TEAM_ID}" \
    archive
else
  if [[ ! -d "${ARCHIVE_PATH}" ]]; then
    echo "error: EXPORT_ONLY=1 but archive missing at ${ARCHIVE_PATH}" >&2
    echo "  Run npm run ios:archive without EXPORT_ONLY first." >&2
    exit 1
  fi
  echo "==> EXPORT_ONLY=1 — re-exporting existing archive (team ${TEAM_ID})…"
fi

cat > "${EXPORT_PLIST}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>method</key>
	<string>app-store-connect</string>
	<key>teamID</key>
	<string>${TEAM_ID}</string>
	<key>uploadSymbols</key>
	<true/>
	<key>signingStyle</key>
	<string>automatic</string>
</dict>
</plist>
EOF

rm -rf "${EXPORT_DIR}"
mkdir -p "${EXPORT_DIR}"

echo "==> xcodebuild -exportArchive (team ${TEAM_ID})…"
"${WITH_XCODE}" xcodebuild \
  -exportArchive \
  -archivePath "${ARCHIVE_PATH}" \
  -exportPath "${EXPORT_DIR}" \
  -exportOptionsPlist "${EXPORT_PLIST}" \
  -allowProvisioningUpdates

IPA="${EXPORT_DIR}/Vericonomy Wallet.ipa"
if [[ -f "${IPA}" ]]; then
  echo ""
  echo "App Store IPA ready:"
  echo "  ${IPA}"
  echo ""
  echo "Upload with Transporter or Xcode Organizer → Distribute App → App Store Connect."
else
  echo "warning: expected IPA not found; searching export directory…" >&2
  find "${EXPORT_DIR}" -name '*.ipa' -print
fi
