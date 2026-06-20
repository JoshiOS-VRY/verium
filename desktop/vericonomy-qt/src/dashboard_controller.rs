//! Dashboard snapshot controller for DashboardHero parity.

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
        #[qproperty(QString, snapshotJson)]
        #[qproperty(bool, loading)]
        type DashboardController = super::DashboardControllerRust;

        #[qsignal]
        fn dashboardRefreshed(self: Pin<&mut DashboardController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut DashboardController>);
    }

    impl cxx_qt::Threading for DashboardController {}
}

pub struct DashboardControllerRust {
    coin: QString,
    snapshotJson: QString,
    loading: bool,
}

impl Default for DashboardControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            snapshotJson: QString::from("{}"),
            loading: false,
        }
    }
}

impl qobject::DashboardController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = vericonomy_desktop_host::CoinId::parse(&this.coin().to_string())
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::dashboard::get_dashboard_snapshot(&ctx, coin)
                    .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(snap) => {
                    let json = serde_json::to_string(&snap).unwrap_or_else(|_| "{}".into());
                    ctrl.as_mut().set_snapshotJson(QString::from(&json));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().dashboardRefreshed(true);
                }
                Err(_) => {
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().dashboardRefreshed(false);
                }
            });
        });
    }
}
