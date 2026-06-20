# Vericonomy Wallet — Qt / QML desktop (port in progress)

Native **Qt 6 / QML** desktop frontend for the Vericonomy wallet, replacing the
Tauri + React/Tailwind shell. The UI talks to the shared Rust core
([`vericonomy-sdk`](../../../vericonomy-sdk)) through a **cxx-qt** Qt↔Rust bridge.
There is no embedded browser engine (the security driver for this port) and no
hand-written C++ — the application shell is created from Rust via `cxx-qt-lib`.

See the migration plan: `Qt QML Wallet Port`.

## Status

- **Phase 0 (spike): in progress** — `NodeController` proves the
  QML → Rust (Q_INVOKABLE) → async tokio work → Qt-thread marshalling →
  Q_PROPERTY + signal → QML binding loop, plus a first taste of the design
  tokens ported from `desktop/verium-app/src/index.css`.

## Layout

```
verium-qt/
  Cargo.toml            # cxx-qt bin crate (Cargo-driven build)
  build.rs              # registers the QML module + Rust QObjects
  src/
    main.rs             # QGuiApplication + QQmlApplicationEngine (Rust)
    runtime.rs          # shared tokio runtime (mirrors vericonomy-ffi)
    node_controller.rs  # cxx-qt bridge: NodeController QObject
  qml/
    main.qml            # spike status card (design tokens preview)
```

## Prerequisites

- Rust (stable, matches workspace `rust-version`)
- Qt 6 (Core, Gui, Qml, Quick, QuickControls2)
  - macOS: `brew install qt`
  - Linux: distro Qt 6 dev packages
  - Windows: official Qt 6 installer or `vcpkg`
- `cxx-qt-build` locates Qt via `qmake`/`Qt6_DIR` on `PATH`/`CMAKE_PREFIX_PATH`.

## Run (Cargo-driven)

```bash
# Ensure Qt's bin (qmake) is discoverable, e.g. on Apple Silicon:
export PATH="$(brew --prefix qt)/bin:$PATH"

cargo run -p verium-qt
```

## Packaging (Phase 5)

CMake + Corrosion drives release builds so `macdeployqt` / `windeployqt` and the
existing installer scripts apply. That target is added in Phase 5; Phase 0 uses
the Cargo-driven build for fast iteration.
