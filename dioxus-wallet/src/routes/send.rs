use crate::components::alerts::{Alert, ALERTS};
use crate::components::button::Button;
use crate::components::common::Header;
use crate::components::loader::LOADER;
use crate::helpers::{convert_to_lowest_unit, display_loader};
use crate::state::{use_chain_from_route, Chain};
use anyhow::Result;
use dioxus::prelude::*;
use dioxus_router::{use_route, use_router, RouterService};
use fermi::*;
use fraction::BigDecimal;
use rosetta_client::crypto::address::Address;
use rosetta_client::RosettaAccount;

#[allow(non_snake_case)]
#[inline_props]
pub fn Send(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx);
    let info = chain.info();
    let amount = use_state(&cx, || BigDecimal::from(0));
    let alerts = use_atom_ref(&cx, ALERTS).clone();
    let router = use_router(&cx);
    let address = use_route(&cx).segment("address").unwrap().to_string();
    let address = Address::new(info.config.address_format, address);
    let loader_state = use_set(&cx, LOADER).clone();
    cx.render(rsx! {
        div {
            class: "main-container",
            Header{
                title:"send {info.config.blockchain}",
                onbackclick: move  |_| {
                    router.navigate_to(&format!("/txns/{}/{}", info.chain.blockchain, info.chain.network));
                }
            }
            div {
                class:"container",
                div {
                    class: "title",
                    "Unit: {info.config.currency_symbol.to_lowercase()}",
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
                        if amount.clone() == BigDecimal::from(0)  {
                            let alert =
                            Alert::warning("i.e amount is required.".into());
                            alerts.write().push(alert);
                        }
                        else {
                            let router = router.clone();
                            let alerts = alerts.clone();
                            let address = address.clone();
                            let amount = convert_to_lowest_unit(amount.get().clone(), info.chain);
                            let loader = loader_state.clone();
                            cx.spawn(async move {
                                display_loader(
                                    loader,
                                    transfer(router, alerts, info.chain, address, amount)
                                ).await;
                            })
                        }
                    }
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
            router.navigate_to(&format!("/txns/{}/{}", chain.blockchain, chain.network));
        }
        Err(error) => {
            alerts.write().push(Alert::error(error.to_string()));
        }
    }
}

async fn fallible_transfer(chain: Chain, address: Address, amount: u128) -> Result<()> {
    let wallet = crate::worker::create_wallet(chain)?;
    wallet.transfer(&address.to_rosetta(), amount).await?;
    Ok(())
}
