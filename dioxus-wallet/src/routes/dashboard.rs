#[allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_router::{use_router};
use crate::components::{globals::*, listing_rows::DashListingRow};
use rosetta_client::{createWalletEthereum, Wallet};

pub struct AssetsType {
    assetName: String,
    nativePrice: f64,
    fiatPrice: f64,
    assetSymbol: String,
    marketCap:String,
}

pub fn Dashboard(cx: Scope) -> Element {
    let dummy_assets = [
        AssetsType {
            assetName: "Bitcoin".to_string(),
            fiatPrice: 1.1,
            nativePrice: 1.2 ,
            assetSymbol:"BTC".to_string(),
            marketCap:"2.3%".to_string(),

        },
        AssetsType {
            assetName: "Ethereum".to_string(),
            fiatPrice: 1.1,
            nativePrice: 1.2 ,
            assetSymbol:"ETH".to_string(),
            marketCap:"2.3%".to_string(),
        },
        AssetsType {
            assetName: "Polkadot".to_string(),
            fiatPrice: 1.1,
            nativePrice: 1.2 ,
            assetSymbol:"Dot".to_string(),
            marketCap:"2.3%".to_string(),
        },
    ];
    let assets = use_state(&cx, || dummy_assets);
    let balance = use_state(&cx, || 2.30);
    let account_address = use_state(&cx, || {
        "0x853Be3012eCeb1fC9Db70ef0Dc85Ccf3b63994BE".to_string()
    });
    let router = use_router(&cx);

        // For Testing purpose Only 
    use_effect(&cx, (), |_| async move {
          if let Ok(wallet)  = rosetta_client::createWalletEthereum().await {
            println!("{}",wallet.public_key().hex_bytes)
          }else {
            println!("Error case while wallet creation  ");
          }
        }
        );

    cx.render(rsx!(
            div {
                 class:"main-container",
                div {
                    class: "dashboard-container" ,
                    h2 {"$ {balance}"}
                    div { class:"wallet-name", "My Wallet" }
                    div {
                         class:"button-container",
                         LinkButton {
                            title:"Send".to_string(),
                            onClick: move |evt| {router.push_route(&format!("/selectAsset/{}", "SEND"), None, None)} ,
                            uri:"https://img.icons8.com/ios-glyphs/30/000000/filled-sent.png"
                         }
                         LinkButton {
                            title:"Receive".to_string(),
                            onClick: move |evt| {router.push_route(&format!("/selectAsset/{}", "RECEIVE"), None, None)} ,

                            uri:"https://img.icons8.com/external-xnimrodx-lineal-xnimrodx/64/000000/external-receive-passive-income-xnimrodx-lineal-xnimrodx.png"
                         }
                        }
                    }
                    div {
                        class:"listing-container",
                        div {
                            class:"list",
                             assets.iter().map(|item| rsx!(
                             
                                    DashListingRow {
                                    assetName:item.assetName.as_str(),
                                    assetSymbol: item.assetSymbol.as_str(),
                                    marketCap:item.marketCap.as_str(),
                                    fiatPrice:item.fiatPrice,
                                    nativePrice:item.nativePrice,
                                    assetIconUri:"https://img.icons8.com/ios-filled/50/000000/bitcoin.png"
                                    

                                }

                                 ))
                        }
                    }
                    LinkButton {
                        onClick: move |evt| {router.push_route("/addAsset", None, None)} ,
                        uri: "https://img.icons8.com/ios-glyphs/90/000000/plus-math.png",
                    }

                
                }))
}







