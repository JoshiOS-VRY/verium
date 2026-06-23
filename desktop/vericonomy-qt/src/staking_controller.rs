//! Staking controller (Vericoin PoST).

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
        #[qproperty(bool, enabled)]
        #[qproperty(f64, stakeWeight)]
        #[qproperty(f64, netStakeWeight)]
        #[qproperty(f64, expectedTime)]
        #[qproperty(f64, searchesPerSecond)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, lastMessage)]
        type StakingController = super::StakingControllerRust;

        #[qsignal]
        fn stakingRefreshed(self: Pin<&mut StakingController>, ok: bool);

        #[qsignal]
        fn stakingStateChanged(self: Pin<&mut StakingController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut StakingController>);

        #[qinvokable]
        fn startStaking(self: Pin<&mut StakingController>);

        #[qinvokable]
        fn stopStaking(self: Pin<&mut StakingController>);
    }

    impl cxx_qt::Threading for StakingController {}
}

pub struct StakingControllerRust {
    coin: QString,
    enabled: bool,
    stakeWeight: f64,
    netStakeWeight: f64,
    expectedTime: f64,
    searchesPerSecond: f64,
    loading: bool,
    lastMessage: QString,
    refresh_coin: String,
    refresh_ready: bool,
}

impl Default for StakingControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("vericoin"),
            enabled: false,
            stakeWeight: 0.0,
            netStakeWeight: 0.0,
            expectedTime: 0.0,
            searchesPerSecond: 0.0,
            loading: false,
            lastMessage: QString::default(),
            refresh_coin: String::new(),
            refresh_ready: false,
        }
    }
}

impl qobject::StakingController {
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
            .unwrap_or(vericonomy_desktop_host::CoinId::Vericoin);
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::staking::get_staking_info(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(info) => {
                    ctrl.as_mut().set_enabled(info.enabled);
                    ctrl.as_mut().set_stakeWeight(info.stake_weight);
                    ctrl.as_mut().set_netStakeWeight(info.netstakeweight);
                    ctrl.as_mut().set_expectedTime(info.expected_time);
                    ctrl.as_mut().set_searchesPerSecond(info.searches_per_second);
                    ctrl.as_mut().set_loading(false);
                    crate::controller_refresh::mark_poll_ready(&mut Pin::get_mut(ctrl.as_mut().rust_mut()).refresh_ready);
                    ctrl.as_mut().stakingRefreshed(true);
                }
                Err(_) => {
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().stakingRefreshed(false);
                }
            });
        });
    }

    pub fn startStaking(self: Pin<&mut Self>) {
        let mut this = self;
        let coin = vericonomy_desktop_host::CoinId::Vericoin;
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::staking::staking_start(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(_) => {
                    ctrl.as_mut().set_enabled(true);
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Staking started"));
                    ctrl.as_mut().stakingStateChanged(true);
                    ctrl.as_mut().refresh();
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().stakingStateChanged(false);
                }
            });
        });
    }

    pub fn stopStaking(self: Pin<&mut Self>) {
        let mut this = self;
        let coin = vericonomy_desktop_host::CoinId::Vericoin;
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::staking::staking_stop(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(_) => {
                    ctrl.as_mut().set_enabled(false);
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Staking stopped"));
                    ctrl.as_mut().stakingStateChanged(true);
                    ctrl.as_mut().refresh();
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().stakingStateChanged(false);
                }
            });
        });
    }
}
