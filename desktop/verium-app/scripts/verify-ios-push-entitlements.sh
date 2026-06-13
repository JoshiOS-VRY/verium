#!/usr/bin/env bash
# Verify the App Store xcarchive has production push entitlements before TestFlight upload.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCHIVE="${ROOT}/src-tauri/gen/apple/build/vericonomy-wallet_iOS.xcarchive"
APP="${ARCHIVE}/Products/Applications/Vericonomy Wallet.app"

if [[ ! -d "${APP}" ]]; then
  echo "error: archive app not found at:" >&2
  echo "  ${APP}" >&2
  echo "Run npm run ios:archive first." >&2
  exit 1
fi

echo "==> Embedded entitlements"
ENTITLEMENTS="$(mktemp)"
codesign -d --entitlements :- "${APP}" 2>/dev/null > "${ENTITLEMENTS}" || true
if [[ ! -s "${ENTITLEMENTS}" ]]; then
  echo "error: could not read entitlements from signed app" >&2
  exit 1
fi
cat "${ENTITLEMENTS}"
rm -f "${ENTITLEMENTS}"

APS="$(codesign -d --entitlements :- "${APP}" 2>/dev/null | plutil -extract aps-environment raw -o - - 2>/dev/null || true)"
echo ""
echo "aps-environment: ${APS:-<missing>}"

if [[ "${APS}" != "production" ]]; then
  echo "" >&2
  echo "FAIL: TestFlight requires aps-environment=production in the signed app." >&2
  echo "Fix: enable Push Notifications on the App ID, regenerate the App Store" >&2
  echo "distribution profile, archive again (npm run ios:archive)." >&2
  exit 1
fi

echo "OK: production push entitlements present in archive."
