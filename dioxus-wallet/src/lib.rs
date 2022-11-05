use dioxus::prelude::*;
use dioxus_router::{ Route, Router};

mod qrcode;


mod components;
mod routes;

use crate::routes::assets::*;
use crate::routes::dashboard::*;
use crate::routes::receive::*;
use crate::routes::send::*;

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
    let cfg = dioxus_desktop::Config::new().with_window(
        dioxus_desktop::WindowBuilder::new()
            .with_resizable(false)
            .with_inner_size(dioxus_desktop::tao::dpi::LogicalSize::new(400.0, 800.0))
            .with_title("Wallet"),
    );
    dioxus_desktop::launch_cfg(app, cfg);
}

#[cfg(target_family = "wasm")]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    dioxus_web::launch(app);
}

static app: Component<()> = |cx| {
    cx.render(rsx! {
                  Router {
                        style {
                        [include_str!("./style.css"),
                        include_str!("./styles/button.css")]
                    }

                      Route{to:"/",Dashboard{}}
                      Route{to:"/send",SendComponent{}}
                      Route{to:"/receive",ReceiveComponent{}}
                      Route{to:"/addAsset",AddAssets{}}
                      Route{to:"/selectAsset/:from",SelectAsset{}}
            }
    })
};
 
