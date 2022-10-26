use std::collections::{binary_heap, HashMap};

use dioxus::{
    core::exports::bumpalo::collections::vec,
    prelude::*,
    router::{Route, Router},
};

fn main() {
    dioxus::desktop::launch_cfg(APP, |c| {
        c.with_window(|w| {
            w.with_resizable(true)
                .with_inner_size(dioxus::desktop::wry::application::dpi::LogicalSize::new(
                    400.0, 800.0,
                ))
                .with_title("Wallet")
                .with_resizable(false)
        })
    });
}

static APP: Component<()> = |cx| {
    cx.render(rsx! {
                    style { [include_str!("./style.css")] }
                    Router{
                        Route{to:"/",Dashboard{}}
                        Route{to:"/send",SendScreen{}}
                        Route{to:"/receive",ReceiveScreen{}}
              }
    })
};

//-------- Dashboard Component ------ //

pub fn Dashboard(cx: Scope) -> Element {
    let dummyAssets = ["Eth", "Bitcoin", "Dot"];

    let balance = use_state(&cx, || 0.00);

    cx.render(rsx!(
        div {
                class:"main-container",
        div{
        class: "dashboard-container" ,
        h2{
            class:"chain-name",
            "Ethereum"
        }
        div{
            class: "balance-container",
            "${balance}",
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
                         dummyAssets.iter().map(|name| rsx!(
                            div {
                                onclick :move |evt| println!("clicked {:?}",evt),
                                class:"list-item",
                            div{
                                class:"asset-name",
                            "{name}",
                            }
                            div{
                                class:"asset-name",
                                "0",
                            }
                            }

                             ))
                    }
                }}))
}

//-------------------------------------//

//-------- Send Component -------- //

pub fn SendScreen(cx: Scope) -> Element {
    cx.render(rsx! {
      div{
          "send Screen",
          Link{to:"/", "goback"}
      }
    })
}

// ------Receive component------ //

pub fn ReceiveScreen(cx: Scope) -> Element {
    cx.render(rsx! {
      div{
          "Receive Screen",
          Link{to:"/", "goback"}
      }
    })
}

//------------------------------------//
