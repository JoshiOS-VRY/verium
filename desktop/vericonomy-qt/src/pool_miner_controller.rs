//! Pool miner sidecar controls (Verium pool stratum).

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
        #[qproperty(bool, running)]
        #[qproperty(bool, sidecarFound)]
        #[qproperty(bool, poolConnected)]
        #[qproperty(f64, hashrateHm)]
        #[qproperty(i32, activeThreads)]
        #[qproperty(i32, maxThreads)]
        #[qproperty(i32, suggestedThreads)]
        #[qproperty(QString, worker)]
        #[qproperty(QString, backend)]
        #[qproperty(QString, connectionState)]
        #[qproperty(QString, lastMessage)]
        type PoolMinerController = super::PoolMinerControllerRust;

        #[qsignal]
        fn poolRefreshed(self: Pin<&mut PoolMinerController>, ok: bool);

        #[qsignal]
        fn poolStateChanged(self: Pin<&mut PoolMinerController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut PoolMinerController>);

        #[qinvokable]
        fn detect(self: Pin<&mut PoolMinerController>);

        #[qinvokable]
        fn startPool(
            self: Pin<&mut PoolMinerController>,
            stratumUrl: &QString,
            username: &QString,
            password: &QString,
            threads: i32,
        );

        #[qinvokable]
        fn stopPool(self: Pin<&mut PoolMinerController>);
    }

    impl cxx_qt::Threading for PoolMinerController {}
}

pub struct PoolMinerControllerRust {
    coin: QString,
    loading: bool,
    running: bool,
    sidecarFound: bool,
    poolConnected: bool,
    hashrateHm: f64,
    activeThreads: i32,
    maxThreads: i32,
    suggestedThreads: i32,
    worker: QString,
    backend: QString,
    connectionState: QString,
    lastMessage: QString,
}

impl Default for PoolMinerControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            loading: false,
            running: false,
            sidecarFound: false,
            poolConnected: false,
            hashrateHm: 0.0,
            activeThreads: 0,
            maxThreads: 4,
            suggestedThreads: 2,
            worker: QString::default(),
            backend: QString::default(),
            connectionState: QString::from("stopped"),
            lastMessage: QString::default(),
        }
    }
}

fn coin_id(coin: &QString) -> vericonomy_desktop_host::CoinId {
    vericonomy_desktop_host::CoinId::parse(&coin.to_string())
        .unwrap_or(vericonomy_desktop_host::CoinId::Verium)
}

impl qobject::PoolMinerController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = coin_id(&this.coin());
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::pool::pool_miner_status(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(st) => {
                    ctrl.as_mut().set_running(st.running);
                    ctrl.as_mut().set_hashrateHm(st.hashrate_hm);
                    ctrl.as_mut().set_activeThreads(st.active_threads as i32);
                    ctrl.as_mut().set_worker(QString::from(&st.worker));
                    ctrl.as_mut().set_backend(QString::from(&st.backend));
                    ctrl.as_mut().set_poolConnected(st.pool_connected);
                    ctrl.as_mut()
                        .set_connectionState(QString::from(&st.connection_state));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().poolRefreshed(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().poolRefreshed(false);
                }
            });
        });
    }

    pub fn detect(self: Pin<&mut Self>) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let detect =
                vericonomy_desktop_host::commands::pool::pool_miner_detect(&ctx, coin).await;
            let limits =
                vericonomy_desktop_host::commands::pool::pool_miner_memory_limits(&ctx, coin)
                    .await;
            let _ = qt_thread.queue(move |mut ctrl| {
                if let Ok(d) = detect {
                    ctrl.as_mut().set_sidecarFound(d.sidecar_found);
                    if let Some(path) = d.path {
                        ctrl.as_mut()
                            .set_backend(QString::from(&format!("{} ({})", d.source, path)));
                    } else {
                        ctrl.as_mut().set_backend(QString::from(&d.source));
                    }
                }
                if let Ok(l) = limits {
                    ctrl.as_mut().set_maxThreads(l.max_manual_threads as i32);
                    ctrl.as_mut()
                        .set_suggestedThreads(l.max_safe_threads as i32);
                }
            });
        });
    }

    pub fn startPool(
        self: Pin<&mut Self>,
        stratumUrl: &QString,
        username: &QString,
        password: &QString,
        threads: i32,
    ) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();
        let config = vericonomy_desktop_host::commands::pool::PoolMinerStartConfig {
            stratum_url: stratumUrl.to_string(),
            username: username.to_string(),
            password: if password.is_empty() {
                None
            } else {
                Some(password.to_string())
            },
            threads: threads.max(1) as u32,
            backup_url: None,
        };

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::pool::pool_miner_start(&ctx, coin, config)
                    .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    ctrl.as_mut().set_running(true);
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Pool miner started"));
                    ctrl.as_mut().poolStateChanged(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().poolStateChanged(false);
                }
            });
        });
    }

    pub fn stopPool(self: Pin<&mut Self>) {
        let mut this = self;
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::pool::stop_pool_miner(&ctx).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    ctrl.as_mut().set_running(false);
                    ctrl.as_mut().set_hashrateHm(0.0);
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Pool miner stopped"));
                    ctrl.as_mut().poolStateChanged(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().poolStateChanged(false);
                }
            });
        });
    }
}
