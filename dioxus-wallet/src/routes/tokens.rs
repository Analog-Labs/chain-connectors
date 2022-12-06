use crate::components::{button::LinkButton, token_list::TokenList};
use dioxus::prelude::*;
use dioxus_router::use_router;

#[allow(non_snake_case)]
#[inline_props]
pub fn Tokens(cx: Scope) -> Element {
    let router = use_router(&cx);
    cx.render(rsx! {
        div {
            class:"main-container",
            div {
                class: "upper-container" ,
                div { class:"title", "Analog Wallet" }
                div {
                    class:"horizontal-button-container",
                    LinkButton {
                        title: "SEND".to_string(),
                        onclick: move |_| {
                            router.push_route(&format!("/selectAsset/{}", "SEND"), None, None)
                        },
                        uri: "https://img.icons8.com/ios-glyphs/30/000000/filled-sent.png"
                     }
                     LinkButton {
                        title: "RECEIVE".to_string(),
                        onclick: move |_| {
                            router.push_route(&format!("/selectAsset/{}", "RECEIVE"), None, None)
                        } ,
                        uri: "https://img.icons8.com/external-xnimrodx-lineal-xnimrodx/64/000000/external-receive-passive-income-xnimrodx-lineal-xnimrodx.png"
                     }
                }
            }
            div {
                class: "tokens-listing-container",
                div {
                    class:"list",
                    TokenList {
                        onclick: |chain| router.navigate_to(&format!("/txns/{}", chain)),
                    },

                }

            }
        }
    })
}
