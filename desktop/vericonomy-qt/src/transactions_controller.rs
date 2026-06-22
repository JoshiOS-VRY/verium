//! Transaction list controller — feeds Transactions page + Dashboard chart.

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
        #[qproperty(QString, rowsJson)]
        #[qproperty(QString, balanceSeriesJson)]
        #[qproperty(QString, poolPayoutTxidsJson)]
        #[qproperty(i32, rowCount)]
        #[qproperty(i32, walletTxCount)]
        #[qproperty(bool, historyCapped)]
        #[qproperty(bool, loading)]
        #[qproperty(bool, hasError)]
        type TransactionsController = super::TransactionsControllerRust;

        #[qsignal]
        fn dataRefreshed(self: Pin<&mut TransactionsController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut TransactionsController>);
    }

    impl cxx_qt::Threading for TransactionsController {}
}

pub struct TransactionsControllerRust {
    coin: QString,
    rowsJson: QString,
    balanceSeriesJson: QString,
    poolPayoutTxidsJson: QString,
    rowCount: i32,
    walletTxCount: i32,
    historyCapped: bool,
    loading: bool,
    hasError: bool,
}

impl Default for TransactionsControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            rowsJson: QString::from("[]"),
            balanceSeriesJson: QString::from("[]"),
            poolPayoutTxidsJson: QString::from("[]"),
            rowCount: 0,
            walletTxCount: 0,
            historyCapped: false,
            loading: false,
            hasError: false,
        }
    }
}

impl qobject::TransactionsController {
    pub fn refresh(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        this.as_mut().set_hasError(false);

        let coin = vericonomy_desktop_host::CoinId::parse(&this.coin().to_string())
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let ctx = crate::app_context::context();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let tx_result =
                vericonomy_desktop_host::commands::transactions::fetch_transactions(&ctx, coin)
                    .await;

            let wallet_tx_count = tx_result.as_ref().map(|l| l.rows.len() as i64).unwrap_or(0);
            let history_capped = wallet_tx_count
                > vericonomy_desktop_host::commands::transactions::TRANSACTIONS_LIST_CAP as i64;

            let _ = qt_thread.queue(move |mut ctrl| {
                ctrl.as_mut().set_walletTxCount(wallet_tx_count as i32);
                ctrl.as_mut().set_historyCapped(history_capped);

                match tx_result {
                    Ok(list) => {
                        let rows_json =
                            serde_json::to_string(&list.rows).unwrap_or_else(|_| "[]".into());
                        let series_json = serde_json::to_string(&list.balance_series)
                            .unwrap_or_else(|_| "[]".into());
                        ctrl.as_mut().set_rowCount(list.rows.len() as i32);
                        ctrl.as_mut().set_rowsJson(QString::from(&rows_json));
                        ctrl.as_mut()
                            .set_balanceSeriesJson(QString::from(&series_json));
                        ctrl.as_mut().set_loading(false);
                        ctrl.as_mut().set_hasError(false);
                        ctrl.as_mut().dataRefreshed(true);
                    }
                    Err(_) => {
                        ctrl.as_mut().set_loading(false);
                        ctrl.as_mut().set_hasError(true);
                        ctrl.as_mut().dataRefreshed(false);
                    }
                }
            });
        });
    }
}
