#[allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_router::{use_router};
use crate::components::globals::*;
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
                            onClick: move |evt| {router.push_route(&format!("/selectAsset/{}", "send"), None, None)} ,
                            uri:"https://img.icons8.com/ios-glyphs/30/000000/filled-sent.png"
                         }
                         LinkButton {
                            title:"Receive".to_string(),
                            onClick: move |evt| {router.push_route(&format!("/selectAsset/{}", "receive"), None, None)} ,

                            uri:"https://img.icons8.com/external-xnimrodx-lineal-xnimrodx/64/000000/external-receive-passive-income-xnimrodx-lineal-xnimrodx.png"
                         }
                        }
                    }
                    div {
                        class:"listing-container",
                        div {
                            class:"list",
                             assets.iter().map(|item| rsx!(
                             
                                    ListingRow {
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





// --------------------------------------- //

// ------------ Listing Row --------------- //

#[derive(Props)]
pub struct ListingRowProps<'a> {
    assetName: &'a str,
    assetSymbol: &'a str,
    marketCap: &'a str,
    fiatPrice: f64,
    nativePrice: f64,
    assetIconUri: &'a str
}

pub fn ListingRow<'a>(cx: Scope<'a, ListingRowProps<'a>>) -> Element {
    cx.render(rsx! {
        div{
            class:"listing-row-container",
            div{
                class:"left-row-container",
                div{
                    class:"image-container",
                    img{
                        class:"row-image",
                        src:cx.props.assetIconUri
                    }
                }
                div{
                    class:"row-left-title-container",
                    div{
                        class:"row-title",
                        "{cx.props.assetName}",
                        img{
                            class:"arrow-down",
                            src:"https://img.icons8.com/ios-glyphs/30/000000/long-arrow-down.png"
                        }
                        div{
                            class:"row-title-2",
                            "{cx.props.marketCap}",
                        }
                    }
                    div{
                        class:"row-subtitle",
                        "{cx.props.assetSymbol}"
                    }
                }
            }
            div{
                class:"right-row-container",
                div{
                    class:"row-right-title-container",
                div {
                    class:"row-title",
                    "{cx.props.fiatPrice}",
                }
                div {
                    class:"row-subtitle",
                    "{cx.props.nativePrice}"
                }
            }

            }
        
        }

    })
}



