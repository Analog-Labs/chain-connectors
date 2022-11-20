use crate::qrcode::{scan_qrcode, Qrcode};
use dioxus::prelude::*;
use dioxus_router::{use_router, Link, Route, Router};
use fermi::*;

mod qrcode;

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn start_app() {
    use wry::android_binding;

    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Trace)
            .with_tag("dioxus_wallet"),
    );

    android_binding!(com_example, dioxus_1wallet, _start_app);
}

#[cfg(target_os = "android")]
fn _start_app() {
    if let Err(err) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(main)) {
        eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
        std::process::abort();
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn main() {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    std::env::set_var("RUST_BACKTRACE", "1");
    dioxus_desktop::launch(app);
}

#[cfg(target_family = "wasm")]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        Router {
            style {
                include_str!("../assets/bootstrap-alert.css")
            }
            Route { to: "/", Home {} }
            Route { to: "/scan", Scan {} }
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
fn Home(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            Alerts {}
            Qrcode {
                data: b"wallet://btc?address=bcrt1q38k5zlaxfumy7gqj200xsqdmyhnlj7wg8wqx9s"
            }
            Link { to: "/scan", "scan qr code" }
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
fn Scan(cx: Scope) -> Element {
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

static ALERTS: AtomRef<Vec<(&'static str, String)>> = |_| vec![];

#[allow(non_snake_case)]
#[inline_props]
fn Alerts(cx: Scope) -> Element {
    let alerts = use_atom_ref(&cx, ALERTS);
    cx.render(rsx! {
        div {
            alerts.read().iter().enumerate().map(|(i, (ty, msg))| rsx! {
                div {
                    class: "alert alert-{ty} alert-dismissible",
                    role: "alert",
                    div { "{msg}" }
                    button {
                        r#type: "button",
                        class: "close",
                        aria_label: "Close",
                        onclick: move |_| {
                            alerts.write().remove(i);
                        },
                        span {
                            aria_hidden: true,
                            "\u{00d7}"
                        }
                    }
                }
            })
        }
    })
}
