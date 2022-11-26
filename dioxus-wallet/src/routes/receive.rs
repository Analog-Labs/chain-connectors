#![allow(dead_code, non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::use_router;

use crate::{
    components::globals::{Button, Header},
    qrcode::Qrcode,
};

pub fn ReceiveComponent(cx: Scope) -> Element {
    let account_address = use_state(&cx, || {
        "0x853Be3012eCeb1fC9Db70ef0Dc85Ccf3b63994BE".to_string()
    });
    let router = use_router(&cx);
    cx.render(rsx! {
            div{
            class:"main-container",
            div{
                class:"header-container",
            Header{
                title:"Receive",
                onbackclick: move |_|  router.push_route("/", None, None),
            }
        }
        input{
            class:"input",
            placeholder:"Amount"
        }
        div{
            class:"qr-code-container",
            Qrcode {
                data: b"wallet://btc?address=bcrt1q38k5zlaxfumy7gqj200xsqdmyhnlj7wg8wqx9s"
            }
        }
        div{
            "{account_address}"
        }
        div{
            class:"receive-bottom-button-container",
            Button{
                onclick:move |_|{

                },
                title:"COPY"
            }
        }
    }
        })
}
