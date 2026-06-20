//! `HostLinks` — QML singleton for OS integration that replaces Tauri plugins.
//!
//! Today: open external links via the host layer's platform opener, with a
//! scheme allowlist (http/https only) so QML can never hand arbitrary URIs to
//! the shell. File dialogs / notifications / deep links attach here as the
//! corresponding host commands land (Phase 5).

#![allow(non_snake_case)]

use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type HostLinks = super::HostLinksRust;

        /// Open an external http(s) URL with the OS default handler.
        #[qinvokable]
        fn open(self: &HostLinks, url: &QString);
    }
}

#[derive(Default)]
pub struct HostLinksRust;

impl qobject::HostLinks {
    pub fn open(&self, url: &QString) {
        let url = url.to_string();
        if vericonomy_desktop_host::os::is_safe_external(&url) {
            vericonomy_desktop_host::os::open_external(&url);
        }
    }
}
