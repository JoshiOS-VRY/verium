//! Logs controller — tails daemon debug.log.

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
}

impl Default for LogsControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            linesJson: QString::from("[]"),
            loading: false,
        }
    }
}

impl qobject::LogsController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = vericonomy_desktop_host::CoinId::parse(&this.coin().to_string())
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
