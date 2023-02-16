use crate::{
    components::{alerts::Alert, button::Button},
    helpers::get_hash,
    worker,
};
use dioxus::prelude::*;
use fermi::prelude::*;

use super::alerts::ALERTS;

#[allow(non_snake_case)]
pub static LOCK: Atom<bool> = |_| true;

#[allow(non_snake_case)]
pub fn LockModal(cx: Scope) -> Element {
    let locked = use_atom_state(&cx, LOCK);
    let password = use_state(&cx, || "".to_string());
    let hash = get_hash();
    let alerts = use_atom_ref(&cx, ALERTS);
    match locked.get() {
        true => cx.render(rsx! {
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
                                locked.set(false);
                                cx.needs_update_any(ScopeId(0))
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
        }),
        false => None,
    }
}

pub fn verify_hash(hash: String, password: String) -> bool {
    argon2::verify_encoded(&hash, password.as_bytes()).unwrap()
}
