use crate::components::token_list::TokenList;
use dioxus::prelude::*;
use dioxus_router::use_router;

#[allow(non_snake_case)]
#[inline_props]
pub fn Tokens(cx: Scope) -> Element {
    let router = use_router(&cx);
    cx.render(rsx! {
        div {
            TokenList {
                onclick: |chain| router.navigate_to(&format!("/txns/{}", chain)),
            },
        }
    })
}
