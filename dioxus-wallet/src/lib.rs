#![allow(clippy::derive_partial_eq_without_eq)]
use crate::components::alerts::Alerts;
use crate::routes::*;
use dioxus::prelude::*;
use dioxus_router::{Route, Router};

mod components;
mod qrcode;
mod routes;
mod state;
mod worker;

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
    // TODO: don't unwrap
    worker::use_chain_workers(&cx).unwrap();
    cx.render(rsx! {
        Alerts {},
        Router {
            style {
                include_str!("../assets/bootstrap-alert.css")
            }
            Route { to: "/", Tokens {} }
            Route { to: "/txns/:chain", Txns {} }
            Route { to: "/send/:chain", Send {} }
            Route { to: "/recv/:chain", Recv {} }
            Route { to: "/scan/:chain", Scan {} }
        }
    })
}
