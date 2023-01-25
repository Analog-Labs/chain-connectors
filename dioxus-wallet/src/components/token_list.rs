use crate::state::{Chain, CHAINS};
use dioxus::prelude::*;
use fermi::*;

pub static TOKENS: AtomRef<Vec<Chain>> = |_| Chain::CHAINS.to_vec();

#[allow(non_snake_case)]
#[inline_props]
pub fn TokenList<'a>(cx: Scope<'a>, onclick: EventHandler<'a, Chain>) -> Element {
    let tokens = use_atom_ref(&cx, TOKENS);
    cx.render(rsx! {
        tokens.read().iter().copied().map(|chain| {
            rsx! {
                TokenListItem {
                    chain: chain,
                    onclick: |chain| onclick.call(chain),
                }
            }
        })
    })
}

#[allow(non_snake_case)]
#[inline_props]
fn TokenListItem<'a>(cx: Scope<'a>, chain: Chain, onclick: EventHandler<'a, Chain>) -> Element {
    let chain = CHAINS.get(chain).unwrap();
    let info = chain.info();
    let state = chain.use_state(&cx).read();
    let icon = info.icon.to_str().unwrap();
    cx.render(rsx! {
        div {
            class: "token-list-item",
            onclick: move |_| onclick.call(info.chain),
            div {
                class: "list-item-left-container",
                div {
                    class: "list-item-image-container",
                    img {
                        class: "list-item-image",
                        src: icon,
                    }
                }
                div {
                    class: "list-item-title-container",
                    div {
                        class: "list-item-title",
                        "{info.config.blockchain}",
                    }
                }
            }
            div {
                class: "list-item-right-container",
                 div {
                    "{state.balance}"
                }
            }
        }
    })
}
