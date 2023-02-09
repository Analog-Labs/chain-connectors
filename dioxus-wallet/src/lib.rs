#![allow(clippy::derive_partial_eq_without_eq)]
use crate::components::{alerts::Alerts, loader::Loader};
use crate::routes::*;
use dioxus::prelude::*;
use dioxus_router::{Route, Router};
use rosetta_client::MnemonicStore;

#[macro_use]
mod assets;
mod components;
mod helpers;
mod qrcode;
mod routes;
mod state;
mod worker;

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn start_app() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Trace)
            .with_tag("dioxus_wallet"),
    );

    dioxus_desktop::wry::android_binding!(
        com_example,
        dioxus_1wallet,
        _start_app,
        dioxus_desktop::wry
    );
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
    use dioxus_desktop::Config;
    #[cfg(any(target_os = "android", target_os = "ios"))]
    std::env::set_var("RUST_BACKTRACE", "1");
    let config = Config::default().with_custom_protocol("asset".into(), assets::asset_handler);
    dioxus_desktop::launch_cfg(app, config);
}

#[cfg(target_family = "wasm")]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    let styles = rsx!(
        css!("buttons")
        css!("bootstrap-alert")
        css!("common")
        css!("listings")
        css!("inputs")
        css!("signup")
    );

    cx.render(rsx! {
        Alerts {},
        Loader{},
        Router {
            style { styles }
            if MnemonicStore::new(None).unwrap().exists() {
               worker::use_chain_workers(&cx).unwrap();
               rsx!(Route { to: "/", Tokens{}})
            } else {
                rsx!(Route { to: "/", Signup{}})
            }
            Route { to: "/txns/:chain", Txns {} }
            Route { to: "/send/:chain/:address", Send {} }
            Route { to: "/recv/:chain", Recv {} }
            Route { to: "/scan/:chain", Scan {} }
        }
    })
}
