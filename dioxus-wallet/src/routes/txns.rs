use crate::state::use_chain_from_route;
use dioxus::prelude::*;
use dioxus_router::Link;

#[allow(non_snake_case)]
#[inline_props]
pub fn Txns(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx);
    let info = chain.info();
    let icon = info.icon.to_str().unwrap();
    let state = chain.use_state(&cx).read();
    cx.render(rsx! {
        div {
            Link { to: "/", "Back" },
            img {
                src: "{icon}",
            }
            "{info.config.network.blockchain}",
            "{state.balance}",
            ul {
                li {
                    Link { to: "/scan/{info.chain}", "Send" }
                },
                li {
                    Link { to: "/recv/{info.chain}", "Receive" }
                },
            },
        }
    })
}
