//! Light wallet controller — import xprv/phrase, unlock, sync.

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
        #[qproperty(bool, exists)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, lastMessage)]
        type LightWalletController = super::LightWalletControllerRust;

        #[qsignal]
        fn importCompleted(self: Pin<&mut LightWalletController>, ok: bool);

        #[qsignal]
        fn unlockCompleted(self: Pin<&mut LightWalletController>, ok: bool);

        #[qinvokable]
        fn refreshExists(self: Pin<&mut LightWalletController>);

        #[qinvokable]
        fn importWallet(
            self: Pin<&mut LightWalletController>,
            seedSecret: &QString,
            passphrase: &QString,
        );

        #[qinvokable]
        fn unlock(self: Pin<&mut LightWalletController>, passphrase: &QString);
    }

    impl cxx_qt::Threading for LightWalletController {}
}

pub struct LightWalletControllerRust {
    coin: QString,
    exists: bool,
    loading: bool,
    lastMessage: QString,
}

impl Default for LightWalletControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            exists: false,
            loading: false,
            lastMessage: QString::default(),
        }
    }
}

impl qobject::LightWalletController {
    pub fn refreshExists(self: Pin<&mut Self>) {
        let coin = vericonomy_desktop_host::CoinId::parse(&self.coin().to_string())
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let ctx = crate::app_context::context();
        let qt_thread = self.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::light_wallet::exists(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(ex) => {
                    ctrl.as_mut().set_exists(ex);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                }
            });
        });
    }

    pub fn importWallet(mut self: Pin<&mut Self>, seed_secret: &QString, passphrase: &QString) {
        let coin = vericonomy_desktop_host::CoinId::parse(&self.coin().to_string())
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let seed = seed_secret.to_string();
        let pass = passphrase.to_string();
        let ctx = crate::app_context::context();
        let qt_thread = self.qt_thread();
        self.as_mut().set_loading(true);

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::light_wallet::import_wallet(
                &ctx, coin, &seed, &pass, None,
            )
            .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    ctrl.as_mut().set_exists(true);
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Wallet imported — syncing…"));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().importCompleted(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().importCompleted(false);
                }
            });
        });
    }

    pub fn unlock(mut self: Pin<&mut Self>, passphrase: &QString) {
        let coin = vericonomy_desktop_host::CoinId::parse(&self.coin().to_string())
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let pass = passphrase.to_string();
        let ctx = crate::app_context::context();
        let qt_thread = self.qt_thread();
        self.as_mut().set_loading(true);

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::light_wallet::unlock(&ctx, coin, &pass).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Wallet unlocked"));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().unlockCompleted(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().unlockCompleted(false);
                }
            });
        });
    }
}
