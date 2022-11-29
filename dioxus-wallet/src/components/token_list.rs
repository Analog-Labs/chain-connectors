use crate::worker::Action;
use dioxus::prelude::*;
use fermi::*;
use rosetta_client::Chain;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    chain: Chain,
    name: &'static str,
    icon: &'static Path,
    balance: String,
}

impl Token {
    pub fn chain(&self) -> Chain {
        self.chain
    }

    pub fn set_balance(&mut self, balance: String) {
        self.balance = balance;
    }
}

pub static TOKENS: AtomRef<Vec<Token>> = |_| {
    vec![
        Token {
            chain: Chain::Btc,
            name: "Bitcoin",
            icon: "btc.png".as_ref(),
            balance: "0".into(),
        },
        Token {
            chain: Chain::Eth,
            name: "Ethereum",
            icon: "eth.png".as_ref(),
            balance: "0".into(),
        },
        Token {
            chain: Chain::Dot,
            name: "Polkadot",
            icon: "dot.png".as_ref(),
            balance: "0".into(),
        },
    ]
};

#[allow(non_snake_case)]
#[inline_props]
pub fn TokenList(cx: Scope) -> Element {
    use_coroutine_handle(&cx)
        .unwrap()
        .send(Action::SyncBalances);
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
