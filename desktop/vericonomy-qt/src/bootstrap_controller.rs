//! Bootstrap import controller — CDN/local zip + progress events.

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
        #[qproperty(bool, loading)]
        #[qproperty(QString, progressJson)]
        #[qproperty(QString, resultJson)]
        #[qproperty(QString, lastMessage)]
        type BootstrapController = super::BootstrapControllerRust;

        #[qsignal]
        fn bootstrapCompleted(self: Pin<&mut BootstrapController>, ok: bool);

        #[qinvokable]
        fn importBootstrap(self: Pin<&mut BootstrapController>, localPath: &QString);

        #[qinvokable]
        fn cancel(self: Pin<&mut BootstrapController>);
    }

    impl cxx_qt::Threading for BootstrapController {}
}

pub struct BootstrapControllerRust {
    coin: QString,
    loading: bool,
    progressJson: QString,
    resultJson: QString,
    lastMessage: QString,
}

impl Default for BootstrapControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            loading: false,
            progressJson: QString::from("{}"),
            resultJson: QString::from("{}"),
            lastMessage: QString::default(),
        }
    }
}

fn coin_id(coin: &QString) -> vericonomy_desktop_host::CoinId {
    vericonomy_desktop_host::CoinId::parse(&coin.to_string())
        .unwrap_or(vericonomy_desktop_host::CoinId::Verium)
}

impl qobject::BootstrapController {
    pub fn importBootstrap(self: Pin<&mut Self>, local_path: &QString) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let path_text = local_path.to_string();
        let local = if path_text.trim().is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(path_text))
        };
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();
        let progress_thread = qt_thread.clone();

        this.as_mut().set_loading(true);
        this.as_mut().set_progressJson(QString::from("{}"));
        this.as_mut().set_resultJson(QString::from("{}"));
        this.as_mut().set_lastMessage(QString::default());

        crate::host_events::set_host_event_listener(move |event, payload| {
            if event != vericonomy_desktop_host::bootstrap::PROGRESS_EVENT {
                return;
            }
            let coin_str = payload
                .get("coin")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if coin_str != coin.as_str() {
                return;
            }
            let json = payload.to_string();
            let _ = progress_thread.queue(move |mut ctrl| {
                ctrl.as_mut().set_progressJson(QString::from(&json));
            });
        });

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::bootstrap::import_bootstrap(
                    &ctx,
                    coin,
                    local,
                )
                .await;
            let _ = qt_thread.queue(move |mut ctrl| {
                ctrl.as_mut().set_loading(false);
                match &result {
                    Ok(res) => {
                        let json = serde_json::to_string(res).unwrap_or_else(|_| "{}".into());
                        ctrl.as_mut().set_resultJson(QString::from(&json));
                        ctrl.as_mut()
                            .set_lastMessage(QString::from(&res.message));
                        ctrl.as_mut().bootstrapCompleted(res.success);
                    }
                    Err(e) => {
                        ctrl.as_mut()
                            .set_lastMessage(QString::from(&e.to_string()));
                        ctrl.as_mut().bootstrapCompleted(false);
                    }
                }
            });
        });
    }

    pub fn cancel(self: Pin<&mut Self>) {
        let coin = coin_id(&self.coin());
        vericonomy_desktop_host::commands::bootstrap::cancel_bootstrap(coin);
    }
}
