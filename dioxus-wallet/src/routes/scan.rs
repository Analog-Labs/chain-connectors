use crate::components::alerts::{Alert, ALERTS};
use crate::components::button::{Button, LinkButton};
use crate::components::common::Header;
use crate::state::use_chain_from_route;
use dioxus::prelude::*;
use dioxus_router::use_router;
use fermi::*;
#[cfg(any(target_os = "android", target_os = "ios"))]
#[derive(Props)]
pub struct ScanCodeProps<'a> {
    on_scan_result: EventHandler<'a, String>,
}
#[cfg(any(target_os = "android", target_os = "ios"))]
#[allow(non_snake_case)]
pub fn ScanCode<'a>(cx: Scope<'a, ScanCodeProps<'a>>) -> Element {
    use crate::helpers::slice_string;
    use crate::qrcode::scan_qrcode;
    #[cfg(target_os = "ios")]
    dioxus_desktop::use_window(&cx).pop_view();
    let chain = use_chain_from_route(&cx).info().chain;
    let alerts = use_atom_ref(&cx, ALERTS);
    let router = use_router(&cx);
    let fut = use_future(&cx, (), move |_| scan_qrcode(&cx));
    match fut.value() {
        Some(Ok(address)) => {
            cx.props.on_scan_result.call(slice_string(address, ":"));
        }
        Some(Err(error)) => {
            let alert = Alert::error(error.to_string());
            alerts.write().push(alert);
            router.navigate_to(&format!("/txns/{}/{}", chain.blockchain, chain.network));
        }
        None => {}
    }
    None
}

#[allow(non_snake_case)]
pub fn Scan(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx).info().chain;
    let router = use_router(&cx);
    let address = use_state(&cx, String::new);
    let alerts = use_atom_ref(&cx, ALERTS);
    let show_qr_code_scanner = use_state(&cx, || false);
    let is_mobile_device = cfg!(any(target_os = "android", target_os = "ios"));
    cx.render(rsx! {
        div {
            class: "main-container",
            Header {
                onbackclick: move |_| router.navigate_to(&format!("/txns/{}/{}", chain.blockchain, chain.network)),
                title: "SEND"
            },
            div {
                class: "container",
                div {
                    class: "label",
                    "Receiver Address:"
                },
                input {
                    id: "address",
                    r#type: "text",
                    class: "input",
                    value: "{address}",
                    autofocus: true,
                    oninput: move |e| address.set(e.value.clone())
                },
            },
            is_mobile_device.then(|| {
                rsx!(
                    LinkButton {
                        onclick: move |_| {
                            show_qr_code_scanner.set(!show_qr_code_scanner);
                        },
                        uri:img!("qrcode.png"),
                        title:"SCAN".to_string(),
                    }
                )
            })
            div {
                class: "container",
                Button {
                    onclick: move |_| {
                        if address.is_empty() {
                            let alert =
                                Alert::warning("i.e Receiver address is required.".into());
                                alerts.write().push(alert);
                        }
                        else {
                            router.navigate_to(&format!("/send/{}/{}/{}", chain.blockchain, chain.network, address));
                        }
                    }
                    title:"Next",
                }
            },
            show_qr_code_scanner.then(|| {
            #[cfg(any(target_os = "android", target_os = "ios"))]
                rsx!(
                    ScanCode {
                        on_scan_result:  |value| {
                            address.set(value);
                            show_qr_code_scanner.set(false);
                        }
                })
            })
        },
    })
}
