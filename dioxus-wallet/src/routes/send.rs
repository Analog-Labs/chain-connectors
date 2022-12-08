use crate::components::alerts::{Alert, ALERTS};
use crate::components::button::Button;
use crate::components::common::Header;
use crate::state::{use_chain_from_route, Chain};
use anyhow::Result;
use dioxus::prelude::*;
use dioxus_router::{use_route, use_router, RouterService};
use fermi::*;
use rosetta_client::crypto::address::Address;
use rosetta_client::signer::RosettaAccount;

#[allow(non_snake_case)]
#[inline_props]
pub fn Send(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx);
    let info = chain.info();
    let amount = use_state(&cx, || 0u128);
    let alerts = use_atom_ref(&cx, ALERTS).clone();
    let router = use_router(&cx);
    let address = use_route(&cx).segment("address").unwrap().to_string();
    let address = Address::new(info.config.address_format, address);
    cx.render(rsx! {
        div {
            class: "main-container",
            Header{
                title:"send {info.config.network.blockchain}",
                onbackclick: move  |_| {
                    router.navigate_to(&format!("/txns/{}", info.chain));
                }
            }
            div {
                class:"container",
                div {
                    class: "title",
                    "Unit: {info.config.unit}",
                }
                div {
                    class: "label",
                    "Amount:"
                }
                input {
                    class: "input",
                    id: "amount",
                    r#type: "number",
                    value: "{amount}",
                    autofocus: true,
                    oninput: move |e| {
                        if let Ok(value) = e.value.parse() {
                            amount.set(value);
                        }
                    }
                },
            }
            div {
                class:"container",
                Button {
                    onclick: move |_| {
                        let router = router.clone();
                        let alerts = alerts.clone();
                        let address = address.clone();
                        let amount = *amount.get();
                        cx.spawn(async move {
                            transfer(router, alerts, info.chain, address, amount).await;
                        });
                    },
                   title:"Send",
                }
            }
        }
    })
}

async fn transfer(
    router: RouterService,
    alerts: UseAtomRef<Vec<Alert>>,
    chain: Chain,
    address: Address,
    amount: u128,
) {
    match fallible_transfer(chain, address, amount).await {
        Ok(_) => {
            alerts
                .write()
                .push(Alert::info("transfer successful".into()));
            router.navigate_to(&format!("/txns/{}", chain));
        }
        Err(error) => alerts.write().push(Alert::error(error.to_string())),
    }
}

async fn fallible_transfer(chain: Chain, address: Address, amount: u128) -> Result<()> {
    let wallet = rosetta_client::create_wallet(chain, None, None)?;
    wallet.transfer(&address.to_rosetta(), amount).await?;
    Ok(())
}
