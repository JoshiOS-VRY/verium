//! First-run setup status controller.

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
        #[qobject]
        #[qml_element]
        #[qproperty(QString, coin)]
        #[qproperty(bool, setupCompleted)]
        #[qproperty(bool, walletReady)]
        #[qproperty(bool, nodeReachable)]
        #[qproperty(bool, hasFullNodeWallet)]
        #[qproperty(bool, hasLightWallet)]
        #[qproperty(bool, isLightMode)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, lastMessage)]
        type SetupController = super::SetupControllerRust;

        #[qsignal]
        fn setupRefreshed(self: Pin<&mut SetupController>, ok: bool);

        #[qsignal]
        fn setupFinished(self: Pin<&mut SetupController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut SetupController>);

        #[qinvokable]
        fn completeSetup(self: Pin<&mut SetupController>);
    }

    impl cxx_qt::Threading for SetupController {}
}

pub struct SetupControllerRust {
    coin: QString,
    setupCompleted: bool,
    walletReady: bool,
    nodeReachable: bool,
    hasFullNodeWallet: bool,
    hasLightWallet: bool,
    isLightMode: bool,
    loading: bool,
    lastMessage: QString,
}

impl Default for SetupControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            setupCompleted: false,
            walletReady: false,
            nodeReachable: false,
            hasFullNodeWallet: false,
            hasLightWallet: false,
            isLightMode: false,
            loading: false,
            lastMessage: QString::default(),
        }
    }
}

impl qobject::SetupController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = vericonomy_desktop_host::CoinId::parse(&this.coin().to_string())
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::setup::get_setup_status(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(status) => {
                    ctrl.as_mut().set_setupCompleted(status.setup_completed);
                    ctrl.as_mut().set_walletReady(status.wallet_ready);
                    ctrl.as_mut().set_nodeReachable(status.node_reachable);
                    ctrl.as_mut()
                        .set_hasFullNodeWallet(status.has_full_node_wallet);
                    ctrl.as_mut().set_hasLightWallet(status.has_light_wallet);
                    ctrl.as_mut().set_isLightMode(status.is_light_mode);
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().setupRefreshed(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().setupRefreshed(false);
                }
            });
        });
    }

    pub fn completeSetup(self: Pin<&mut Self>) {
        let mut this = self;
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::setup::complete_setup();
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    ctrl.as_mut().set_setupCompleted(true);
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Setup complete"));
                    ctrl.as_mut().setupFinished(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().setupFinished(false);
                }
            });
        });
    }
}
