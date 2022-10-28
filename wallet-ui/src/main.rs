use arboard::Clipboard;
use dioxus::{
    prelude::*,
    router::{Route, Router},
};

mod receive;
mod send;

// ------ Functionalities ------ //
fn main() {
    dioxus::desktop::launch_cfg(APP, |c| {
        c.with_window(|w| {
            w.with_resizable(false)
                .with_inner_size(dioxus::desktop::wry::application::dpi::LogicalSize::new(
                    400.0, 800.0,
                ))
                .with_title("Wallet")
        })
    });
}

static APP: Component<()> = |cx| {
    cx.render(rsx! {
                    style { [include_str!("./style.css")] }
                    Router{
                        Route{to:"/",Dashboard{}}
                        Route{to:"/send",send::SendComponent{}}
                        Route{to:"/receive",receive::ReceiveComponent{}}
              }
    })
};

// ---------------------- //

//-------- Dashboard Component ------ //

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

//-------------------------------------//
