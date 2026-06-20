#!/usr/bin/env bash
# Build + bundle the Qt desktop wallet into a signed .app / .dmg on macOS.
#
# Uses the validated Cargo-driven cxx-qt build, then macdeployqt to fold the Qt
# frameworks + QML modules into a relocatable bundle.
set -euo pipefail

QT_PREFIX="${QT_PREFIX:-$(brew --prefix qt)}"
export PATH="$QT_PREFIX/bin:$PATH"
export CMAKE_PREFIX_PATH="$QT_PREFIX:${CMAKE_PREFIX_PATH:-}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="Vericonomy Wallet"
BIN_NAME="verium-qt"
DIST="$ROOT/dist"
APP="$DIST/$APP_NAME.app"

echo "==> Building release binary"
( cd "$ROOT" && cargo build --release )

echo "==> Assembling $APP"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$ROOT/target/release/$BIN_NAME" "$APP/Contents/MacOS/$BIN_NAME"
cp "$ROOT/packaging/Info.plist" "$APP/Contents/Info.plist"

echo "==> Fetching daemon sidecars (veriumd / vericoind)"
# Reuse the existing fetch scripts from the Tauri app so both shells ship the
# same daemons. Copies into the bundle Resources for first-run bootstrap.
SIDECAR_SRC="$ROOT/../verium-app/src-tauri/binaries"
if [ -d "$SIDECAR_SRC" ]; then
  cp -R "$SIDECAR_SRC/." "$APP/Contents/Resources/" || true
else
  echo "    (sidecar binaries not found at $SIDECAR_SRC — run the verium-app fetch script first)"
fi

echo "==> Running macdeployqt"
macdeployqt "$APP" \
  -qmldir="$ROOT/qml" \
  ${MACDEPLOY_DMG:+-dmg} \
  ${CODESIGN_IDENTITY:+-codesign="$CODESIGN_IDENTITY"}

echo "==> Done: $APP"
