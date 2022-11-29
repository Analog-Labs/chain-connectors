use crate::components::token_list::TOKENS;
use dioxus::prelude::*;
use dioxus_router::{use_route, Link};
use fermi::*;
use rosetta_client::Chain;

#[allow(non_snake_case)]
#[inline_props]
pub fn Send(cx: Scope) -> Element {
    let tokens = use_atom_ref(&cx, TOKENS);
    let route = use_route(&cx);
    let chain: Chain = route.last_segment().unwrap().parse().unwrap();
    let token = tokens
        .read()
        .iter()
        .find(|token| token.chain() == chain)
        .cloned()
        .unwrap();
    cx.render(rsx! {
        div {
            Link { to: "/", "Back" },
            "Send {token.name()}"
        }
    })
}
