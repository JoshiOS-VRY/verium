#!/usr/bin/env bash
# Verify App Store export has production push entitlements before TestFlight upload.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCHIVE="${ROOT}/src-tauri/gen/apple/build/vericonomy-wallet_iOS.xcarchive"
ARCHIVE_APP="${ARCHIVE}/Products/Applications/Vericonomy Wallet.app"
IPA="${ROOT}/src-tauri/gen/apple/build/app-store/Vericonomy Wallet.ipa"

read_aps() {
  local app_path="$1"
  codesign -d --entitlements :- "${app_path}" 2>/dev/null \
    | plutil -extract aps-environment raw -o - - 2>/dev/null || true
}

read_task_allow() {
  local app_path="$1"
  codesign -d --entitlements :- "${app_path}" 2>/dev/null \
    | plutil -extract get-task-allow raw -o - - 2>/dev/null || true
}

check_app() {
  local label="$1"
  local app_path="$2"
  local aps task_allow
  aps="$(read_aps "${app_path}")"
  task_allow="$(read_task_allow "${app_path}")"
  echo "==> ${label}"
  codesign -d --entitlements :- "${app_path}" 2>/dev/null || true
  echo ""
  echo "aps-environment: ${aps:-<missing>}"
  echo "get-task-allow: ${task_allow:-<missing>}"
  echo ""
  if [[ "${aps}" == "production" ]]; then
    return 0
  fi
  return 1
}

# TestFlight uploads the exported IPA — export re-signs with App Store distribution.
if [[ -f "${IPA}" ]]; then
  TMPDIR_IPA="$(mktemp -d)"
  unzip -q "${IPA}" -d "${TMPDIR_IPA}"
  IPA_APP="${TMPDIR_IPA}/Payload/Vericonomy Wallet.app"
  if check_app "Exported IPA (upload this to TestFlight)" "${IPA_APP}"; then
    rm -rf "${TMPDIR_IPA}"
    echo "OK: production push entitlements present in IPA."
    exit 0
  fi
  rm -rf "${TMPDIR_IPA}"
  echo "FAIL: IPA missing production push entitlements." >&2
  echo "Re-run npm run ios:archive (export step re-signs for App Store)." >&2
  exit 1
fi

if [[ ! -d "${ARCHIVE_APP}" ]]; then
  echo "error: no IPA or xcarchive found." >&2
  echo "  IPA:     ${IPA}" >&2
  echo "  Archive: ${ARCHIVE_APP}" >&2
  echo "Run npm run ios:archive first." >&2
  exit 1
fi

if check_app "xcarchive (pre-export — often still development-signed)" "${ARCHIVE_APP}"; then
  echo "OK: production push entitlements present in archive."
  exit 0
fi

echo "note: xcarchive is often development-signed until export." >&2
echo "Run npm run ios:export (or full ios:archive) and re-check the IPA." >&2
exit 1
