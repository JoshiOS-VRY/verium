#!/usr/bin/env bash
# Install the signed release IPA onto a connected iPhone and launch it.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"
IPA="${ROOT}/src-tauri/gen/apple/build/arm64/Vericonomy Wallet.ipa"

DEVICE_NAME="${1:-JQT}"
DEVICE_ID="${IOS_DEVICE_ID:-}"

cd "${ROOT}"

if [[ ! -f "${IPA}" ]]; then
  echo "error: IPA not found at ${IPA}" >&2
  echo "Run ./scripts/build-ios.sh first." >&2
  exit 1
fi

if [[ -z "${DEVICE_ID}" ]]; then
  DEVICE_ID="$("${WITH_XCODE}" xcrun devicectl list devices 2>/dev/null \
    | awk -v name="${DEVICE_NAME}" '$1 == name { print $3; exit }')"
fi

if [[ -z "${DEVICE_ID}" ]]; then
  echo "error: could not resolve device id for '${DEVICE_NAME}'." >&2
  echo "Pass a name (default JQT) or set IOS_DEVICE_ID." >&2
  "${WITH_XCODE}" xcrun devicectl list devices 2>&1 || true
  exit 1
fi

echo "Installing ${IPA} to ${DEVICE_NAME} (${DEVICE_ID})…"
"${WITH_XCODE}" xcrun devicectl device install app \
  --device "${DEVICE_ID}" \
  "${IPA}"

echo "Launching com.vericonomy.wallet.ios…"
"${WITH_XCODE}" xcrun devicectl device process launch \
  --device "${DEVICE_ID}" \
  com.vericonomy.wallet.ios

echo "Done. Force-quit the app first if the UI still looks stale, then relaunch."
