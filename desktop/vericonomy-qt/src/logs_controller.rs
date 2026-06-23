//! Logs controller — tails daemon debug.log.

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
        #[qproperty(QString, linesJson)]
        #[qproperty(bool, loading)]
        type LogsController = super::LogsControllerRust;

        #[qsignal]
        fn logsRefreshed(self: Pin<&mut LogsController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut LogsController>);
    }

    impl cxx_qt::Threading for LogsController {}
}

pub struct LogsControllerRust {
    coin: QString,
    linesJson: QString,
    loading: bool,
    refresh_coin: String,
    refresh_ready: bool,
}

impl Default for LogsControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            linesJson: QString::from("[]"),
            loading: false,
            refresh_coin: String::new(),
            refresh_ready: false,
        }
    }
}

impl qobject::LogsController {
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
                vericonomy_desktop_host::commands::diagnostics::tail_logs(&ctx, coin, 200).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(lines) => {
                    let json = serde_json::to_string(&lines).unwrap_or_else(|_| "[]".into());
                    ctrl.as_mut().set_linesJson(QString::from(&json));
                    ctrl.as_mut().set_loading(false);
                    crate::controller_refresh::mark_poll_ready(&mut Pin::get_mut(ctrl.as_mut().rust_mut()).refresh_ready);
                    ctrl.as_mut().logsRefreshed(true);
                }
                Err(_) => {
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().logsRefreshed(false);
                }
            });
        });
    }
}
