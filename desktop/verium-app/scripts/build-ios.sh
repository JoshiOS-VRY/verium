#!/usr/bin/env bash
# Build + install the Vericonomy iOS light wallet on a connected device.
# One command: npm run ios:deploy  (or npm run ios:ship)
# Optional: FORCE_RUST_REBUILD=1 to re-embed frontend in Rust without deleting stamps manually.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"
BUILD_DIR="${ROOT}/src-tauri/gen/apple/build"
LIBAPP="${ROOT}/src-tauri/gen/apple/Externals/arm64/release/libapp.a"

if [[ -n "${APPLE_DEVELOPMENT_TEAM:-}" ]]; then
  IOS_BUILD_MODE=device
else
  IOS_BUILD_MODE=simulator
  echo ""
  echo "warning: APPLE_DEVELOPMENT_TEAM is not set." >&2
  echo "  ios:ship / ios:deploy will build for the Simulator only (not your phone)." >&2
  echo "  To install on a connected device (e.g. JQT):" >&2
  echo "    export APPLE_DEVELOPMENT_TEAM=your-team-id" >&2
  echo "    npm run ios:ship" >&2
  echo ""
fi

STAMP_FILE="${ROOT}/src-tauri/gen/apple/.frontend-build-stamp-${IOS_BUILD_MODE}"

cd "${ROOT}"

if [[ "${FORCE_RUST_REBUILD:-}" == "1" ]]; then
  echo "==> FORCE_RUST_REBUILD=1 — clearing Rust iOS stamp and libapp.a"
  rm -f "${STAMP_FILE}" "${LIBAPP}" 2>/dev/null || true
fi

frontend_fingerprint() {
  if [[ ! -f "${ROOT}/dist/index.html" ]]; then
    return 1
  fi
  {
    shasum -a 256 "${ROOT}/dist/index.html"
    shasum -a 256 "${ROOT}/dist/assets/"index-*.js 2>/dev/null || true
  } | shasum -a 256 | awk '{print $1}'
}

run_rust_ios_compile() {
  if [[ "${IOS_BUILD_MODE}" == "device" ]]; then
    bash "${ROOT}/scripts/compile-ios-rust.sh"
  else
    bash "${ROOT}/scripts/compile-ios-rust-sim.sh"
  fi
}

echo "==> [1/4] Building frontend (no Prettier — use npm run build for a formatted CI build)…"
npm run build:app

CURRENT_FP="$(frontend_fingerprint || echo missing)"
CURRENT_STAMP="${CURRENT_FP} ${IOS_BUILD_MODE}"
PREVIOUS_STAMP="$(cat "${STAMP_FILE}" 2>/dev/null || echo "")"
NEED_RUST_REBUILD=0

if [[ ! -f "${LIBAPP}" ]]; then
  echo "==> No libapp.a yet — Rust iOS compile required (${IOS_BUILD_MODE})."
  NEED_RUST_REBUILD=1
elif [[ "${CURRENT_STAMP}" != "${PREVIOUS_STAMP}" ]]; then
  if [[ "${CURRENT_FP}" != "${PREVIOUS_STAMP%% *}" ]]; then
    echo "==> Frontend or build target changed — Rust iOS compile required (${IOS_BUILD_MODE})."
  else
    echo "==> Switching iOS target (${IOS_BUILD_MODE}) — Rust recompile required."
  fi
  NEED_RUST_REBUILD=1
else
  echo "==> Frontend unchanged for ${IOS_BUILD_MODE} — skipping full Rust rebuild (faster deploy)."
fi

if [[ "${NEED_RUST_REBUILD}" -eq 1 ]]; then
  rm -rf \
    "${ROOT}/src-tauri/target/aarch64-apple-ios"/*/build/vericonomy-wallet-*/out/tauri-codegen-assets \
    "${ROOT}/src-tauri/target/aarch64-apple-ios-sim"/*/build/vericonomy-wallet-*/out/tauri-codegen-assets \
    "${ROOT}/src-tauri/gen/apple/Externals/arm64/debug/libapp.a" \
    "${LIBAPP}" 2>/dev/null || true
  printf '%s\n' "${CURRENT_STAMP}" > "${STAMP_FILE}"
  run_rust_ios_compile
elif [[ ! -f "${LIBAPP}" ]]; then
  echo "error: libapp.a missing after compile step." >&2
  exit 1
fi

if [[ -d "${ROOT}/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset" ]]; then
  ICON_SRC="${ROOT}/src-tauri/icons/ios"
  ICON_DST="${ROOT}/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset"
  if [[ -d "${ICON_SRC}" ]]; then
    cp -f "${ICON_SRC}"/*.png "${ICON_DST}/"
  fi
fi

# Stale IPA under build/arm64 caused installs to push old UI; always clear export output.
if [[ -d "${BUILD_DIR}" ]]; then
  rm -rf \
    "${BUILD_DIR}/arm64" \
    "${BUILD_DIR}/arm64-sim" \
    "${BUILD_DIR}/aarch64-sim" \
    "${BUILD_DIR}/vericonomy-wallet_iOS.xcarchive" \
    "${BUILD_DIR}/Packaging.log" \
    "${BUILD_DIR}/ExportOptions.plist" \
    "${BUILD_DIR}/DistributionSummary.plist"
fi

if [[ "${IOS_CLEAN_DERIVED_DATA:-}" == "1" ]]; then
  echo "Clearing Xcode DerivedData for vericonomy-wallet…"
  rm -rf "${HOME}/Library/Developer/Xcode/DerivedData/vericonomy-wallet-"*
fi

echo ""
echo "==> [2/4] Xcode release build (uses prebuilt libapp.a — no tauri ios build hang)…"

if [[ "${IOS_BUILD_MODE}" == "simulator" ]]; then
  "${WITH_XCODE}" npm run tauri:ios:build:sim:inner
else
  bash "${ROOT}/scripts/xcode-build-ios-device.sh"
fi

echo ""
echo "==> [3/4] Build artifacts"
echo "iOS build finished."

IPA="${BUILD_DIR}/arm64/Vericonomy Wallet.ipa"
if [[ -f "${LIBAPP}" ]]; then
  echo "libapp.a: $(ls -la "${LIBAPP}")"
  if [[ -f "${IPA}" ]]; then
    echo "IPA:      $(ls -la "${IPA}")"
    if [[ "${IPA}" -ot "${LIBAPP}" ]]; then
      echo "warning: IPA is older than libapp.a — install will use the fresher Xcode .app." >&2
    fi
  else
    echo "note: no IPA at ${IPA} — install will use Xcode .app from DerivedData." >&2
  fi
  if [[ -f "${ROOT}/dist/index.html" ]]; then
    echo "frontend: $(head -1 "${ROOT}/dist/index.html")"
  fi
fi

if [[ -n "${APPLE_DEVELOPMENT_TEAM:-}" && "${SKIP_IOS_INSTALL:-}" != "1" ]]; then
  echo ""
  echo "==> [4/4] Installing to connected device (${IOS_DEVICE_NAME:-JQT})…"
  bash "${ROOT}/scripts/install-ios-device.sh" "${IOS_DEVICE_NAME:-JQT}"
  echo ""
  echo "All done — latest build is on your phone."
else
  echo ""
  if [[ -z "${APPLE_DEVELOPMENT_TEAM:-}" ]]; then
    echo "Simulator build only. For device build + install:"
    echo "  export APPLE_DEVELOPMENT_TEAM=your-team-id && npm run ios:deploy"
  else
    echo "Install skipped (SKIP_IOS_INSTALL=1). Run: npm run ios:install"
  fi
fi
