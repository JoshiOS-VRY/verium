#!/usr/bin/env bash
# Shared AppIcon fingerprint + Xcode asset-catalog cache busting for iOS builds.

ios_icon_root() {
  local root="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
  printf '%s\n' "${root}"
}

ios_icon_stamp_path() {
  local root
  root="$(ios_icon_root "${1:-}")"
  printf '%s/src-tauri/gen/apple/.ios-icon-stamp\n' "${root}"
}

ios_icon_archived_stamp_path() {
  local root
  root="$(ios_icon_root "${1:-}")"
  printf '%s/src-tauri/gen/apple/.ios-icon-archived-stamp\n' "${root}"
}

ios_icon_marketing_png() {
  local root
  root="$(ios_icon_root "${1:-}")"
  printf '%s/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png\n' "${root}"
}

ios_icon_fingerprint() {
  local root="${1:-}"
  local marketing
  marketing="$(ios_icon_marketing_png "${root}")"
  if [[ ! -f "${marketing}" ]]; then
    echo "missing"
    return 1
  fi
  shasum -a 256 "${marketing}" | awk '{print $1}'
}

ios_icon_read_stamp() {
  local stamp
  stamp="$(ios_icon_stamp_path "${1:-}")"
  if [[ -f "${stamp}" ]]; then
    cat "${stamp}"
  fi
}

ios_icon_write_stamp() {
  local root="${1:-}"
  local stamp
  stamp="$(ios_icon_stamp_path "${root}")"
  ios_icon_fingerprint "${root}" > "${stamp}"
}

ios_icon_mark_archived() {
  local root="${1:-}"
  local stamp archived
  stamp="$(ios_icon_stamp_path "${root}")"
  archived="$(ios_icon_archived_stamp_path "${root}")"
  if [[ ! -f "${stamp}" ]]; then
    ios_icon_write_stamp "${root}"
  fi
  cp "${stamp}" "${archived}"
}

ios_icon_archived_matches_current() {
  local root="${1:-}"
  local current archived
  current="$(ios_icon_read_stamp "${root}")"
  archived="$(cat "$(ios_icon_archived_stamp_path "${root}")" 2>/dev/null || true)"
  [[ -n "${current}" && "${current}" == "${archived}" ]]
}

ios_icon_clear_xcode_asset_cache() {
  local root="${1:-}"
  root="$(ios_icon_root "${root}")"
  local apple_dir="${root}/src-tauri/gen/apple"
  echo "==> Clearing Xcode DerivedData and stale iOS archives (AppIcon asset catalog cache)…"
  rm -rf \
    "${HOME}/Library/Developer/Xcode/DerivedData/vericonomy-wallet-"* \
    "${apple_dir}/build/vericonomy-wallet_iOS.xcarchive" \
    "${apple_dir}/build/app-store" \
    "${apple_dir}/build/arm64" \
    "${apple_dir}/build/arm64-sim" \
    "${apple_dir}/build/aarch64-sim" \
    2>/dev/null || true
}

ios_icon_xcode_clean() {
  local root="${1:-}"
  local with_xcode="${2:-}"
  root="$(ios_icon_root "${root}")"
  local project="${root}/src-tauri/gen/apple/vericonomy-wallet.xcodeproj"
  local scheme="${3:-vericonomy-wallet_iOS}"
  if [[ ! -d "${project}" ]]; then
    return 0
  fi
  echo "==> xcodebuild clean (force AppIcon asset catalog recompile)…"
  "${with_xcode}" xcodebuild \
    -project "${project}" \
    -scheme "${scheme}" \
    -configuration release \
    clean
}

ios_icon_invalidate_if_needed() {
  local root="${1:-}"
  local with_xcode="${2:-}"
  root="$(ios_icon_root "${root}")"

  if [[ "${IOS_FORCE_ICON_REBUILD:-}" == "1" ]] || ! ios_icon_archived_matches_current "${root}"; then
    ios_icon_clear_xcode_asset_cache "${root}"
    if [[ -n "${with_xcode}" ]]; then
      ios_icon_xcode_clean "${root}" "${with_xcode}"
    fi
    return 0
  fi

  echo "==> AppIcon fingerprint unchanged — skipping DerivedData wipe."
}
