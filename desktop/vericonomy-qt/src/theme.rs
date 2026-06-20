//! `Theme` — design tokens exposed to QML as a reactive singleton.
//!
//! Ported 1:1 from `desktop/verium-app/src/index.css`. Implemented as a cxx-qt
//! `#[qml_singleton]` so QML can bind `Theme.bg`, `Theme.accent`, etc. and the
//! whole UI recolours live when `Theme.setDark(...)` toggles the palette.

#![allow(non_snake_case)]

use cxx_qt_lib::{QColor, QString};
use std::pin::Pin;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(bool, dark)]
        // Surfaces
        #[qproperty(QColor, bg)]
        #[qproperty(QColor, bgSubtle)]
        #[qproperty(QColor, bgPanel)]
        // Foreground
        #[qproperty(QColor, fg)]
        #[qproperty(QColor, fgMuted)]
        #[qproperty(QColor, fgSubtle)]
        // Lines
        #[qproperty(QColor, border)]
        #[qproperty(QColor, borderStrong)]
        // Brand / semantic
        #[qproperty(QColor, accent)]
        #[qproperty(QColor, accentFg)]
        #[qproperty(QColor, success)]
        #[qproperty(QColor, warning)]
        #[qproperty(QColor, danger)]
        // Radii
        #[qproperty(i32, radiusSm)]
        #[qproperty(i32, radiusMd)]
        #[qproperty(i32, radiusLg)]
        #[qproperty(i32, radiusXl)]
        #[qproperty(i32, radius2xl)]
        #[qproperty(i32, space)]
        // Typography
        #[qproperty(QString, fontFamily)]
        #[qproperty(QString, monoFamily)]
        type Theme = super::ThemeRust;

        /// Switch palette and recolour every bound property.
        #[qinvokable]
        fn setDarkMode(self: Pin<&mut Theme>, dark: bool);
    }
}

fn rgb(r: i32, g: i32, b: i32) -> QColor {
    QColor::from_rgb(r, g, b)
}

/// All palette colours for a given mode.
struct Palette {
    bg: QColor,
    bgSubtle: QColor,
    bgPanel: QColor,
    fg: QColor,
    fgMuted: QColor,
    fgSubtle: QColor,
    border: QColor,
    borderStrong: QColor,
    accent: QColor,
    accentFg: QColor,
    success: QColor,
    warning: QColor,
    danger: QColor,
}

fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: rgb(7, 10, 15),
            bgSubtle: rgb(11, 16, 24),
            bgPanel: rgb(26, 34, 48),
            fg: rgb(226, 232, 240),
            fgMuted: rgb(148, 163, 184),
            fgSubtle: rgb(100, 116, 139),
            border: rgb(30, 41, 59),
            borderStrong: rgb(51, 65, 85),
            accent: rgb(65, 139, 202),
            accentFg: rgb(240, 249, 255),
            success: rgb(53, 155, 55),
            warning: rgb(245, 158, 11),
            danger: rgb(233, 58, 93),
        }
    } else {
        Palette {
            bg: rgb(238, 241, 246),
            bgSubtle: rgb(248, 250, 252),
            bgPanel: rgb(255, 255, 255),
            fg: rgb(15, 23, 42),
            fgMuted: rgb(71, 85, 105),
            fgSubtle: rgb(100, 116, 139),
            border: rgb(226, 232, 240),
            borderStrong: rgb(203, 213, 225),
            accent: rgb(65, 139, 202),
            accentFg: rgb(255, 255, 255),
            success: rgb(22, 132, 41),
            warning: rgb(217, 119, 6),
            danger: rgb(220, 38, 68),
        }
    }
}

pub struct ThemeRust {
    dark: bool,
    bg: QColor,
    bgSubtle: QColor,
    bgPanel: QColor,
    fg: QColor,
    fgMuted: QColor,
    fgSubtle: QColor,
    border: QColor,
    borderStrong: QColor,
    accent: QColor,
    accentFg: QColor,
    success: QColor,
    warning: QColor,
    danger: QColor,
    radiusSm: i32,
    radiusMd: i32,
    radiusLg: i32,
    radiusXl: i32,
    radius2xl: i32,
    space: i32,
    fontFamily: QString,
    monoFamily: QString,
}

impl Default for ThemeRust {
    fn default() -> Self {
        let p = palette(true);
        Self {
            dark: true,
            bg: p.bg,
            bgSubtle: p.bgSubtle,
            bgPanel: p.bgPanel,
            fg: p.fg,
            fgMuted: p.fgMuted,
            fgSubtle: p.fgSubtle,
            border: p.border,
            borderStrong: p.borderStrong,
            accent: p.accent,
            accentFg: p.accentFg,
            success: p.success,
            warning: p.warning,
            danger: p.danger,
            radiusSm: 6,
            radiusMd: 8,
            radiusLg: 12,
            radiusXl: 16,
            radius2xl: 20,
            space: 4,
            fontFamily: QString::from("Inter"),
            monoFamily: QString::from(if cfg!(target_os = "macos") {
                "Menlo"
            } else if cfg!(target_os = "windows") {
                "Consolas"
            } else {
                "monospace"
            }),
        }
    }
}

impl qobject::Theme {
    pub fn setDarkMode(self: Pin<&mut Self>, dark: bool) {
        let mut this = self;
        let p = palette(dark);
        this.as_mut().set_dark(dark);
        this.as_mut().set_bg(p.bg);
        this.as_mut().set_bgSubtle(p.bgSubtle);
        this.as_mut().set_bgPanel(p.bgPanel);
        this.as_mut().set_fg(p.fg);
        this.as_mut().set_fgMuted(p.fgMuted);
        this.as_mut().set_fgSubtle(p.fgSubtle);
        this.as_mut().set_border(p.border);
        this.as_mut().set_borderStrong(p.borderStrong);
        this.as_mut().set_accent(p.accent);
        this.as_mut().set_accentFg(p.accentFg);
        this.as_mut().set_success(p.success);
        this.as_mut().set_warning(p.warning);
        this.as_mut().set_danger(p.danger);
    }
}
