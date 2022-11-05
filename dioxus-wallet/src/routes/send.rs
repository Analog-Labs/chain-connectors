use dioxus::prelude::*;
use dioxus_router::Link;

pub fn SendComponent(cx: Scope) -> Element {
    let sender_address = use_state(&cx, || {
        "0x853Be3012eCeb1fC9Db70ef0Dc85Ccf3b63994BE".to_string()
    });
    let selectedAsset = use_state(&cx, || "");

    let amount = use_state(&cx, || "".to_string());
    println!("{:?}", amount);
    cx.render(rsx! {
      div{
        class:"main-container",
            div{
                class:"back-button-container",
                Link{
                    class:"back-button"
                    to:"/",
                    "X"
                         },
                     },
                h2{"SEND ETHEREUM"}
                div{
                    class:"send-input-container",
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
                        // oninput:move |evt| println!("{:?}",evt.value.parse::<f64>())
                        oninput: move |evt| match evt.value.clone().parse::<f64>(){
                            Ok(_) =>{ amount.set(evt.value.clone()) }
                            Err(e) => {amount.set("".to_string());
                                            println!("invalid Input {:?}",e);}
                        },
                    }
                    button{
                        class:"button",
                        onclick: move |evt| println!("{:?} {:?}", sender_address, amount ),
                        "SEND",
                    }
                }
      }
    })
}
