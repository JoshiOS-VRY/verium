//! Mining stats + minerstart/minerstop controls.

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
        #[qproperty(f64, networkHashps)]
        #[qproperty(f64, difficulty)]
        #[qproperty(i32, blocks)]
        #[qproperty(QString, chain)]
        #[qproperty(QString, warnings)]
        #[qproperty(bool, loading)]
        #[qproperty(bool, minerActive)]
        #[qproperty(i32, threads)]
        #[qproperty(QString, lastMessage)]
        type MiningController = super::MiningControllerRust;

        #[qsignal]
        fn miningRefreshed(self: Pin<&mut MiningController>, ok: bool);

        #[qsignal]
        fn minerStateChanged(self: Pin<&mut MiningController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut MiningController>);

        #[qinvokable]
        fn startMiner(self: Pin<&mut MiningController>, threads: i32);

        #[qinvokable]
        fn stopMiner(self: Pin<&mut MiningController>);
    }

    impl cxx_qt::Threading for MiningController {}
}

pub struct MiningControllerRust {
    coin: QString,
    networkHashps: f64,
    difficulty: f64,
    blocks: i32,
    chain: QString,
    warnings: QString,
    loading: bool,
    minerActive: bool,
    threads: i32,
    lastMessage: QString,
}

impl Default for MiningControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            networkHashps: 0.0,
            difficulty: 0.0,
            blocks: 0,
            chain: QString::default(),
            warnings: QString::default(),
            loading: false,
            minerActive: false,
            threads: 2,
            lastMessage: QString::default(),
        }
    }
}

fn coin_id(coin: &QString) -> vericonomy_desktop_host::CoinId {
    vericonomy_desktop_host::CoinId::parse(&coin.to_string())
        .unwrap_or(vericonomy_desktop_host::CoinId::Verium)
}

impl qobject::MiningController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = coin_id(&this.coin());
        let ctx = crate::app_context::context();
        let earn = vericonomy_desktop_host::commands::mining::get_miner_state(&ctx, coin);
        this.as_mut().set_minerActive(earn.active);
        this.as_mut().set_threads(earn.threads as i32);
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::mining::get_mining_info(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(info) => {
                    ctrl.as_mut().set_networkHashps(info.networkhashps);
                    ctrl.as_mut().set_difficulty(info.difficulty);
                    ctrl.as_mut().set_blocks(info.blocks as i32);
                    ctrl.as_mut().set_chain(QString::from(&info.chain));
                    ctrl.as_mut().set_warnings(QString::from(&info.warnings));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().miningRefreshed(true);
                }
                Err(_) => {
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().miningRefreshed(false);
                }
            });
        });
    }

    pub fn startMiner(self: Pin<&mut Self>, threads: i32) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();
        let n = threads.max(1) as u32;

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::mining::miner_start(&ctx, coin, n, None).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(state) => {
                    ctrl.as_mut().set_minerActive(state.active);
                    ctrl.as_mut().set_threads(state.threads as i32);
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Miner started"));
                    ctrl.as_mut().minerStateChanged(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().minerStateChanged(false);
                }
            });
        });
    }

    pub fn stopMiner(self: Pin<&mut Self>) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::mining::miner_stop(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(state) => {
                    ctrl.as_mut().set_minerActive(state.active);
                    ctrl.as_mut().set_threads(state.threads as i32);
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Miner stopped"));
                    ctrl.as_mut().minerStateChanged(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().minerStateChanged(false);
                }
            });
        });
    }
}
