#!/usr/bin/env bash
# Compile the Tauri static lib for device (arm64 / aarch64-apple-ios) and stage libapp.a
# for Xcode. Do NOT call `tauri ios xcode-script` here — that subcommand expects a
# WebSocket connection to a parent `tauri ios build` process and will panic with
# "Connection refused" when run standalone.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"
TAURI_DIR="${ROOT}/src-tauri"
LIB_SRC="${TAURI_DIR}/target/aarch64-apple-ios/release/libverium_app_lib.a"
LIB_DST_DIR="${TAURI_DIR}/gen/apple/Externals/arm64/release"
LIB_DST="${LIB_DST_DIR}/libapp.a"
CRATE_NAME="vericonomy-wallet"
DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-14.0}"

echo ""
echo "==> Compiling Rust for iOS (aarch64-apple-ios release)…"
echo "    This step can take 5–15 minutes on a full rebuild."
echo "    Watch for 'Compiling …' lines — do not Ctrl+C unless another cargo iOS"
echo "    build is already running (file lock)."
echo ""

if pgrep -fl 'cargo.*(aarch64-apple-ios|apple-ios)' >/dev/null 2>&1; then
  echo "warning: another cargo iOS build appears to be running." >&2
  pgrep -fl 'cargo.*(aarch64-apple-ios|apple-ios)' >&2 || true
  echo "Kill the other build or wait for it to finish, then retry." >&2
fi

"${WITH_XCODE}" bash -c "
  set -euo pipefail
  cd \"${TAURI_DIR}\"
  SDKROOT=\$(xcrun --sdk iphoneos --show-sdk-path)
  export PATH=\"\${PATH}:/opt/homebrew/bin:/usr/local/bin:\${HOME}/.cargo/bin\"
  export SDKROOT
  export IPHONEOS_DEPLOYMENT_TARGET=\"${DEPLOYMENT_TARGET}\"
  export TAURI_ENV_TARGET_TRIPLE=aarch64-apple-ios
  ISYSROOT=\"-isysroot \${SDKROOT}\"
  export CFLAGS_aarch64_apple_ios=\"\${ISYSROOT}\"
  export CXXFLAGS_aarch64_apple_ios=\"\${ISYSROOT}\"
  export OBJC_INCLUDE_PATH_aarch64_apple_ios=\"\${SDKROOT}/usr/include\"
  cargo build --lib --release --target aarch64-apple-ios -p \"${CRATE_NAME}\"
"

if [[ ! -f "${LIB_SRC}" ]]; then
  echo "error: expected static lib at ${LIB_SRC}" >&2
  exit 1
fi

mkdir -p "${LIB_DST_DIR}"
cp -f "${LIB_SRC}" "${LIB_DST}"
echo "==> Staged ${LIB_DST} ($(ls -lh "${LIB_DST}" | awk '{print $5}'))"
