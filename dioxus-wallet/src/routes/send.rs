use dioxus::prelude::*;
use dioxus_router::{use_router};

use crate::components::globals::{Header};

pub fn SendComponent(cx: Scope) -> Element {
    let sender_address = use_state(&cx, || {
        "0x853Be3012eCeb1fC9Db70ef0Dc85Ccf3b63994BE".to_string()
    });
    let amount = use_state(&cx, || "".to_string());
    let is_loading = use_state(&cx, || false);
    // let amountInDollar = use_state(&cx, || "0.00".to_string()); // Todo phase 2
    let router = use_router(&cx);
    if **is_loading {
        None
        // cx.render(rsx!(Loader {}))
    } else {
        cx.render(rsx! {
          div{
            class:"main-container",
            div{
                class:"header-container",
            Header{
                title:"Send",
                onbackclick: move |_|  router.push_route("/", None, None),
            }
        }
            div{class:"asset-icon-container",
                    div{
                        class:"asset-icon-wrapper",
                        img{
                            class:"asset-image",
                            src:"https://img.icons8.com/ios-filled/50/000000/bitcoin.png"
                        }
                    }
        }
                    div {
                        class:"input-container",
                        input{
                            class:"input",
                            "type":"text",
                            value:"{sender_address}",
                            placeholder:"Recipient Address",
                            oninput: move |evt| sender_address.set(evt.value.clone()),
                        }
                        input{
                            class:"input",
                            value:"{amount}",
                            placeholder:"ETH Amount",
                            oninput: move |evt| match evt.value.clone().parse::<f64>(){
                                Ok(_) =>{ amount.set(evt.value.clone()) }
                                Err(e) => {amount.set("".to_string());
                                                println!("invalid Input {:?}",e);}
                            },
                        }
                        // div{"{amountInDollar}"} // Todo phase 2
                    }

                    div{
                        class:"asset-bottom-container",
                        button{
                            class:"button",
                            onclick: move |_| { println!("{:?} {:?}", sender_address, amount )},
                            "NEXT",
                        }

                    }



          }
        })
    }
}
