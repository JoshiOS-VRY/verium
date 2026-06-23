//! Public pool stats feed for the Mining page.

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
        #[qproperty(QString, statsJson)]
        #[qproperty(QString, lastMessage)]
        #[qproperty(bool, loading)]
        type PoolStatsController = super::PoolStatsControllerRust;

        #[qsignal]
        fn poolStatsRefreshed(self: Pin<&mut PoolStatsController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut PoolStatsController>);
    }

    impl cxx_qt::Threading for PoolStatsController {}
}

pub struct PoolStatsControllerRust {
    statsJson: QString,
    lastMessage: QString,
    loading: bool,
    refresh_ready: bool,
}

impl Default for PoolStatsControllerRust {
    fn default() -> Self {
        Self {
            statsJson: QString::from("{}"),
            lastMessage: QString::default(),
            loading: false,
            refresh_ready: false,
        }
    }
}

impl qobject::PoolStatsController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        let show_loading = !Pin::get_mut(this.as_mut().rust_mut()).refresh_ready;
        if show_loading {
            this.as_mut().set_loading(true);
        }
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::pool_stats::fetch_pool_stats(&ctx).await;

            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(stats) => {
                    let sj = serde_json::to_string(&stats).unwrap_or_else(|_| "{}".into());
                    ctrl.as_mut().set_statsJson(QString::from(&sj));
                    ctrl.as_mut().set_lastMessage(QString::default());
                    ctrl.as_mut().set_loading(false);
                    crate::controller_refresh::mark_poll_ready(&mut Pin::get_mut(ctrl.as_mut().rust_mut()).refresh_ready);
                    ctrl.as_mut().poolStatsRefreshed(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().poolStatsRefreshed(false);
                }
            });
        });
    }
}
