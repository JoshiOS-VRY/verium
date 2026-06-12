#!/usr/bin/env bash
# Compile the Tauri static lib for iOS Simulator (aarch64-apple-ios-sim) and stage libapp.a.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WITH_XCODE="${ROOT}/scripts/with-xcode.sh"
TAURI_DIR="${ROOT}/src-tauri"
LIB_SRC="${TAURI_DIR}/target/aarch64-apple-ios-sim/release/libverium_app_lib.a"
LIB_DST_DIR="${TAURI_DIR}/gen/apple/Externals/arm64/release"
LIB_DST="${LIB_DST_DIR}/libapp.a"
CRATE_NAME="vericonomy-wallet"
DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-14.0}"

echo ""
echo "==> Compiling Rust for iOS Simulator (aarch64-apple-ios-sim release)…"
echo "    This step can take 5–15 minutes on a full rebuild."
echo ""

"${WITH_XCODE}" bash -c "
  set -euo pipefail
  cd \"${TAURI_DIR}\"
  SDKROOT=\$(xcrun --sdk iphonesimulator --show-sdk-path)
  export PATH=\"\${PATH}:/opt/homebrew/bin:/usr/local/bin:\${HOME}/.cargo/bin\"
  export SDKROOT
  export IPHONEOS_DEPLOYMENT_TARGET=\"${DEPLOYMENT_TARGET}\"
  export TAURI_ENV_TARGET_TRIPLE=aarch64-apple-ios-sim
  ISYSROOT=\"-isysroot \${SDKROOT}\"
  export CFLAGS_aarch64_apple_ios_sim=\"\${ISYSROOT}\"
  export CXXFLAGS_aarch64_apple_ios_sim=\"\${ISYSROOT}\"
  export OBJC_INCLUDE_PATH_aarch64_apple_ios_sim=\"\${SDKROOT}/usr/include\"
  cargo build --lib --release --target aarch64-apple-ios-sim -p \"${CRATE_NAME}\"
"

if [[ ! -f "${LIB_SRC}" ]]; then
  echo "error: expected static lib at ${LIB_SRC}" >&2
  exit 1
fi

mkdir -p "${LIB_DST_DIR}"
cp -f "${LIB_SRC}" "${LIB_DST}"
echo "==> Staged simulator libapp.a at ${LIB_DST} ($(ls -lh "${LIB_DST}" | awk '{print $5}'))"
