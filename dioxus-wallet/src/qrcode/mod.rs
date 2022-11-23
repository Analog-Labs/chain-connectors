#![allow(dead_code, non_snake_case)]

use dioxus::prelude::*;
use qrcode::render::svg;
use qrcode::QrCode;

#[allow(non_snake_case)]
#[inline_props]
pub fn Qrcode<'a>(cx: Scope<'a>, data: &'a [u8]) -> Element<'a> {
    let code = QrCode::new(data).unwrap();
    let xml = code.render::<svg::Color>().build();
    let svg = xml
        .strip_prefix(r#"<?xml version="1.0" standalone="yes"?>"#)
        .unwrap_or(&xml);
    cx.render(rsx! {
        div {
            dangerous_inner_html: "{svg}",
        }
    })
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn scan_qrcode(_: &ScopeState) -> impl std::future::Future<Output = anyhow::Result<String>> {
    async move {
        anyhow::bail!("qr scanning is unsupported on this platform");
    }
}

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::scan_qrcode;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::scan_qrcode;
