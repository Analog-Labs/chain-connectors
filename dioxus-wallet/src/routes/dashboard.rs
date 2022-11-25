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
            nativePrice: "0.0".to_string(),
            assetSymbol: "ETH".to_string(),
            isSelected: false,
        },
        AssetsType {
            assetName: "Bitcoin".to_string(),
            nativePrice: "0.0".to_string(),
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
    let eth_instance = wallet_context.clone().unwrap().eth;
    let btc_instance = wallet_context.clone().unwrap().btc;
    let set_assets = use_set(&cx, ASSETS);
    let assets_state = use_read(&cx, ASSETS);
    let router = use_router(&cx);
    let assets = cx.use_hook(|| assets_state.clone());

    if assets[0].isSelected {
        let eth_balance = use_future(&cx, (), |_| async move {
            let amount = eth_instance.balance().await;
            let amount_string = rosetta_client::amount_to_string(&amount.unwrap());
            amount_string.unwrap()
        });
        let eth_balance = match eth_balance.value() {
            Some(b) => b,
            None => "",
        };
        assets[0].nativePrice = eth_balance.to_string();
        set_assets(assets.clone());
    }

    if assets[1].isSelected {
        let btc_balance = use_future(&cx, (), |_| async move {
            let amount = btc_instance.balance().await;
            let amount_string = rosetta_client::amount_to_string(&amount.unwrap());
            amount_string.unwrap()
        });
        let btc_balance = match btc_balance.value() {
            Some(b) => b,
            None => "",
        };
        assets[1].nativePrice = btc_balance.to_string();
        set_assets(assets.clone());
    }

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
