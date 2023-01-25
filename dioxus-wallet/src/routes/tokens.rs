use crate::components::token_list::TokenList;
use crate::state::Chain;
use dioxus::prelude::*;
use dioxus_router::use_router;

#[allow(non_snake_case)]
#[inline_props]
pub fn Tokens(cx: Scope) -> Element {
    let router = use_router(&cx);
    cx.render(rsx! {
        div {
            class: "main-container",
            div {
                class: "upper-container",
                div {
                    class:"title",
                    "Analog Wallet"
                }
            }
            div {
                class: "tokens-listing-container",
                div {
                    class: "list",
                    TokenList {
                        onclick: |chain: Chain| router.navigate_to(&format!("/txns/{}/{}", chain.blockchain, chain.network)),
                    },
                }
            }
        }
    })
}
