use arboard::Clipboard;
use dioxus::{
    prelude::*,
    router::{Route, Router},
};

fn main() {
    dioxus::desktop::launch_cfg(APP, |c| {
        c.with_window(|w| {
            w.with_resizable(true)
                .with_inner_size(dioxus::desktop::wry::application::dpi::LogicalSize::new(
                    400.0, 800.0,
                ))
                .with_title("Wallet")
                .with_resizable(false)
        })
    });
}

static APP: Component<()> = |cx| {
    cx.render(rsx! {
                    style { [include_str!("./style.css")] }
                    Router{
                        Route{to:"/",Dashboard{}}
                        Route{to:"/send",SendScreen{}}
                        Route{to:"/receive",ReceiveScreen{}}
              }
    })
};

//-------- Dashboard Component ------ //

pub fn Dashboard(cx: Scope) -> Element {
    let dummyAssets = ["Eth", "Bitcoin", "Dot"];

    let balance = use_state(&cx, || 2.30);
    let account_address = use_state(&cx, || {
        "0x853Be3012eCeb1fC9Db70ef0Dc85Ccf3b63994BE".to_string()
    });

    cx.render(rsx!(
        div {
                class:"main-container",
        div{
        class: "dashboard-container" ,
        h1{
            class:"screen-title",

            "Ethereum"
        }

        h5{
            "{account_address}"
        }
        h2{


            "{balance} ETH",
        }
        div {
            class:"button-container",
            Link { class:"button", to: "/send", "SEND"},
            Link { class:"button", to: "/receive", "RECEIVE"}
        }
    }   div{
                    class:"listing-container",
                    div{
                        class:"listing-title",
                        "Tokens"}
                    div {
                        class:"list",
                         dummyAssets.iter().map(|name| rsx!(
                            div {
                                onclick :move |evt| println!("clicked {:?}",evt),
                                class:"list-item",
                            div{
                                class:"asset-name",
                            "{name}",
                            }
                            div{
                                class:"asset-name",
                                "0",
                            }
                            }

                             ))
                    }
                }}))
}

//-------------------------------------//

//-------- Send Component -------- //

pub fn SendScreen(cx: Scope) -> Element {
    let sender_address = use_state(&cx, || {
        "0x853Be3012eCeb1fC9Db70ef0Dc85Ccf3b63994BE".to_string()
    });
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

// ------Receive component------ //

pub fn ReceiveScreen(cx: Scope) -> Element {
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

//------------------------------------//

// ------ Utilities --------  //

fn copy_to_clipboard(string: String) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(string).unwrap();
    println!("copied Text is: \"{:?}\"", clipboard.get_text().unwrap());
}
