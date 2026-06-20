//! RPC console controller.

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
        #[qproperty(QString, historyJson)]
        #[qproperty(bool, loading)]
        type RpcController = super::RpcControllerRust;

        #[qsignal]
        fn rpcCompleted(self: Pin<&mut RpcController>, ok: bool);

        #[qinvokable]
        fn call(self: Pin<&mut RpcController>, method: &QString, paramsJson: &QString);
    }

    impl cxx_qt::Threading for RpcController {}
}

pub struct RpcControllerRust {
    coin: QString,
    historyJson: QString,
    loading: bool,
}

impl Default for RpcControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            historyJson: QString::from("[]"),
            loading: false,
        }
    }
}

impl qobject::RpcController {
    pub fn call(self: Pin<&mut Self>, method: &QString, params_json: &QString) {
        let mut this = self;
        let method_s = method.to_string();
        let params_s = params_json.to_string();
        if method_s.trim().is_empty() {
            return;
        }
        let prev_json = this.historyJson().to_string();
        this.as_mut().set_loading(true);

        let coin = vericonomy_desktop_host::CoinId::parse(&this.coin().to_string())
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::diagnostics::rpc_raw_call(
                &ctx,
                coin,
                &method_s,
                &params_s,
            )
            .await;

            let _ = qt_thread.queue(move |mut ctrl| {
                let mut hist: Vec<serde_json::Value> =
                    serde_json::from_str(&prev_json).unwrap_or_default();
                let out = match &result {
                    Ok(v) => v.clone(),
                    Err(e) => format!("{{\"error\": \"{e}\"}}"),
                };
                hist.push(serde_json::json!({ "cmd": method_s, "out": out }));
                let json = serde_json::to_string(&hist).unwrap_or_else(|_| "[]".into());
                ctrl.as_mut().set_historyJson(QString::from(&json));
                ctrl.as_mut().set_loading(false);
                ctrl.as_mut().rpcCompleted(result.is_ok());
            });
        });
    }
}
