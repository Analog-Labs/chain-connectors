use dioxus::prelude::*;
use dioxus_router::{Link, Route, Router};

mod receive;
mod send;

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn start_app() {
    use wry::android_binding;

    std::env::set_var("RUST_BACKTRACE", "1");
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
                        style { [include_str!("./style.css")] }

                      Route{to:"/",Dashboard{}}
                      Route{to:"/send",send::SendComponent{}}
                      Route{to:"/receive",receive::ReceiveComponent{}}
            }
    })
};

pub struct AssetsType {
    name: String,
    balance: f64,
    symbol: String,
}

pub fn Dashboard(cx: Scope) -> Element {
    let dummy_assets = [
        AssetsType {
            name: "Bitcoin".to_string(),
            balance: 1.1,
            symbol: "BTC".to_string(),
        },
        AssetsType {
            name: "Eth".to_string(),
            balance: 2.2,
            symbol: "ETH".to_string(),
        },
        AssetsType {
            name: "Dot".to_string(),
            balance: 2.2,
            symbol: "DOT".to_string(),
        },
    ];
    let assets = use_state(&cx, || dummy_assets);

    let balance = use_state(&cx, || 2.30);
    let account_address = use_state(&cx, || {
        "0x853Be3012eCeb1fC9Db70ef0Dc85Ccf3b63994BE".to_string()
    });

    cx.render(rsx!(
        div {
                class:"main-container",
        div{
        class: "dashboard-container" ,
        h1{
            class:"screen-title",

            "Ethereum"
        }

        h5{
            "{account_address}"
        }
        h2{
            "{balance} ETH",
        }
        div {
            class:"button-container",
            Link { class:"button", to: "/send", "SEND"},
            Link { class:"button", to: "/receive", "RECEIVE"}
        }
    }   div{
                    class:"listing-container",
                    div{
                        class:"listing-title",
                        "Tokens"}
                    div {
                        class:"list",
                         assets.iter().map(|item| rsx!(
                            div {
                                onclick :move |evt| println!("clicked {:?}",evt),
                                class:"list-item",
                            div{
                                class:"asset-name",
                            "{item.name}",
                            }
                            div{
                                class:"asset-name",
                                "{item.balance} {item.symbol}",
                            }
                            }

                             ))
                    }
                }}))
}
