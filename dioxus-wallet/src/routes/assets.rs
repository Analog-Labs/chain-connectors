use crate::components::globals::*;
use crate::components::listing_rows::{SingleSelectListingRow,MultiSelectListingRow};
use dioxus::core::UiEvent;
use dioxus::events::*;
use dioxus::prelude::*;
use dioxus_router::{use_router, use_route};
use std::sync::Arc;


pub struct addAssetsType {
    assetName: String,
    nativePrice: f64,
    isSelected: String,
}

pub fn AddAssets(cx: Scope) -> Element {
    let mut dummyAddAssets = vec![
        addAssetsType {
            assetName: "Bitcoin".to_string(),
            nativePrice: 1.1,
            isSelected: "false".to_string(),
        },
        addAssetsType {
            assetName: "Ethereum".to_string(),
            nativePrice: 1.2,
            isSelected: "false".to_string(),
        },
        addAssetsType {
            assetName: "Polkadot".to_string(),
            nativePrice: 1.2,
            isSelected: "false".to_string(),
        },
    ];
    let assets = use_state(&cx, || dummyAddAssets);
    let router = use_router(&cx);

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
                class:"input-container",
                input{

                    class:"input",
                    value:"",
                    placeholder:"Search"
                }
            }
            div {
                class:"add-asset-listing-container",

                assets.iter().enumerate().map(|(id, asset)| rsx!(
                    MultiSelectListingRow {
                            assetName:asset.assetName.as_str(),
                            nativePrice:asset.nativePrice,
                            assetIconUri:"https://img.icons8.com/ios-filled/50/000000/bitcoin.png",
                            isSelected:asset.isSelected.as_str(),
                            onSelect: move |evt: UiEvent<FormData>| {println!("{:?}",id)} ,

                        }
                ))


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
    let name = route.segment("from").unwrap();
    println!{"{}",name}

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
                    class:"input-container",
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
                                   match name == "send"   {
                                    true => router.push_route("/send", None, None),
                                    false => router.push_route("/receive", None, None),
                                   }
                                } ,

                            }
                    ))


                }


                div {
                    onclick: move |evt| {},
                    "next",
                    

                }



            }

    })
}
