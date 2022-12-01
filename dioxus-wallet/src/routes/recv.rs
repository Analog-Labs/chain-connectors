use crate::qrcode::Qrcode;
use crate::state::use_chain_from_route;
use crate::worker::Action;
use dioxus::prelude::*;
use dioxus_router::Link;

#[allow(non_snake_case)]
#[inline_props]
pub fn Recv(cx: Scope) -> Element {
    let handle = use_coroutine_handle(&cx).unwrap();
    let chain = use_chain_from_route(&cx);
    let info = chain.info();
    let state = chain.use_state(&cx).read();
    handle.send(Action::SyncAccount(info.chain));
    let qrcode = format!("{};{}", info.chain, &state.account);
    cx.render(rsx! {
        div {
            Link { to: "/", "Back" },
            Qrcode {
                data: qrcode.into(),
            },
            "Recv {info.name}"
        }
    })
}
