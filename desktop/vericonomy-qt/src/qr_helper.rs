//! QR code + payment URI helpers for receive panels.

#![allow(non_snake_case)]

use base64::{engine::general_purpose::STANDARD, Engine};
use cxx_qt_lib::QString;
use image::Luma;
use qrcode::QrCode;

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
        type QrHelper = super::QrHelperRust;

        /// PNG data URL suitable for QML `Image.source`.
        #[qinvokable]
        fn pngDataUrl(self: &QrHelper, text: &QString) -> QString;

        /// BIP21-style payment URI (`verium:` / `vericoin:`).
        #[qinvokable]
        fn paymentUri(
            self: &QrHelper,
            coin: &QString,
            address: &QString,
            amount: f64,
            includeAmount: bool,
            label: &QString,
            message: &QString,
        ) -> QString;
    }
}

#[derive(Default)]
pub struct QrHelperRust;

impl qobject::QrHelper {
    pub fn pngDataUrl(&self, text: &QString) -> QString {
        let payload = text.to_string();
        if payload.is_empty() {
            return QString::default();
        }
        match encode_png_data_url(&payload) {
            Ok(url) => QString::from(&url),
            Err(_) => QString::default(),
        }
    }

    pub fn paymentUri(
        &self,
        coin: &QString,
        address: &QString,
        amount: f64,
        include_amount: bool,
        label: &QString,
        message: &QString,
    ) -> QString {
        let coin_id = vericonomy_desktop_host::CoinId::parse(&coin.to_string())
            .unwrap_or(vericonomy_desktop_host::CoinId::Verium);
        let amt = if include_amount && amount > 0.0 {
            Some(amount)
        } else {
            None
        };
        let label_s = label.to_string();
        let message_s = message.to_string();
        match vericonomy_desktop_host::commands::security::build_payment_uri(
            coin_id,
            &address.to_string(),
            amt,
            Some(&label_s).filter(|s| !s.is_empty()).map(String::as_str),
            Some(&message_s).filter(|s| !s.is_empty()).map(String::as_str),
        ) {
            Ok(uri) => QString::from(&uri),
            Err(_) => address.clone(),
        }
    }
}

fn encode_png_data_url(text: &str) -> Result<String, String> {
    let code = QrCode::new(text.as_bytes()).map_err(|e| e.to_string())?;
    let img = code
        .render::<Luma<u8>>()
        .min_dimensions(160, 160)
        .dark_color(Luma([0u8]))
        .light_color(Luma([255u8]))
        .build();
    let mut buf = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buf),
        image::ImageFormat::Png,
    )
    .map_err(|e| e.to_string())?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(buf)))
}
