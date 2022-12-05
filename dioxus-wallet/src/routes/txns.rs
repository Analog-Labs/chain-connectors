use crate::components::alerts::{Alert, ALERTS};
use crate::state::{use_chain_from_route, Chain};
use anyhow::Result;
use dioxus::prelude::*;
use dioxus_router::Link;
use fermi::*;

#[allow(non_snake_case)]
#[inline_props]
pub fn Txns(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx);
    let info = chain.info();
    let icon = info.icon.to_str().unwrap();
    let state = chain.use_state(&cx).read();
    let alerts = use_atom_ref(&cx, ALERTS);
    cx.render(rsx! {
        div {
            Link { to: "/", "Back" },
            img {
                src: "{icon}",
            }
            "{info.config.network.blockchain}",
            "{state.balance}",
            ul {
                li {
                    Link { to: "/scan/{info.chain}", "Send" }
                },
                li {
                    Link { to: "/recv/{info.chain}", "Receive" }
                },
            },
            button {
                onclick: move |_| {
                    let alerts = alerts.clone();
                    cx.spawn(async move {
                        faucet(alerts, info.chain, 3000000000000000).await
                    });
                },
                "gimme some tokens",
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
        Err(error) => alerts.write().push(Alert::error(error.to_string())),
    }
}

async fn fallible_faucet(chain: Chain, amount: u128) -> Result<()> {
    let wallet = rosetta_client::create_wallet(chain, None, None)?;
    wallet.faucet_dev(amount).await?;
    Ok(())
}
