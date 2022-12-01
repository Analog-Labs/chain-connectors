use crate::state::use_chain_from_route;
use dioxus::prelude::*;
use dioxus_router::Link;

#[allow(non_snake_case)]
#[inline_props]
pub fn Send(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx);
    let info = chain.info();
    cx.render(rsx! {
        div {
            Link { to: "/", "Back" },
            "Send {info.name}"
        }
    })
}
