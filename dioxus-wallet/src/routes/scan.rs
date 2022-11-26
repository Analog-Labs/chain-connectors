use crate::qrcode::scan_qrcode;
use dioxus::prelude::*;
use dioxus_router::use_router;
use fermi::use_atom_ref;

use crate::components::alerts::ALERTS;

#[allow(non_snake_case)]
pub fn Scan(cx: Scope) -> Element {
    #[cfg(target_os = "ios")]
    dioxus_desktop::use_window(&cx).pop_view();
    let alerts = use_atom_ref(&cx, ALERTS);
    let router = use_router(&cx);
    let fut = use_future(&cx, (), move |_| scan_qrcode(&cx));
    let alert = match fut.value() {
        Some(Ok(url)) => Some(("success", url.to_string())),
        Some(Err(error)) => Some(("danger", error.to_string())),
        None => None,
    };
    if let Some(alert) = alert {
        alerts.write().push(alert);
        router.pop_route();
    }
    None
}
