//! `WalletController` — balance, send/receive, sign/verify.

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
        #[qproperty(f64, balance)]
        #[qproperty(f64, unconfirmed)]
        #[qproperty(f64, immature)]
        #[qproperty(f64, total)]
        #[qproperty(bool, locked)]
        #[qproperty(bool, missing)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, receiveAddress)]
        #[qproperty(QString, lastMessage)]
        #[qproperty(QString, lastTxid)]
        type WalletController = super::WalletControllerRust;

        #[qsignal]
        fn balanceRefreshed(self: Pin<&mut WalletController>, ok: bool);

        #[qsignal]
        fn addressRefreshed(self: Pin<&mut WalletController>, ok: bool);

        #[qsignal]
        fn sendCompleted(self: Pin<&mut WalletController>, ok: bool);

        #[qsignal]
        fn signCompleted(self: Pin<&mut WalletController>, ok: bool);

        #[qsignal]
        fn verifyCompleted(self: Pin<&mut WalletController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut WalletController>);

        #[qinvokable]
        fn refreshAddress(self: Pin<&mut WalletController>);

        #[qinvokable]
        fn send(self: Pin<&mut WalletController>, address: &QString, amount: f64);

        #[qinvokable]
        fn signMessage(
            self: Pin<&mut WalletController>,
            address: &QString,
            message: &QString,
        );

        #[qinvokable]
        fn verifyMessage(
            self: Pin<&mut WalletController>,
            address: &QString,
            signature: &QString,
            message: &QString,
        );

        #[qsignal]
        fn unlockCompleted(self: Pin<&mut WalletController>, ok: bool);

        #[qsignal]
        fn createCompleted(self: Pin<&mut WalletController>, ok: bool);

        #[qsignal]
        fn restoreCompleted(self: Pin<&mut WalletController>, ok: bool);

        #[qinvokable]
        fn unlock(self: Pin<&mut WalletController>, passphrase: &QString);

        #[qinvokable]
        fn createEncrypted(self: Pin<&mut WalletController>, passphrase: &QString);

        #[qinvokable]
        fn restoreWallet(self: Pin<&mut WalletController>, sourcePath: &QString);
    }

    impl cxx_qt::Threading for WalletController {}
}

pub struct WalletControllerRust {
    coin: QString,
    balance: f64,
    unconfirmed: f64,
    immature: f64,
    total: f64,
    locked: bool,
    missing: bool,
    loading: bool,
    receiveAddress: QString,
    lastMessage: QString,
    lastTxid: QString,
}

impl Default for WalletControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            balance: 0.0,
            unconfirmed: 0.0,
            immature: 0.0,
            total: 0.0,
            locked: false,
            missing: false,
            loading: false,
            receiveAddress: QString::default(),
            lastMessage: QString::default(),
            lastTxid: QString::default(),
        }
    }
}

fn coin_id(coin: &QString) -> vericonomy_desktop_host::CoinId {
    vericonomy_desktop_host::CoinId::parse(&coin.to_string())
        .unwrap_or(vericonomy_desktop_host::CoinId::Verium)
}

impl qobject::WalletController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = coin_id(&this.coin());
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::wallet::get_wallet_info(&ctx, coin).await;

            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(info) => {
                    let total = info.total();
                    ctrl.as_mut().set_balance(info.balance);
                    ctrl.as_mut().set_unconfirmed(info.unconfirmed_balance);
                    ctrl.as_mut().set_immature(info.immature_balance);
                    ctrl.as_mut().set_total(total);
                    ctrl.as_mut().set_locked(info.locked);
                    ctrl.as_mut().set_missing(info.missing);
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().balanceRefreshed(true);
                }
                Err(_) => {
                    ctrl.as_mut().set_missing(true);
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().balanceRefreshed(false);
                }
            });
        });
    }

    pub fn refreshAddress(self: Pin<&mut Self>) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::wallet::get_new_address(&ctx, coin).await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(addr) => {
                    ctrl.as_mut().set_receiveAddress(QString::from(&addr));
                    ctrl.as_mut().addressRefreshed(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().addressRefreshed(false);
                }
            });
        });
    }

    pub fn send(self: Pin<&mut Self>, address: &QString, amount: f64) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let addr = address.to_string();
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::wallet::send_to_address(
                &ctx, coin, &addr, amount,
            )
            .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(txid) => {
                    ctrl.as_mut().set_lastTxid(QString::from(&txid));
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Transaction sent"));
                    ctrl.as_mut().sendCompleted(true);
                    ctrl.as_mut().refresh();
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().sendCompleted(false);
                }
            });
        });
    }

    pub fn signMessage(self: Pin<&mut Self>, address: &QString, message: &QString) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let addr = address.to_string();
        let msg = message.to_string();
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::wallet::sign_message(
                &ctx, coin, &addr, &msg,
            )
            .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(sig) => {
                    ctrl.as_mut().set_lastMessage(QString::from(&sig));
                    ctrl.as_mut().signCompleted(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().signCompleted(false);
                }
            });
        });
    }

    pub fn verifyMessage(
        self: Pin<&mut Self>,
        address: &QString,
        signature: &QString,
        message: &QString,
    ) {
        let mut this = self;
        let coin = coin_id(&this.coin());
        let addr = address.to_string();
        let sig = signature.to_string();
        let msg = message.to_string();
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::wallet::verify_message(
                &ctx, coin, &addr, &sig, &msg,
            )
            .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(valid) => {
                    let text = if valid {
                        "Signature is valid"
                    } else {
                        "Signature is invalid"
                    };
                    ctrl.as_mut().set_lastMessage(QString::from(text));
                    ctrl.as_mut().verifyCompleted(valid);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().verifyCompleted(false);
                }
            });
        });
    }

    pub fn unlock(self: Pin<&mut Self>, passphrase: &QString) {
        let this = self;
        let coin = coin_id(&this.coin());
        let pass = passphrase.to_string();
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::wallet::wallet_unlock(
                &ctx,
                coin,
                &pass,
                3600,
            )
            .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Wallet unlocked"));
                    ctrl.as_mut().unlockCompleted(true);
                    ctrl.as_mut().refresh();
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().unlockCompleted(false);
                }
            });
        });
    }

    pub fn createEncrypted(self: Pin<&mut Self>, passphrase: &QString) {
        let this = self;
        let coin = coin_id(&this.coin());
        let pass = passphrase.to_string();
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::wallet::wallet_create_encrypted(
                &ctx, coin, &pass,
            )
            .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(res) => {
                    ctrl.as_mut().set_lastMessage(QString::from(&res.message));
                    ctrl.as_mut().createCompleted(res.success);
                    if res.success {
                        ctrl.as_mut().refresh();
                    }
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().createCompleted(false);
                }
            });
        });
    }

    pub fn restoreWallet(self: Pin<&mut Self>, sourcePath: &QString) {
        let this = self;
        let coin = coin_id(&this.coin());
        let path = sourcePath.to_string();
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::wallet::wallet_restore(&ctx, coin, &path)
                    .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(res) => {
                    ctrl.as_mut().set_lastMessage(QString::from(&res.message));
                    ctrl.as_mut().restoreCompleted(res.success);
                    if res.success {
                        ctrl.as_mut().refresh();
                    }
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().restoreCompleted(false);
                }
            });
        });
    }
}
