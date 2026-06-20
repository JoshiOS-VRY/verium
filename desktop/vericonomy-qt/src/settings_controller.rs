//! Settings / preferences controller.

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
        #[qproperty(QString, prefsJson)]
        #[qproperty(QString, lastMessage)]
        #[qproperty(bool, loading)]
        type SettingsController = super::SettingsControllerRust;

        #[qsignal]
        fn prefsLoaded(self: Pin<&mut SettingsController>, ok: bool);

        #[qsignal]
        fn prefsSaved(self: Pin<&mut SettingsController>, ok: bool);

        #[qinvokable]
        fn load(self: Pin<&mut SettingsController>);

        #[qinvokable]
        fn savePrefs(self: Pin<&mut SettingsController>, json: &QString);
    }

    impl cxx_qt::Threading for SettingsController {}
}

pub struct SettingsControllerRust {
    prefsJson: QString,
    lastMessage: QString,
    loading: bool,
}

impl Default for SettingsControllerRust {
    fn default() -> Self {
        Self {
            prefsJson: QString::from("{}"),
            lastMessage: QString::default(),
            loading: false,
        }
    }
}

impl qobject::SettingsController {
    pub fn load(self: Pin<&mut Self>) {
        let mut this = self;
        this.as_mut().set_loading(true);
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = vericonomy_desktop_host::commands::setup::get_prefs();
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(prefs) => {
                    let json = serde_json::to_string(&prefs).unwrap_or_else(|_| "{}".into());
                    ctrl.as_mut().set_prefsJson(QString::from(&json));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().prefsLoaded(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().prefsLoaded(false);
                }
            });
        });
    }

    pub fn savePrefs(self: Pin<&mut Self>, json: &QString) {
        let mut this = self;
        let text = json.to_string();
        this.as_mut().set_loading(true);
        let qt_thread = this.qt_thread();

        crate::runtime::runtime().spawn(async move {
            let result = match serde_json::from_str::<vericonomy_desktop_host::UserPreferences>(&text)
            {
                Ok(prefs) => vericonomy_desktop_host::commands::setup::save_user_prefs(&prefs),
                Err(e) => Err(vericonomy_desktop_host::HostError::other(e.to_string())),
            };
            let _ = qt_thread.queue(move |mut ctrl| match result {
                Ok(()) => {
                    ctrl.as_mut().set_prefsJson(QString::from(&text));
                    ctrl.as_mut()
                        .set_lastMessage(QString::from("Preferences saved"));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().prefsSaved(true);
                }
                Err(e) => {
                    ctrl.as_mut()
                        .set_lastMessage(QString::from(&e.to_string()));
                    ctrl.as_mut().set_loading(false);
                    ctrl.as_mut().prefsSaved(false);
                }
            });
        });
    }
}
