use crate::components::token_list::TokenList;
use dioxus::prelude::*;
use dioxus_router::Link;

#[allow(non_snake_case)]
#[inline_props]
pub fn Tokens(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            ul {
                li {
                    Link { to: "/send", "Send" }
                },
                li {
                    Link { to: "/recv", "Receive" }
                },
            },
            TokenList {},
        }
    })
}
