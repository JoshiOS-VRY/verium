//! Per-coin wallet mode controller (full node vs light).

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
        #[qproperty(QString, mode)]
        #[qproperty(bool, isLight)]
        #[qproperty(bool, lightWalletExists)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, lastMessage)]
        type WalletModeController = super::WalletModeControllerRust;

        #[qsignal]
        fn modeRefreshed(self: Pin<&mut WalletModeController>, ok: bool);

        #[qsignal]
        fn walletModeChanged(self: Pin<&mut WalletModeController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut WalletModeController>);

        #[qinvokable]
        fn setWalletMode(self: Pin<&mut WalletModeController>, mode: &QString);
    }

    impl cxx_qt::Threading for WalletModeController {}
}

pub struct WalletModeControllerRust {
    coin: QString,
    mode: QString,
    isLight: bool,
    lightWalletExists: bool,
    loading: bool,
    lastMessage: QString,
    refresh_coin: String,
    refresh_ready: bool,
}

impl Default for WalletModeControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            mode: QString::from("full_node"),
            isLight: false,
            lightWalletExists: false,
            loading: false,
            lastMessage: QString::default(),
            refresh_coin: String::new(),
            refresh_ready: false,
        }
    }
}

fn coin_id(coin: &QString) -> vericonomy_desktop_host::CoinId {
    vericonomy_desktop_host::CoinId::parse(&coin.to_string())
        .unwrap_or(vericonomy_desktop_host::CoinId::Verium)
}

impl qobject::WalletModeController {
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
        let coin = coin_id(&this.coin());
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::wallet_mode::get_for_coin(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(status) => {
                    let is_light = status.mode == "light";
                    ctrl.as_mut().set_mode(QString::from(&status.mode));
                    ctrl.as_mut().set_isLight(is_light);
                    ctrl.as_mut()
                        .set_lightWalletExists(status.light_wallet_exists);
                    ctrl.as_mut().set_loading(false);
                    crate::controller_refresh::mark_poll_ready(&mut Pin::get_mut(ctrl.as_mut().rust_mut()).refresh_ready);
                    ctrl.as_mut().modeRefreshed(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().modeRefreshed(false);
                }
            });
        });
    }

    pub fn setWalletMode(self: Pin<&mut Self>, mode: &QString) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let mode_str = mode.to_string();
        this.as_mut().set_loading(true);
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::wallet_mode::set_for_coin(coin, &mode_str);
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    let is_light = mode_str == "light";
                    ctrl.as_mut().set_mode(QString::from(&mode_str));
                    ctrl.as_mut().set_isLight(is_light);
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Wallet mode updated"));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().walletModeChanged(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().walletModeChanged(false);
                }
            });
        });
    }
}
