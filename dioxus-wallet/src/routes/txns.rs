use crate::components::alerts::{Alert, ALERTS};
use crate::components::button::LinkButton;
use crate::components::common::Header;
use crate::components::loader::LOADER;
use crate::components::txn_list::TransactionList;
use crate::helpers::display_loader;
use crate::state::{use_chain_from_route, Chain};
use anyhow::Result;
use dioxus::prelude::*;
use dioxus_router::use_router;
use fermi::*;
use futures::stream::StreamExt;
use rosetta_client::types::BlockTransaction;

#[allow(non_snake_case)]

pub fn Txns(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx);
    let info = chain.info();
    let icon = info.icon.to_str().unwrap();
    let state = chain.use_state(&cx).read();
    let alerts = use_atom_ref(&cx, ALERTS);
    let router = use_router(&cx);
    let loader_state = use_set(&cx, LOADER).clone();
    let transactions_state: &UseRef<Vec<BlockTransaction>> = use_ref(&cx, Vec::new);
    cx.use_hook(|| {
        let alerts = alerts.clone();
        let transactions_state = transactions_state.clone();
        cx.spawn(async move {
            fetch_transactions(alerts.clone(), info.chain, transactions_state).await;
        })
    });
    cx.render(rsx! {
        div {
            class: "main-container",
            Header {
                onbackclick:|_| router.replace_route("/", None, None),
                title: "{info.config.blockchain}"
            }
            div {
                class: "token-icon-container",
                div {
                    class: "token-icon-wrapper",
                    img {
                        class:"token-image",
                        src: "{icon}",
                    }
                }
            }
            div {
                class: "title",
                "{state.balance}"
            }
            div {
                class: "horizontal-button-container",
                LinkButton {
                    title: "Send".to_string(),
                    onclick: move |_| {
                        router.navigate_to(&format!("/scan/{}/{}", info.chain.blockchain, info.chain.network));
                    },
                    uri: img!("send.png")
                }
                LinkButton {
                    title: "Receive".to_string(),
                    onclick: move |_| {
                        router.navigate_to(&format!("/recv/{}/{}", info.chain.blockchain, info.chain.network));
                    },
                    uri: img!("receive.png")
                }
                LinkButton {
                    title: "Faucet".to_string(),
                    onclick: move |_| {
                    let alerts = alerts.clone();
                    let loader = loader_state.clone();
                    cx.spawn(async move {
                        display_loader(
                            loader,
                            faucet(alerts, info.chain, 3000000000000000)
                        ).await;
                    });
                    },
                    uri: img!("send.png")
                }
            }
            div {
                class:"transaction-container",
                        TransactionList {
                            chain_state:state.clone(),
                            transactions:transactions_state.read().clone(),
                            chain:info.chain
                     }
                }
        }
    })
}

async fn faucet(alerts: UseAtomRef<Vec<Alert>>, chain: Chain, amount: u128) {
    match fallible_faucet(chain, amount).await {
        Ok(_) => {
            alerts
                .write()
                .push(Alert::info("transfer successful".into()));
        }
        Err(error) => {
            alerts.write().push(Alert::error(error.to_string()));
        }
    }
}

async fn fetch_transactions(
    alerts: UseAtomRef<Vec<Alert>>,
    chain: Chain,
    _transactions: UseRef<Vec<BlockTransaction>>,
) {
    match fallible_transactions(chain).await {
        Ok(fetched_transactions) => _transactions.set(fetched_transactions),
        Err(error) => alerts.write().push(Alert::error(error.to_string())),
    }
}

async fn fallible_faucet(chain: Chain, amount: u128) -> Result<()> {
    let wallet = crate::worker::create_wallet(chain)?;
    wallet.faucet(amount).await?;
    Ok(())
}

async fn fallible_transactions(chain: Chain) -> Result<Vec<BlockTransaction>> {
    let wallet = crate::worker::create_wallet(chain)?;
    Ok(wallet
        .transactions(1000)
        .next()
        .await
        .transpose()?
        .unwrap_or_default())
}
