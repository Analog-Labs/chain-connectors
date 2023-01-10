use crate::components::alerts::{Alert, ALERTS};
use crate::components::button::{Button, LinkButton};
use crate::components::common::Header;
use crate::components::loader::LOADER;
use crate::helpers::display_loader;
use crate::state::{use_chain_from_route, Chain};
use anyhow::Result;
use dioxus::prelude::*;
use dioxus_router::use_router;
use fermi::*;

#[allow(non_snake_case)]
#[inline_props]
pub fn Txns(cx: Scope) -> Element {
    let chain = use_chain_from_route(cx);
    let info = chain.info();
    let icon = info.icon.to_str().unwrap();
    let state = chain.use_state(cx).read();
    let alerts = use_atom_ref(cx, ALERTS);
    let router = use_router(cx);
    let loader_state = use_set(cx, LOADER).clone();
    cx.render(rsx! {
        div {
            class: "main-container",
            Header {
                onbackclick:|_| router.replace_route("/", None, None),
                title: "{info.config.network.blockchain}"
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
                        router.navigate_to(&format!("/scan/{}", info.chain));
                    },
                    uri: img!("receive.png")
                }
                LinkButton {
                    title: "Receive".to_string(),
                    onclick: move |_| {
                        router.navigate_to(&format!("/recv/{}", info.chain));
                    },
                    uri: img!("send.png")
                }
            }
            // Only for dev/testing purpose
            h5 {"i.e  Only for dev/testing purpose"}
            Button {
                title: "Get Test Tokens",
                onclick: move |_|  {
                    let alerts = alerts.clone();
                    let loader = loader_state.clone();
                    cx.spawn(async move {
                        display_loader(
                            loader,
                            faucet(alerts, info.chain, 3000000000000000)
                        ).await;
                    });
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

async fn fallible_faucet(chain: Chain, amount: u128) -> Result<()> {
    let wallet = rosetta_client::create_wallet(chain, None, None)?;
    wallet.faucet_dev(amount).await?;
    Ok(())
}
