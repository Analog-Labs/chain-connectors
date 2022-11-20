
use crate::components::globals::*;
use crate::components::listing_rows::{SingleSelectListingRow,MultiSelectListingRow};
use dioxus::prelude::*;
use dioxus_router::{use_router, use_route};
use fermi::{use_read, use_set};

use super::dashboard::ASSETS;

pub fn AddAssets(cx: Scope) -> Element {

    let router = use_router(&cx);
    let   assets_state = use_read(&cx, ASSETS);
    let assets_set = use_set(&cx,ASSETS);

    cx.render(rsx! {

        div {
            class:"main-container",
            div{
                    class:"header-container",
                Header{
                    title:"Add Assets",
                    onbackclick: move |evt|  router.push_route("/", None, None),

                }
            }
            div{
                class:"search-input-container",
                input{
                    class:"input",
                    value:"",
                    placeholder:"Search"
                }
            }
            div {
                class:"add-asset-listing-container",
                assets_state.iter().enumerate().map(|(id, asset)| rsx!{

                    MultiSelectListingRow {
                            assetName:asset.assetName,
                            nativePrice:0.0,
                            assetIconUri:"https://img.icons8.com/ios-filled/50/000000/bitcoin.png",
                            isSelected:asset.isSelected,
                            onSelect: move |_| {
                                let mut updated_assets = assets_state.clone().to_owned();
                                updated_assets[id].isSelected = true;
                                assets_set(updated_assets)
                            },
                            } ,
                        
            })
            }
            Button{
                title:"Save",
            }
           
        }
    })
}







pub struct selectAssetType {
    assetName: String,
    nativePrice: f64,
    symbol: String,
}

pub fn SelectAsset(cx: Scope) -> Element {
    let mut dummySelectAssets = vec![
        selectAssetType {
            assetName: "Bitcoin".to_string(),
            nativePrice: 1.1,
            symbol: "BTC".to_string(),
        },
        selectAssetType {
            assetName: "Ethereum".to_string(),
            nativePrice: 1.2,
            symbol: "ETH".to_string(),
        },
        selectAssetType {
            assetName: "Polkadot".to_string(),
            nativePrice: 1.2,
            symbol: "DOT".to_string(),
        },
    ];
    let router = use_router(&cx);
    let assets = use_state(&cx, || dummySelectAssets);
    let selectedAsset = use_state(&cx, || "");
    let route = use_route(&cx);
    let routeFromName = route.segment("from").unwrap();
    cx.render(rsx! {
            div {
                class:"main-container",
                div{
                        class:"header-container",
                    Header{
                        title:"{routeFromName}",
                        onbackclick: move |evt|  router.push_route("/", None, None),
                    }
                }
                div{
                    class:"search-input-container",
                    input{
                        class:"input",
                        value:"",
                        placeholder:"Search"
                    }
                }
                div {
                    class:"add-asset-listing-container",
                    assets.iter().enumerate().map(|(id, asset)| rsx!(
                        SingleSelectListingRow {
                                assetName:asset.assetName.as_str(),
                                nativePrice:asset.nativePrice,
                                assetIconUri:"https://img.icons8.com/ios-filled/50/000000/bitcoin.png",
                                onSelect: move |evt| {
                                   match routeFromName == "SEND"   {
                                    true => router.push_route("/send", None, None),
                                    false => router.push_route("/receive", None, None),
                                   }
                                },
                            }
                    ))
                }
             
            }
    })
}
