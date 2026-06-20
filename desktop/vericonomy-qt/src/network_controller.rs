//! Network peers controller.

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
}

impl Default for NetworkControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            peersJson: QString::from("[]"),
            peerCount: 0,
            loading: false,
        }
    }
}

impl qobject::NetworkController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = vericonomy_desktop_host::CoinId::parse(&this.coin().to_string())
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
