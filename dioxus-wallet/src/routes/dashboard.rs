#![allow(dead_code, non_snake_case)]

use crate::{
    components::{globals::*, listing_rows::DashListingRow},
    WalletContext,
};
use dioxus::prelude::*;
use dioxus_router::use_router;
use fermi::{use_read, use_set, Atom};

pub static ASSETS: Atom<Vec<AssetsType>> = |_| {
    vec![
        AssetsType {
            assetName: "Ethereum".to_string(),
            nativePrice: "0.0 ETH".to_string(),
            assetSymbol: "ETH".to_string(),
            isSelected: false,
        },
        AssetsType {
            assetName: "Bitcoin".to_string(),
            nativePrice: "0.0 BTC".to_string(),
            assetSymbol: "BTC".to_string(),
            isSelected: false,
        },
    ]
};

#[derive(Clone)]
pub struct AssetsType {
    pub assetName: String,
    pub nativePrice: String,
    pub assetSymbol: String,
    pub isSelected: bool,
}

pub fn Dashboard(cx: Scope) -> Element {
    let wallet_context = cx.use_hook(|| cx.consume_context::<WalletContext>());
    let set_assets = use_set(&cx, ASSETS);
    let assets_state = use_read(&cx, ASSETS);
    let router = use_router(&cx);

    fn update_assets_balance(
        cx: &ScopeState,
        wallet_context: WalletContext,
        assets: Vec<AssetsType>,
    ) -> Vec<AssetsType> {
        let assets = cx.use_hook(|| assets);

        for item in assets.iter_mut() {
            if item.isSelected {
                let wallet = match item.assetSymbol.as_str() {
                    "ETH" => wallet_context.eth.clone(),
                    "BTC" => wallet_context.btc.clone(),
                    &_ => wallet_context.eth.clone(),
                };
                let balance = use_future(cx, (), |_| async move {
                    let amount = wallet.balance().await;
                    let amount_string = rosetta_client::amount_to_string(&amount.unwrap());
                    amount_string.unwrap()
                });
                let balance_in_string = match balance.value() {
                    Some(b) => b,
                    None => "",
                };
                item.nativePrice = balance_in_string.to_string();
            }
        }
        assets.to_vec()
    }

    let updated_assets =
        update_assets_balance(&cx, wallet_context.clone().unwrap(), assets_state.to_vec());
    set_assets(updated_assets);

    cx.render(
        rsx!{
            div {
                 class:"main-container",
                div {
                    class: "dashboard-container" ,
                    // h2 {"$ {balance}"} //Todo Dollar price MVP phase 2
                    div { class:"wallet-name", "My Wallet" }
                    div {
                         class:"button-container",
                         LinkButton {
                            title:"Send".to_string(),
                            onClick: move |_| {
                                router.push_route(&format!("/selectAsset/{}", "SEND"), None, None)
                            },
                            uri:"https://img.icons8.com/ios-glyphs/30/000000/filled-sent.png"
                         }
                         LinkButton {
                            title:"Receive".to_string(),
                            onClick: move |_| {
                                router.push_route(&format!("/selectAsset/{}", "RECEIVE"), None, None)
                            } ,
                            uri:"https://img.icons8.com/external-xnimrodx-lineal-xnimrodx/64/000000/external-receive-passive-income-xnimrodx-lineal-xnimrodx.png"
                         }
                        }
                    }
                    div {
                        class:"listing-container",
                        div {
                            class:"list",
                            assets_state.iter().enumerate().filter(|(_,item)| item.isSelected).map(|(_,item)| rsx!(
                                    DashListingRow {
                                    assetName:item.assetName.as_str(),
                                    assetSymbol: item.assetSymbol.as_str(),
                                    marketCap:"",
                                    fiatPrice:0.0,
                                    nativePrice:item.nativePrice.clone(),
                                    assetIconUri:"https://img.icons8.com/ios-filled/50/000000/bitcoin.png"
                                }
                                 ))
                        }
                    }
                    LinkButton {
                        onClick: move |_| {router.push_route("/addAsset", None, None)} ,
                        uri: "https://img.icons8.com/ios-glyphs/90/000000/plus-math.png",
                    }
                }})
}
