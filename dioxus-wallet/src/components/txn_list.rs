use crate::helpers::convert_to_highest_unit;
use crate::state::{Chain, ChainState};
use dioxus::prelude::*;
use rosetta_client::types::BlockTransaction;

#[allow(non_snake_case)]
#[inline_props]
pub fn TransactionList(
    cx: Scope,
    transactions: Vec<BlockTransaction>,
    chain_state: ChainState,
    chain: Chain,
) -> Element {
    cx.render(rsx! {
          transactions.iter().map(|tx| {
               if let Some(metadata) = tx.transaction.metadata.clone(){
                    let sender = metadata["from"].clone();
                    let amount = metadata["amount"].as_str().unwrap();
                    let is_sender = chain_state.account.eq(&sender);
                    let converted_amount = convert_to_highest_unit(
                      amount.to_string(), *chain).unwrap_or_else(|_|{"0".to_string()});
                    let (title,address,icon) = match is_sender {
                          true => (
                            "SENT",
                            format!("To {}", metadata["to"].as_str().unwrap()),
                            img!("sent-txn.png")
                          ),
                          false => (
                            "RECEIVED",
                            format!("From {}", metadata["from"].as_str().unwrap()),
                            img!("receive-txn.png")
                          )
                    };
                    rsx! {
                          div {
                              class: "transaction-item",
                              div {
                                class: "tx-icon-container",
                                img {
                                  class: "tx-icon",
                                  src: icon,
                                }
                              }
                              div {
                                class: "tx-content-container",
                                div {
                                  class: "tx-title",
                                  "{title}"
                                }
                                div {
                                  class: "txn-address",
                                  "{address}"
                                }
                              }
                              div {
                                class: "tx-amount",
                                "{converted_amount}"
                              }
                          }
                    }
                } else {
                   rsx!{{}}
                }
            })
    })
}
