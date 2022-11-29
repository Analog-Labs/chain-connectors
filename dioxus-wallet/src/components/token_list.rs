use dioxus::prelude::*;
use fermi::*;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    name: &'static str,
    icon: &'static Path,
    balance: String,
}

pub static TOKENS: AtomRef<Vec<Token>> = |_| {
    vec![
        Token {
            name: "Bitcoin",
            icon: "btc.png".as_ref(),
            balance: "0".into(),
        },
        Token {
            name: "Ethereum",
            icon: "eth.png".as_ref(),
            balance: "0".into(),
        },
        Token {
            name: "Polkadot",
            icon: "dot.png".as_ref(),
            balance: "0".into(),
        },
    ]
};

#[allow(non_snake_case)]
#[inline_props]
pub fn TokenList(cx: Scope) -> Element {
    let tokens = use_atom_ref(&cx, TOKENS);
    cx.render(rsx! {
        ul {
            tokens.read().iter().map(|token| rsx! {
                TokenListItem {
                    token: token.clone(),
                }
            })
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
pub fn TokenListItem(cx: Scope, token: Token) -> Element {
    let name = &token.name;
    let balance = &token.balance;
    let icon = token.icon.to_str().unwrap();
    cx.render(rsx! {
        li {
            height: "50px",
            div {
                style: "float: left;",
                img {
                    width: "25px",
                    src: icon,
                },
                "{name}",
            }
            div {
                style: "float: right;",
                "{balance}",
            }
        }
    })
}
