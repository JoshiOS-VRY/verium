//! Explorer HTTP feed controller.

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
}

impl Default for ExplorerControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            statsJson: QString::from("{}"),
            blocksJson: QString::from("[]"),
            lastMessage: QString::default(),
            loading: false,
        }
    }
}

impl qobject::ExplorerController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = vericonomy_desktop_host::CoinId::parse(&this.coin().to_string())
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
