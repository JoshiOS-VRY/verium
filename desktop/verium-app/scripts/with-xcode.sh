#!/usr/bin/env bash
# Resolve full Xcode (not Command Line Tools) and run a command with DEVELOPER_DIR set.
set -euo pipefail

resolve_xcode_developer_dir() {
  if [[ -n "${DEVELOPER_DIR:-}" && -x "${DEVELOPER_DIR}/usr/bin/xcodebuild" ]]; then
    echo "${DEVELOPER_DIR}"
    return 0
  fi

  local candidates=(
    "/Applications/Xcode.app/Contents/Developer"
    "${HOME}/Downloads/Xcode.app/Contents/Developer"
    "${HOME}/Applications/Xcode.app/Contents/Developer"
  )

  for dir in "${candidates[@]}"; do
    if [[ -x "${dir}/usr/bin/xcodebuild" ]]; then
      echo "${dir}"
      return 0
    fi
  done

  local found
  found="$(mdfind "kMDItemCFBundleIdentifier == 'com.apple.dt.Xcode'" 2>/dev/null | head -1 || true)"
  if [[ -n "${found}" && -x "${found}/Contents/Developer/usr/bin/xcodebuild" ]]; then
    echo "${found}/Contents/Developer"
    return 0
  fi

  return 1
}

if ! DEV_DIR="$(resolve_xcode_developer_dir)"; then
  echo "error: full Xcode is required (not just Command Line Tools)." >&2
  echo "Install Xcode, then either:" >&2
  echo "  sudo xcode-select -s /Applications/Xcode.app/Contents/Developer" >&2
  echo "or keep Xcode in Downloads/Applications and use scripts/with-xcode.sh" >&2
  exit 1
fi

export DEVELOPER_DIR="${DEV_DIR}"
export PATH="${DEVELOPER_DIR}/usr/bin:${PATH}"

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <command> [args...]" >&2
  exit 1
fi

exec "$@"
