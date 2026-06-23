//! Network peers controller.

#![allow(non_snake_case)]

use cxx_qt::{CxxQtType, Threading};
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
        #[qproperty(QString, peersJson)]
        #[qproperty(i32, peerCount)]
        #[qproperty(bool, loading)]
        type NetworkController = super::NetworkControllerRust;

        #[qsignal]
        fn peersRefreshed(self: Pin<&mut NetworkController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut NetworkController>);
    }

    impl cxx_qt::Threading for NetworkController {}
}

pub struct NetworkControllerRust {
    coin: QString,
    peersJson: QString,
    peerCount: i32,
    loading: bool,
    refresh_coin: String,
    refresh_ready: bool,
}

impl Default for NetworkControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            peersJson: QString::from("[]"),
            peerCount: 0,
            loading: false,
            refresh_coin: String::new(),
            refresh_ready: false,
        }
    }
}

impl qobject::NetworkController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        let coin_key = this.coin().to_string();
        let mut rust = this.as_mut().rust_mut();
        let rust = Pin::get_mut(rust);
        let show_loading = crate::controller_refresh::begin_poll_refresh(
            &coin_key,
            &mut rust.refresh_coin,
            &mut rust.refresh_ready,
        );
        if show_loading {
            this.as_mut().set_loading(true);
        }
        let coin = vericonomy_desktop_host::CoinId::parse(&coin_key)
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::network::get_peer_info(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(peers) => {
                    let json = serde_json::to_string(&peers).unwrap_or_else(|_| "[]".into());
                    ctrl.as_mut().set_peerCount(peers.len() as i32);
                    ctrl.as_mut().set_peersJson(QString::from(&json));
                    ctrl.as_mut().set_loading(false);
                    crate::controller_refresh::mark_poll_ready(&mut Pin::get_mut(ctrl.as_mut().rust_mut()).refresh_ready);
                    ctrl.as_mut().peersRefreshed(true);
                }
                Err(_) => {
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().peersRefreshed(false);
                }
            });
        });
    }
}
