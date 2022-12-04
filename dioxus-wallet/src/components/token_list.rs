use crate::state::{Chain, CHAINS};
use dioxus::prelude::*;
use fermi::*;

pub static TOKENS: AtomRef<Vec<Chain>> = |_| vec![Chain::Btc, Chain::Eth, Chain::Dot];

#[allow(non_snake_case)]
#[inline_props]
pub fn TokenList<'a>(cx: Scope<'a>, onclick: EventHandler<'a, Chain>) -> Element {
    let tokens = use_atom_ref(&cx, TOKENS);
    cx.render(rsx! {
        ul {
            tokens.read().iter().copied().map(|chain| {
                rsx! {
                    TokenListItem {
                        chain: chain,
                        onclick: |chain| onclick.call(chain),
                    }
                }
            })
        }
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
        li {
            onclick: move |_| onclick.call(info.chain),
            height: "50px",
            div {
                style: "float: left;",
                img {
                    width: "25px",
                    src: icon,
                },
                "{info.name}",
            }
            div {
                style: "float: right;",
                "{state.balance}",
            }
        }
    })
}
