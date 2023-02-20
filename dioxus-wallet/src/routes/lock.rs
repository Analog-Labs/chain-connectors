use crate::{
    components::{
        alerts::{Alert, ALERTS},
        button::Button,
    },
    helpers::get_hash,
    worker,
};
use dioxus::prelude::*;
use dioxus_router::use_router;
use fermi::prelude::*;

#[allow(non_snake_case)]
pub fn Lock(cx: Scope) -> Element {
    let password = use_state(&cx, || "".to_string());
    let hash = get_hash();
    let alerts = use_atom_ref(&cx, ALERTS);
    let router = use_router(&cx);
    cx.render(rsx! {
        div {
            class:"main-container",
            h1{
                "Welcome back!"
            },
            input {
                class: "input",
                id: "password",
                r#type: "password",
                autofocus: true,
                oninput: move |e| {
                    password.set(e.value.clone())
                }
            },
            Button {
                onclick: move |_| {
                    let hash = hash.clone();
                    let password = password.get().clone();
                    match verify_hash(hash,password) {
                        true => {
                            worker::use_chain_workers(&cx).unwrap();
                            router.push_route("token", None, None)

                        }
                        false => {
                            let alert = Alert::error("Wrong Password".to_string());
                            alerts.write().push(alert);
                        }
                    }

                },
                title:"Next",
            }
        }
    })
}

pub fn verify_hash(hash: String, password: String) -> bool {
    argon2::verify_encoded(&hash, password.as_bytes()).unwrap()
}
