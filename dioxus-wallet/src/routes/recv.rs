use crate::state::use_chain_from_route;
use crate::{components::common::Header, qrcode::Qrcode};
use dioxus::prelude::*;
use dioxus_router::use_router;

#[allow(non_snake_case)]
#[inline_props]
pub fn Recv(cx: Scope) -> Element {
    let chain = use_chain_from_route(&cx);
    let info = chain.info();
    let state = chain.use_state(&cx).read();
    let router = use_router(&cx);
    cx.render(rsx! {
        div {
            class: "main-container",
            Header {
                title: "receive",
                onbackclick: move |_| router.navigate_to(&format!("/txns/{}/{}", info.chain.blockchain, info.chain.network)),
            }
            div {
                class: "title",
                "QR-CODE"
            }
            div {
                class: "label",
                "i.e scan and share to receive"
            }
            div {
                class: "qr-code-container",
                Qrcode {
                    data: state.account.as_bytes().to_vec(),
                },
            }
            div {
                class: "title",
                style: "height: 25px",
                "Account address:"
            }
            div {
                class:"label",
                style:"font-size: 13px;",
                "{state.account}",
            }
            div {
                class: "label",
                style: "font-size: 13px;",
                "i.e Receive on {info.config.blockchain} network.
                 Otherwise it may cause lost of funds."
            }
        }
    })
}
