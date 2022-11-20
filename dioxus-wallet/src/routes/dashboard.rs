use std::borrow::BorrowMut;

#[allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_router::{use_router};
use fermi::{Atom, use_read, use_set};
use crate::{components::{globals::*, listing_rows::DashListingRow}, EthWalletContext};


pub(crate) static ASSETS: Atom<Vec<AssetsType>> = |_|  vec![AssetsType {
    assetName:"Ethereum",
    nativePrice:"0.0",
    assetSymbol:"ETH",
    isSelected:false
}];


#[derive(Clone,Copy)]

pub struct AssetsType<'a> {
    pub assetName: & 'a str,
    pub nativePrice:  & 'a  str,
    pub assetSymbol:  & 'a str,
    pub  isSelected: bool,

}




pub fn Dashboard(cx: Scope) -> Element {
    let eth_wallet_context =  cx.consume_context::<EthWalletContext>();
    let eth_wallet = eth_wallet_context.unwrap().instance;
    let set_assets = use_set(&cx, ASSETS);
    let assets_state = use_read(&cx, ASSETS);
   
    // let balance = use_future(&cx,(),|_| async move{

    //     let amount =  eth_wallet.balance().await;
      
    //   amount.unwrap()
    // });

let eth_account_address = use_state(&cx, || 
    eth_wallet.public_key().hex_bytes.clone()
);

println!("Ethereum Wallett Public Key {}",eth_account_address);
let router = use_router(&cx);

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

                            
                            assets_state.iter().enumerate().filter(|(_,item)| item.isSelected == true).map(|(id,item)| rsx!(
                                    DashListingRow {
                                    assetName:item.assetName,
                                    assetSymbol: item.assetSymbol,
                                    marketCap:"",
                                    fiatPrice:0.0,
                                    nativePrice:0.0,
                                    assetIconUri:"https://img.icons8.com/ios-filled/50/000000/bitcoin.png"
                                
                                }

                                 ))
                        }
                    }
                    LinkButton {
                        onClick: move |evt| {router.push_route("/addAsset", None, None)} ,
                        uri: "https://img.icons8.com/ios-glyphs/90/000000/plus-math.png",
                    }

                
                }})
}









