use crate::components::alerts::{Alert, ALERTS};
use crate::components::button::Button;
use crate::components::common::Header;
use crate::state::use_chain_from_route;
use dioxus::prelude::*;
use dioxus_router::use_router;
use fermi::*;

#[cfg(any(target_os = "android", target_os = "ios"))]
#[allow(non_snake_case)]
#[inline_props]
pub fn Scan(cx: Scope) -> Element {
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
            router.navigate_to(&format!("/send/{}/{}", chain, slice_string(address, ":")))
        }
        Some(Err(error)) => {
            let alert = Alert::error(error.to_string());
            alerts.write().push(alert);
            router.navigate_to(&format!("/txns/{}", chain));
        }
        None => {}
    }
    None
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[allow(non_snake_case)]
pub fn Scan(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx).info().chain;
    let router = use_router(&cx);
    let address = use_state(&cx, String::new);
    let alerts = use_atom_ref(&cx, ALERTS);
    cx.render(rsx! {
        div {
            class: "main-container",
            Header {
                onbackclick: move |_| router.navigate_to(&format!("/txns/{}", chain)),
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
            div {
                class: "container",
                Button {
                    onclick: move |_| {
                        if address.is_empty() {
                            let alert =
                                Alert::warning(" i.e Receiver address is required.".into());
                                alerts.write().push(alert);
                        }
                        else {
                            router.navigate_to(&format!("/send/{}/{}", chain, address));
                        }
                    }
                    title:"Next",
                }
            },
        },
    })
}
