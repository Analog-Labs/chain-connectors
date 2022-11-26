#![allow(dead_code, non_snake_case)]

use crate::components::globals::*;
use crate::components::listing_rows::{MultiSelectListingRow, SingleSelectListingRow};
use dioxus::prelude::*;
use dioxus_router::{use_route, use_router};
use fermi::{use_read, use_set};

use super::dashboard::ASSETS;

pub fn AddAssets(cx: Scope) -> Element {
    let router = use_router(&cx);
    let assets_state = use_read(&cx, ASSETS);
    let assets_set = use_set(&cx, ASSETS);

    cx.render(rsx! {
        div {
            class:"main-container",
            div{
                    class:"header-container",
                Header{
                    title:"Add Assets",
                    onbackclick: move |_|  router.push_route("/", None, None),
                }
            }
            div {
                class:"add-asset-listing-container",
                assets_state.iter().enumerate().map(|(id, asset)| rsx!{
                        MultiSelectListingRow {
                            assetName:asset.assetName.as_str(),
                            assetIconUri:"https://img.icons8.com/ios-filled/50/000000/bitcoin.png",
                            isSelected:asset.isSelected,
                            onSelect: move |_| {
                                let mut updated_assets =  assets_state.clone();
                                let is_selected = updated_assets[id].isSelected;
                                    if  is_selected {
                                        updated_assets[id].isSelected = false;
                                    }else {
                                        updated_assets[id].isSelected = true;
                                    }
                                    assets_set(updated_assets);
                                    }}
            })
            }
            Button{
                onclick: move |_| { router.push_route("/", None, None)},
                title:"Save",
            }

        }
    })
}

pub fn SelectAsset(cx: Scope) -> Element {
    let router = use_router(&cx);
    let assets_state = use_read(&cx, ASSETS);
    let route = use_route(&cx);
    let route_from_name = route.segment("from").unwrap();
    cx.render(rsx! {
            div {
                class:"main-container",
                div{
                        class:"header-container",
                    Header{
                        title:"{route_from_name}",
                        onbackclick: move |_|  router.push_route("/", None, None),
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
                    assets_state.iter().enumerate().map(|(_, asset)| rsx!(
                        SingleSelectListingRow {
                                assetName:asset.assetName.as_str(),
                                nativePrice:asset.nativePrice.to_string(),
                                assetIconUri:"https://img.icons8.com/ios-filled/50/000000/bitcoin.png",
                                onSelect: move |_| {
                                   match route_from_name == "SEND"   {
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
