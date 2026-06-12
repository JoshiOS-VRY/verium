#!/usr/bin/env bash
# Install the signed release build onto a connected iPhone and launch it.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"
BUILD_DIR="${ROOT}/src-tauri/gen/apple/build"
LIBAPP="${ROOT}/src-tauri/gen/apple/Externals/arm64/release/libapp.a"
BUNDLE_ID="com.vericonomy.wallet.ios"

DEVICE_NAME="${1:-${IOS_DEVICE_NAME:-JQT}}"
DEVICE_ID="${IOS_DEVICE_ID:-}"

cd "${ROOT}"

file_mtime() {
  stat -f '%m' "$1" 2>/dev/null || stat -c '%Y' "$1" 2>/dev/null || echo 0
}

find_ipa() {
  local candidates=(
    "${BUILD_DIR}/arm64/Vericonomy Wallet.ipa"
    "${BUILD_DIR}/arm64/"*.ipa
    "${BUILD_DIR}/"*/*.ipa
  )
  local path
  for path in "${candidates[@]}"; do
    if [[ -f "${path}" ]]; then
      echo "${path}"
      return 0
    fi
  done
  return 1
}

find_release_app() {
  local best=""
  local best_mtime=0
  local app
  while IFS= read -r app; do
    local bin="${app}/Vericonomy Wallet"
    if [[ -f "${bin}" ]]; then
      local mtime
      mtime="$(file_mtime "${bin}")"
      if (( mtime > best_mtime )); then
        best="${app}"
        best_mtime="${mtime}"
      fi
    fi
  done < <(find "${HOME}/Library/Developer/Xcode/DerivedData" -path '*/release-iphoneos/Vericonomy Wallet.app' -type d 2>/dev/null)
  if [[ -n "${best}" ]]; then
    echo "${best}"
    return 0
  fi
  return 1
}

# Prefer the artifact that actually contains the latest Rust/UI bundle.
resolve_install_path() {
  local ipa app libapp_mtime=0 ipa_mtime=0 app_mtime=0
  libapp_mtime=0
  if [[ -f "${LIBAPP}" ]]; then
    libapp_mtime="$(file_mtime "${LIBAPP}")"
  fi

  ipa="$(find_ipa || true)"
  if [[ -n "${ipa}" ]]; then
    ipa_mtime="$(file_mtime "${ipa}")"
  fi

  app="$(find_release_app || true)"
  if [[ -n "${app}" ]]; then
    app_mtime="$(file_mtime "${app}/Vericonomy Wallet")"
  fi

  if [[ -n "${ipa}" && "${ipa_mtime}" -ge "${libapp_mtime}" ]]; then
    echo "ipa:${ipa}"
    return 0
  fi

  if [[ -n "${app}" && "${app_mtime}" -ge "${libapp_mtime}" ]]; then
    echo "app:${app}"
    return 0
  fi

  if [[ -n "${ipa}" ]]; then
    echo "warning: IPA is older than libapp.a — it would reinstall a stale UI." >&2
    echo "warning: $(ls -la "${ipa}")" >&2
    if [[ -f "${LIBAPP}" ]]; then
      echo "warning: $(ls -la "${LIBAPP}")" >&2
    fi
  fi

  if [[ -n "${app}" ]]; then
    echo "warning: using fresher Xcode .app instead of stale IPA." >&2
    echo "app:${app}"
    return 0
  fi

  if [[ -n "${ipa}" ]]; then
    echo "ipa:${ipa}"
    return 0
  fi

  return 1
}

INSTALL_TARGET="$(resolve_install_path || true)"
if [[ -z "${INSTALL_TARGET}" ]]; then
  echo "error: no installable .ipa or .app found." >&2
  echo "Run ./scripts/build-ios.sh first (with APPLE_DEVELOPMENT_TEAM set for device builds)." >&2
  exit 1
fi

INSTALL_KIND="${INSTALL_TARGET%%:*}"
INSTALL_PATH="${INSTALL_TARGET#*:}"

resolve_device_id() {
  local name="$1"
  local json
  json="$(mktemp)"
  if "${WITH_XCODE}" xcrun devicectl list devices --json-output "${json}" >/dev/null 2>&1; then
    local id
    id="$(python3 - "${name}" "${json}" <<'PY'
import json, sys
name = sys.argv[1]
path = sys.argv[2]
with open(path, encoding="utf-8") as f:
    data = json.load(f)
for d in data.get("result", {}).get("devices", []):
    if d.get("deviceProperties", {}).get("name") == name:
        print(d.get("identifier", ""))
        break
PY
)"
    rm -f "${json}"
    if [[ -n "${id}" ]]; then
      echo "${id}"
      return 0
    fi
  fi
  rm -f "${json}"

  "${WITH_XCODE}" xcrun devicectl list devices 2>/dev/null \
    | awk -v name="${name}" '$1 == name { print $3; exit }'
}

if [[ -z "${DEVICE_ID}" ]]; then
  DEVICE_ID="$(resolve_device_id "${DEVICE_NAME}" || true)"
fi

if [[ -z "${DEVICE_ID}" ]]; then
  echo "error: could not resolve device id for '${DEVICE_NAME}'." >&2
  echo "Connect your iPhone, unlock it, trust this Mac, then retry." >&2
  echo "Or set IOS_DEVICE_ID to the UUID from:" >&2
  echo "  ./scripts/with-xcode.sh xcrun devicectl list devices" >&2
  "${WITH_XCODE}" xcrun devicectl list devices 2>&1 || true
  exit 1
fi

echo "Installing ${INSTALL_KIND} ${INSTALL_PATH} to ${DEVICE_NAME} (${DEVICE_ID})…"
"${WITH_XCODE}" xcrun devicectl device install app \
  --device "${DEVICE_ID}" \
  "${INSTALL_PATH}"

echo "Launching ${BUNDLE_ID}…"
"${WITH_XCODE}" xcrun devicectl device process launch \
  --device "${DEVICE_ID}" \
  "${BUNDLE_ID}"

echo "Done. Force-quit the app on the phone if the UI still looks stale, then relaunch."
