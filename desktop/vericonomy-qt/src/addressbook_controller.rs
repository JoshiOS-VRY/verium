//! Address book controller — CRUD for AddressBookPage.

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
        #[qproperty(QString, entriesJson)]
        #[qproperty(QString, lastMessage)]
        #[qproperty(bool, loading)]
        type AddressBookController = super::AddressBookControllerRust;

        #[qsignal]
        fn entriesRefreshed(self: Pin<&mut AddressBookController>, ok: bool);

        #[qsignal]
        fn entrySaved(self: Pin<&mut AddressBookController>, ok: bool);

        #[qsignal]
        fn entryDeleted(self: Pin<&mut AddressBookController>, ok: bool);

        #[qinvokable]
        fn refresh(self: Pin<&mut AddressBookController>);

        #[qinvokable]
        fn upsert(self: Pin<&mut AddressBookController>, entryJson: &QString);

        #[qinvokable]
        fn remove(self: Pin<&mut AddressBookController>, id: &QString);
    }

    impl cxx_qt::Threading for AddressBookController {}
}

pub struct AddressBookControllerRust {
    coin: QString,
    entriesJson: QString,
    lastMessage: QString,
    loading: bool,
    refresh_coin: String,
    refresh_ready: bool,
}

impl Default for AddressBookControllerRust {
    fn default() -> Self {
        Self {
            coin: QString::from("verium"),
            entriesJson: QString::from("[]"),
            lastMessage: QString::default(),
            loading: false,
            refresh_coin: String::new(),
            refresh_ready: false,
        }
    }
}

fn coin_id(coin: &QString) -> vericonomy_desktop_host::CoinId {
    vericonomy_desktop_host::CoinId::parse(&coin.to_string())
        .unwrap_or(vericonomy_desktop_host::CoinId::Verium)
}

impl qobject::AddressBookController {
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
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::addressbook::list(coin);
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(entries) => {
                    let json = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".into());
                    ctrl.as_mut().set_entriesJson(QString::from(&json));
                    ctrl.as_mut().set_loading(false);
                    crate::controller_refresh::mark_poll_ready(&mut Pin::get_mut(ctrl.as_mut().rust_mut()).refresh_ready);
                    ctrl.as_mut().entriesRefreshed(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().entriesRefreshed(false);
                }
            });
        });
    }

    pub fn upsert(self: Pin<&mut Self>, entryJson: &QString) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = coin_id(&this.coin());
        let text = entryJson.to_string();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = match serde_json::from_str::<
                vericonomy_desktop_host::commands::addressbook::AddressBookEntry,
            >(&text)
            {
                Ok(entry) => vericonomy_desktop_host::commands::addressbook::upsert(coin, entry),
                Err(e) => Err(vericonomy_desktop_host::HostError::other(e.to_string())),
            };
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(saved) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Address saved"));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().entrySaved(true);
                    ctrl.as_mut().refresh();
                    let _ = saved;
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().entrySaved(false);
                }
            });
        });
    }

    pub fn remove(self: Pin<&mut Self>, id: &QString) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let coin = coin_id(&this.coin());
        let entry_id = id.to_string();
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result =
                vericonomy_desktop_host::commands::addressbook::delete(coin, &entry_id);
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Address removed"));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().entryDeleted(true);
                    ctrl.as_mut().refresh();
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().entryDeleted(false);
                }
            });
        });
    }
}
