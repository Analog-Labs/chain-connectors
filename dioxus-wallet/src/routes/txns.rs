use crate::components::token_list::TOKENS;
use dioxus::prelude::*;
use dioxus_router::{use_route, Link};
use fermi::*;
use rosetta_client::Chain;

#[allow(non_snake_case)]
#[inline_props]
pub fn Txns(cx: Scope) -> Element {
    let tokens = use_atom_ref(&cx, TOKENS);
    let route = use_route(&cx);
    let chain: Chain = route.last_segment().unwrap().parse().unwrap();
    let token = tokens
        .read()
        .iter()
        .find(|token| token.chain() == chain)
        .cloned()
        .unwrap();
    let name = token.name();
    let icon = token.icon().to_str().unwrap();
    cx.render(rsx! {
        div {
            img {
                src: "{icon}",
            }
            "{name}",
            ul {
                li {
                    Link { to: "/send/{chain}", "Send" }
                },
                li {
                    Link { to: "/recv/{chain}", "Receive" }
                },
            },
        }
    })
}
