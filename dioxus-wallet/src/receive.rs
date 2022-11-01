use dioxus::prelude::*;
use dioxus_router::Link;

pub fn ReceiveComponent(cx: Scope) -> Element {
    let account_address = use_state(&cx, || {
        "0x853Be3012eCeb1fC9Db70ef0Dc85Ccf3b63994BE".to_string()
    });
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

                    h2{"RECEIVE ETHEREUM"},
                    h4{"Copy and share account address."}
                    h6{
                        "i.e Send only Eth to this address."
                    }

                    div{
                        class:"input-container",
                        input{
                            class:"input",
                            value:"{account_address}",
                            disabled:"true",
                        }

                        button{
                            class:"button",
                            onclick:move |evt| copy_to_clipboard(account_address.to_string()),
                            "COPY"
                        }

                    }
        }
    })
}

// ----- Functionalities ------//
#[cfg(not(target_family = "wasm"))]
fn copy_to_clipboard(string: String) {
    let mut clipboard = arboard::Clipboard::new().unwrap();
    clipboard.set_text(string).unwrap();
    println!("copied Text is: \"{:?}\"", clipboard.get_text().unwrap());
}

#[cfg(target_family = "wasm")]
fn copy_to_clipboard(string: String) {}
