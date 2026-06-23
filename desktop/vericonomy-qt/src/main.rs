//! Vericonomy desktop wallet — native Qt 6 / QML entry point (Phase 0 spike).
//!
//! The application shell is created entirely from Rust via `cxx-qt-lib`
//! (`QGuiApplication` + `QQmlApplicationEngine`). There is no hand-written C++
//! and no WebView — which is the security goal driving this port.

mod bootstrap_controller;
mod controller_refresh;
mod host_events;
mod addressbook_controller;
mod app_context;
mod explorer_controller;
mod host_links;
mod logs_controller;
mod pool_miner_controller;
mod pool_stats_controller;
mod mining_controller;
mod network_controller;
mod node_controller;
mod qr_helper;
mod rpc_controller;
mod runtime;
mod settings_controller;
mod sound_controller;
mod wallet_mode_controller;
mod dashboard_controller;
mod light_wallet_controller;
mod security_controller;
mod setup_controller;
mod staking_controller;
mod theme;
mod transactions_controller;
mod wallet_controller;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

extern "C" {
    fn verium_qt_load_fonts();
}

fn main() {
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(_engine) = engine.as_mut() {
        // QML module static init registers Qt resources; then load bundled Inter.
        unsafe {
            verium_qt_load_fonts();
        }
    }

    if let Some(engine) = engine.as_mut() {
        // Resource path produced by the QmlModule (uri com.vericonomy.verium).
        engine.load(&QUrl::from(&QString::from(
            "qrc:/qt/qml/com/vericonomy/verium/qml/main.qml",
        )));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
