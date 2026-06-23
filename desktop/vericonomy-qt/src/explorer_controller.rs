//! Explorer HTTP feed controller.

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
        #[qproperty(QString, statsJson)]
        #[qproperty(QString, blocksJson)]
        #[qproperty(QString, lastMessage)]
        #[qproperty(bool, loading)]
        type ExplorerController = super::ExplorerControllerRust;

        #[qsignal]
        fn explorerRefreshed(self: Pin<&mut ExplorerController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut ExplorerController>);
    }

    impl cxx_qt::Threading for ExplorerController {}
}

pub struct ExplorerControllerRust {
    coin: QString,
    statsJson: QString,
    blocksJson: QString,
    lastMessage: QString,
    loading: bool,
    refresh_coin: String,
    refresh_ready: bool,
}

impl Default for ExplorerControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            statsJson: QString::from("{}"),
            blocksJson: QString::from("[]"),
            lastMessage: QString::default(),
            loading: false,
            refresh_coin: String::new(),
            refresh_ready: false,
        }
    }
}

impl qobject::ExplorerController {
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
            let stats = vericonomy_desktop_host::commands::explorer::fetch_explorer_stats(
                &ctx, coin,
            )
            .await;
            let blocks = vericonomy_desktop_host::commands::explorer::fetch_explorer_blocks(
                &ctx, coin, 10,
            )
            .await;

            let _ = qt_thread.queue(move |mut ctrl| match (stats, blocks) {
                (Ok(s), Ok(b)) => {
                    let sj = serde_json::to_string(&s).unwrap_or_else(|_| "{}".into());
                    let bj = serde_json::to_string(&b).unwrap_or_else(|_| "[]".into());
                    ctrl.as_mut().set_statsJson(QString::from(&sj));
                    ctrl.as_mut().set_blocksJson(QString::from(&bj));
                    ctrl.as_mut().set_loading(false);
                    crate::controller_refresh::mark_poll_ready(&mut Pin::get_mut(ctrl.as_mut().rust_mut()).refresh_ready);
                    ctrl.as_mut().explorerRefreshed(true);
                }
                (Err(e), _) | (_, Err(e)) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().explorerRefreshed(false);
                }
            });
        });
    }
}
