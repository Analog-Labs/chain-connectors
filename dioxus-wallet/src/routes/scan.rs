use crate::state::use_chain_from_route;
use dioxus::prelude::*;
use dioxus_router::use_router;

#[cfg(any(target_os = "android", target_os = "ios"))]
#[allow(non_snake_case)]
#[inline_props]
pub fn Scan(cx: Scope) -> Element {
    use crate::components::alerts::{Alert, ALERTS};
    use crate::qrcode::scan_qrcode;
    use fermi::*;

    #[cfg(target_os = "ios")]
    dioxus_desktop::use_window(&cx).pop_view();
    let chain = use_chain_from_route(&cx).info().chain;
    let alerts = use_atom_ref(&cx, ALERTS);
    let router = use_router(&cx);
    let fut = use_future(&cx, (), move |_| scan_qrcode(&cx));
    match fut.value() {
        Some(Ok(address)) => router.navigate_to(&format!("/send/{}/{}", chain, address)),
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
#[inline_props]
pub fn Scan(cx: Scope) -> Element {
    use dioxus_router::Link;

    let chain = use_chain_from_route(&cx).info().chain;
    let router = use_router(&cx);
    let address = use_state(&cx, String::new);
    cx.render(rsx! {
        div {
            Link { to: "/txns/{chain}", "Back" },
            label {
                r#for: "address",
                "Address: ",
            },
            input {
                id: "address",
                r#type: "text",
                value: "{address}",
                autofocus: true,
                oninput: move |e| address.set(e.value.clone()),
            },
            button {
                onclick: move |_| router.navigate_to(&format!("/send/{}/{}", chain, address)),
                "Next",
            }
        }
    })
}
