# Packaging — Qt desktop wallet

The release build is **Cargo-driven** (cxx-qt-build compiles the QObjects + QML
and links Qt); the OS deploy tools then fold in the Qt runtime, and an installer
wraps the result. Daemon sidecars (`veriumd` / `vericoind`) are reused from the
Tauri app so both shells ship identical daemons.

## macOS

```bash
# optional: CODESIGN_IDENTITY="Developer ID Application: …"  MACDEPLOY_DMG=1
./packaging/build-macos.sh
```

Produces `dist/Vericonomy Wallet.app` (and a `.dmg` when `MACDEPLOY_DMG=1`),
with `macdeployqt -qmldir=qml` bundling Qt frameworks + the QML module.
Notarization: `xcrun notarytool submit dist/*.dmg …`.

## Windows

```powershell
$env:QT_PREFIX = "C:\Qt\6.11.1\msvc2022_64"
.\packaging\build-windows.ps1
```

Runs `windeployqt --qmldir qml`, copies sidecars, then `makensis installer.nsi`
→ `dist/VericonomyWalletSetup.exe`. Sign with `signtool`.

## Linux

```bash
cargo build --release
# Bundle with linuxdeploy + the qt plugin, or ship a Flatpak/AppImage:
linuxdeploy --appdir AppDir -e target/release/verium-qt \
            --plugin qt --output appimage
```

## CMake / Corrosion (IDE + CI)

`CMakeLists.txt` wraps the Cargo build via Corrosion for CMake-based IDEs and
pipelines; the produced `verium-qt` target feeds `install()` / CPack.

## Auto-update + signing

- **macOS:** Sparkle (appcast) or the Qt updater; sign + notarize each build.
- **Windows:** NSIS + signed installer; updater checks an appcast feed.
- Update manifests/signatures are verified by the host layer
  (`installer_verify` is promoted from the Tauri app), so the WebView-era
  download path is gone.

## Fonts

Bundle **Inter** (`.ttf`) as a Qt resource and register it at startup so the UI
matches the web build exactly (the dev machine currently falls back to a system
sans). Add the font to the QML module's `qrc_files` and load via
`QFontDatabase::addApplicationFont`.
