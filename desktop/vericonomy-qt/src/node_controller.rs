//! `NodeController` QObject — node/blockchain status for the dashboard.
//!
//! End-to-end pattern used by every controller in this app:
//!   QML  --(Q_INVOKABLE refresh)-->  Rust
//!   Rust --(tokio async work off the GUI thread)-->  host command
//!   Rust --(CxxQtThread::queue back onto GUI thread)--> set Q_PROPERTYs + emit signal
//!   QML  --(property bindings + onStatusRefreshed)--> live UI update
//!
//! Calls the real `vericonomy-desktop-host` `get_node_status` command over the
//! shared `AppContext`.

// cxx-qt 0.7 uses the Rust field/fn names verbatim as the QML-facing names, so
// we use camelCase here to keep QML idiomatic (node.verificationProgress, etc.).
// This is the naming convention for every Phase 2 controller.
#![allow(non_snake_case)]

use cxx_qt::Threading;
use cxx_qt_lib::QString;
use std::pin::Pin;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        /// Exposed to QML as `NodeController` in module `com.vericonomy.verium`.
        #[qobject]
        #[qml_element]
        /// Active coin: "verium" or "vericoin".
        #[qproperty(QString, coin)]
        #[qproperty(QString, chain)]
        #[qproperty(i32, blocks)]
        #[qproperty(i32, headers)]
        #[qproperty(f64, verificationProgress)]
        #[qproperty(i32, connections)]
        #[qproperty(bool, connected)]
        #[qproperty(bool, warmingUp)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, stateLabel)]
        type NodeController = super::NodeControllerRust;

        /// Emitted after every refresh attempt. `ok = false` signals an error.
        #[qsignal]
        fn statusRefreshed(self: Pin<&mut NodeController>, ok: bool);

        /// Called from QML to (re)load node status asynchronously.
        #[qinvokable]
        fn refresh(self: Pin<&mut NodeController>);
    }

    // Opt in to background-thread -> GUI-thread marshalling via `qt_thread()`.
    impl cxx_qt::Threading for NodeController {}
}

/// Backing data for [`qobject::NodeController`].
pub struct NodeControllerRust {
    coin: QString,
    chain: QString,
    blocks: i32,
    headers: i32,
    verificationProgress: f64,
    connections: i32,
    connected: bool,
    warmingUp: bool,
    loading: bool,
    stateLabel: QString,
}

impl Default for NodeControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            chain: QString::default(),
            blocks: 0,
            headers: 0,
            verificationProgress: 0.0,
            connections: 0,
            connected: false,
            warmingUp: false,
            loading: false,
            stateLabel: QString::from("…"),
        }
    }
}

impl qobject::NodeController {
    /// Q_INVOKABLE entry point. Returns immediately; the UI updates when the
    /// async task queues its result back onto the GUI thread.
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);

        let coin = vericonomy_desktop_host::CoinId::parse(&this.coin().to_string())
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let ctx = crate::app_context::context();

        // Handle usable from the worker thread to hop back to the GUI thread.
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::node::get_node_status(&ctx, coin).await;

            let _ = qt_thread.queue(move |mut ctrl| {
                match result {
                    Ok(status) => {
                        ctrl.as_mut().set_chain(QString::from(&status.chain));
                        ctrl.as_mut().set_blocks(status.blocks as i32);
                        ctrl.as_mut().set_headers(status.headers as i32);
                        ctrl.as_mut()
                            .set_verificationProgress(status.verification_progress);
                        ctrl.as_mut().set_connections(status.connections as i32);
                        ctrl.as_mut().set_connected(status.connected);
                        ctrl.as_mut().set_warmingUp(status.warming_up);
                        ctrl.as_mut().set_stateLabel(QString::from(&status.state));
                        ctrl.as_mut().set_loading(false);
                        ctrl.as_mut().statusRefreshed(true);
                    }
                    Err(_) => {
                        ctrl.as_mut().set_connected(false);
                        ctrl.as_mut().set_stateLabel(QString::from("Error"));
                        ctrl.as_mut().set_loading(false);
                        ctrl.as_mut().statusRefreshed(false);
                    }
                }
            });
        });
    }
}
