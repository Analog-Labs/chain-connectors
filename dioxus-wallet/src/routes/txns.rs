use crate::state::use_chain_from_route;
use dioxus::prelude::*;
use dioxus_router::Link;

#[allow(non_snake_case)]
#[inline_props]
pub fn Txns(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx);
    let info = chain.info();
    let icon = info.icon.to_str().unwrap();
    cx.render(rsx! {
        div {
            img {
                src: "{icon}",
            }
            "{info.name}",
            ul {
                li {
                    Link { to: "/send/{info.chain}", "Send" }
                },
                li {
                    Link { to: "/recv/{info.chain}", "Receive" }
                },
            },
        }
    })
}
