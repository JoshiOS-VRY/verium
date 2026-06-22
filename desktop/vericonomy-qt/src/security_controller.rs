//! Security controller — 2FA, auto-lock, spending controls, receive requests, recovery export.

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
        #[qproperty(QString, twoFactorJson)]
        #[qproperty(QString, autoLockJson)]
        #[qproperty(QString, spendingJson)]
        #[qproperty(QString, receiveRequestsJson)]
        #[qproperty(QString, exportResultJson)]
        #[qproperty(QString, sendCheckJson)]
        #[qproperty(QString, lastMessage)]
        #[qproperty(bool, loading)]
        type SecurityController = super::SecurityControllerRust;

        #[qsignal]
        fn securityLoaded(self: Pin<&mut SecurityController>, ok: bool);

        #[qsignal]
        fn exportCompleted(self: Pin<&mut SecurityController>, ok: bool);

        #[qsignal]
        fn sendCheckCompleted(self: Pin<&mut SecurityController>, ok: bool);

        #[qinvokable]
        fn checkSend(self: Pin<&mut SecurityController>, address: &QString, amount: f64);

        #[qinvokable]
        fn refresh(self: Pin<&mut SecurityController>);

        #[qinvokable]
        fn saveAutoLock(self: Pin<&mut SecurityController>, json: &QString);

        #[qinvokable]
        fn saveSpending(self: Pin<&mut SecurityController>, json: &QString);

        #[qinvokable]
        fn startTwoFactor(self: Pin<&mut SecurityController>);

        #[qinvokable]
        fn confirmTwoFactor(self: Pin<&mut SecurityController>, code: &QString, secret: &QString);

        #[qinvokable]
        fn disableTwoFactor(self: Pin<&mut SecurityController>, code: &QString);

        #[qinvokable]
        fn exportRecovery(self: Pin<&mut SecurityController>, walletPassphrase: &QString);

        #[qinvokable]
        fn appendReceiveRequest(
            self: Pin<&mut SecurityController>,
            label: &QString,
            message: &QString,
            amount: f64,
            includeAmount: bool,
            address: &QString,
        );

        #[qinvokable]
        fn deleteReceiveRequest(self: Pin<&mut SecurityController>, id: &QString);
    }

    impl cxx_qt::Threading for SecurityController {}
}

pub struct SecurityControllerRust {
    coin: QString,
    twoFactorJson: QString,
    autoLockJson: QString,
    spendingJson: QString,
    receiveRequestsJson: QString,
    exportResultJson: QString,
    sendCheckJson: QString,
    lastMessage: QString,
    loading: bool,
}

impl Default for SecurityControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            twoFactorJson: QString::from("{}"),
            autoLockJson: QString::from("{}"),
            spendingJson: QString::from("{}"),
            receiveRequestsJson: QString::from("[]"),
            exportResultJson: QString::from("{}"),
            sendCheckJson: QString::from("{}"),
            lastMessage: QString::default(),
            loading: false,
        }
    }
}

fn coin_id(coin: &QString) -> vericonomy_desktop_host::CoinId {
    vericonomy_desktop_host::CoinId::parse(&coin.to_string())
        .unwrap_or(vericonomy_desktop_host::CoinId::Verium)
}

impl qobject::SecurityController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = coin_id(&this.coin());
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let tf = vericonomy_desktop_host::commands::security::two_factor_status().await;
            let al = vericonomy_desktop_host::commands::security::auto_lock_get_config();
            let sp = vericonomy_desktop_host::commands::security::spending_controls_get();
            let rr = vericonomy_desktop_host::commands::security::receive_requests_list(coin);

            let _ = qt_thread.queue(move |mut ctrl| {
                if let Ok(v) = tf {
                    let json = serde_json::to_string(&v).unwrap_or_else(|_| "{}".into());
                    ctrl.as_mut().set_twoFactorJson(QString::from(&json));
                }
                if let Ok(v) = al {
                    let json = serde_json::to_string(&v).unwrap_or_else(|_| "{}".into());
                    ctrl.as_mut().set_autoLockJson(QString::from(&json));
                }
                if let Ok(v) = sp {
                    let json = serde_json::to_string(&v).unwrap_or_else(|_| "{}".into());
                    ctrl.as_mut().set_spendingJson(QString::from(&json));
                }
                if let Ok(v) = rr {
                    let json = serde_json::to_string(&v).unwrap_or_else(|_| "[]".into());
                    ctrl.as_mut().set_receiveRequestsJson(QString::from(&json));
                }
                ctrl.as_mut().set_loading(false);
                ctrl.as_mut().securityLoaded(true);
            });
        });
    }

    pub fn saveAutoLock(self: Pin<&mut Self>, json: &QString) {
        let text = json.to_string();
        let qt_thread = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let result = match serde_json::from_str::<vericonomy_desktop_host::auto_lock::AutoLockConfig>(&text) {
                Ok(cfg) => vericonomy_desktop_host::commands::security::auto_lock_set_config(&cfg),
                Err(e) => Err(vericonomy_desktop_host::HostError::other(e.to_string())),
            };
            let _ = qt_thread.queue(move |mut ctrl| {
                ctrl.as_mut().set_lastMessage(match &result {
                    Ok(()) => QString::from("Auto-lock settings saved"),
                    Err(e) => QString::from(&e.to_string()),
                });
            });
        });
    }

    pub fn saveSpending(self: Pin<&mut Self>, json: &QString) {
        let text = json.to_string();
        let qt_thread = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let result = match serde_json::from_str::<
                vericonomy_desktop_host::spending_controls::SpendingControlsConfig,
            >(&text)
            {
                Ok(cfg) => vericonomy_desktop_host::commands::security::spending_controls_save(&cfg),
                Err(e) => Err(vericonomy_desktop_host::HostError::other(e.to_string())),
            };
            let _ = qt_thread.queue(move |mut ctrl| {
                ctrl.as_mut().set_lastMessage(match &result {
                    Ok(()) => QString::from("Spending controls saved"),
                    Err(e) => QString::from(&e.to_string()),
                });
            });
        });
    }

    pub fn startTwoFactor(self: Pin<&mut Self>) {
        let qt_thread = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::security::two_factor_start_enrollment();
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(enrollment) => {
                    let json = serde_json::to_string(&enrollment).unwrap_or_else(|_| "{}".into());
                    ctrl.as_mut().set_exportResultJson(QString::from(&json));
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Scan the URI in your authenticator app"));
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                }
            });
        });
    }

    pub fn confirmTwoFactor(self: Pin<&mut Self>, code: &QString, secret: &QString) {
        let code_s = code.to_string();
        let secret_s = secret.to_string();
        let qt_thread = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::security::two_factor_confirm_enrollment(
                &code_s, &secret_s,
            );
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(cfg) => {
                    let json = serde_json::to_string(&cfg).unwrap_or_else(|_| "{}".into());
                    ctrl.as_mut().set_twoFactorJson(QString::from(&json));
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Two-factor authentication enabled"));
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                }
            });
        });
    }

    pub fn disableTwoFactor(self: Pin<&mut Self>, code: &QString) {
        let code_s = code.to_string();
        let qt_thread = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::security::two_factor_disable(&code_s);
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Two-factor authentication disabled"));
                    ctrl.as_mut().refresh();
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                }
            });
        });
    }

    pub fn checkSend(self: Pin<&mut Self>, address: &QString, amount: f64) {
        let coin = coin_id(&self.coin());
        let addr = address.to_string();
        let qt_thread = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let check = vericonomy_desktop_host::commands::security::spending_controls_check_send(
                coin, &addr, amount,
            );
            let two_fa = vericonomy_desktop_host::commands::security::two_factor_is_gated(
                "send",
                Some(amount),
                coin,
            )
            .unwrap_or(false);
            let result = check.map(|c| {
                serde_json::json!({
                    "allowed": c.allowed,
                    "reason": c.reason,
                    "requires_extra_confirmation": c.requires_extra_confirmation,
                    "look_alike_warning": c.look_alike_warning,
                    "two_factor_required": two_fa,
                })
                .to_string()
            });
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(json) => {
                    ctrl.as_mut().set_sendCheckJson(QString::from(&json));
                    ctrl.as_mut().sendCheckCompleted(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().sendCheckCompleted(false);
                }
            });
        });
    }

    pub fn exportRecovery(self: Pin<&mut Self>, wallet_passphrase: &QString) {
        let coin = coin_id(&self.coin());
        let pass = wallet_passphrase.to_string();
        let ctx = crate::app_context::context();
        let qt_thread = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::security::recovery_export_seed(
                ctx.as_ref(),
                coin,
                &pass,
            )
            .await;
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(res) => {
                    let json = serde_json::to_string(&res).unwrap_or_else(|_| "{}".into());
                    ctrl.as_mut().set_exportResultJson(QString::from(&json));
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Recovery material exported — store it safely"));
                    ctrl.as_mut().exportCompleted(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().exportCompleted(false);
                }
            });
        });
    }

    pub fn appendReceiveRequest(
        self: Pin<&mut Self>,
        label: &QString,
        message: &QString,
        amount: f64,
        include_amount: bool,
        address: &QString,
    ) {
        let coin = coin_id(&self.coin());
        let entry = vericonomy_desktop_host::receive_requests::ReceiveRequest {
            id: String::new(),
            created_at: 0,
            label: label.to_string(),
            message: message.to_string(),
            amount: if include_amount && amount > 0.0 {
                Some(amount)
            } else {
                None
            },
            address: address.to_string(),
        };
        let qt_thread = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::security::receive_requests_append(coin, entry);
            let _ = qt_thread.queue(move |mut ctrl| {
                if result.is_ok() {
                    ctrl.as_mut().refresh();
                }
            });
        });
    }

    pub fn deleteReceiveRequest(self: Pin<&mut Self>, id: &QString) {
        let coin = coin_id(&self.coin());
        let entry_id = id.to_string();
        let qt_thread = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let _ = vericonomy_desktop_host::commands::security::receive_requests_delete(
                coin, &entry_id,
            );
            let _ = qt_thread.queue(move |mut ctrl| {
                ctrl.as_mut().refresh();
            });
        });
    }
}
