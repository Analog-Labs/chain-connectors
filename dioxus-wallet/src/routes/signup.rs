use crate::components::alerts::{Alert, ALERTS};
use crate::components::button::Button;
use crate::components::common::Header;
use crate::helpers::copy_to_clipboard;
use dioxus::prelude::*;
use fermi::use_atom_ref;
use rosetta_client::crypto::bip39::Mnemonic;
use rosetta_client::{generate_mnemonic, MnemonicStore};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Signup {
    Intro,
    Create,
    Recover,
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Signup(cx: Scope) -> Element {
    let step = use_state(&cx, || Signup::Intro);
    let component = match step.get() {
        Signup::Intro => rsx!(Intro {
            step_state: step.clone()
        }),
        Signup::Create => rsx!(Create {
            step_state: step.clone()
        }),
        Signup::Recover => rsx!(Recover {
            step_state: step.clone()
        }),
    };
    cx.render(rsx! {
        div {
            class: "main-container",
            component
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Intro(cx: Scope, step_state: UseState<Signup>) -> Element {
    cx.render(rsx! {
        div {
            class: "content-container",
            div {
                class: "intro-content-container",
                h1{ "WELCOME" }
            }
            div {
                class: "signup-buttons-container",
                Button {
                    title: "CREATE NEW WALLET",
                    onclick: move |_| step_state.set(Signup::Create)
                }
                Button {
                    title: "ALREADY HAVE ONE (RECOVER)",
                    onclick: move |_| step_state.set(Signup::Recover)
                }
            }
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Create(cx: Scope, step_state: UseState<Signup>) -> Element {
    let mnemonic = cx.use_hook(|| generate_mnemonic().unwrap());
    let mnemonic_string = mnemonic.to_string();
    let alerts = use_atom_ref(&cx, ALERTS);
    cx.render(rsx! {
        div {
            class:"content-container",
            Header {
                title:"CREATE",
                onbackclick: move |_| {
                    step_state.set(Signup::Intro)
                }
            }
            h3 { "MNEMONIC" }
            div {
                class:"mnemonic-container",
                mnemonic.word_iter().map(|word| {
                    rsx! {
                        div {
                            class: "mnemonic-word",
                            "{word.to_uppercase()}"
                        }
                    }
                })
            }  div {
                class: "signup-buttons-container",
            Button {
                title: "CREATE WALLET",
                onclick: move |_| {
                    let mnemonic = mnemonic.clone();
                    match MnemonicStore::new(None).unwrap().set(&mnemonic) {
                        Ok(_) => cx.needs_update_any(ScopeId(0)),
                        Err(e) => {
                            let alert = Alert::error(e.to_string());
                            alerts.write().push(alert);
                        }
                    };
            }
        }
            Button {
                title: "Copy To Clipboard",
                onclick: move |_| {
                    let mnemonic_string = mnemonic_string.clone();
                    match copy_to_clipboard(mnemonic_string){
                        Ok(_) => {
                            let alert = Alert::info("Copied To Clipboard".to_string());
                            alerts.write().push(alert);
                        },
                        Err(e) => {
                            let alert = Alert::error(e.to_string());
                            alerts.write().push(alert);
                        }
                    }
            }
        }
        }
    }
        })
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Recover(cx: Scope, step_state: UseState<Signup>) -> Element {
    let mnemonic_string = use_state(&cx, || "".to_string());
    let alerts = use_atom_ref(&cx, ALERTS);
    cx.render(rsx! {
            div {
                class:"content-container",
                Header {
                    title:"RECOVER",
                    onbackclick: move |_| {
                        step_state.set(Signup::Intro)
                    }
                }
                textarea {
                    class: "multi-line-input",
                    id: "mnemonic",
                    value: "{mnemonic_string}",
                    placeholder:"Paste mnemonic here to recover",
                    autofocus: true,
                    oninput: move |e| {
                        let mnemonic_string = mnemonic_string.clone();
                        mnemonic_string.set(e.value.clone());
                    },
            }

            div {
                class: "signup-buttons-container",
            Button {
                title: "Recover",
                onclick: move |_| {
                    let mnemonic_string = mnemonic_string.clone();
                      match Mnemonic::parse_normalized(mnemonic_string.get().as_str()){
                        Ok(mnemonic) => {
                            match MnemonicStore::new(None).unwrap().set(&mnemonic) {
                                Ok(_) => {cx.needs_update_any(ScopeId(0))},
                                Err(error) => {
                                let alert = Alert::error(error.to_string());
                                alerts.write().push(alert);
                                }
                               }
                        }
                        Err(error) => {
                                let alert = Alert::error(error.to_string());
                                alerts.write().push(alert);
                        }
                      }
                }
            }
        }
    }
        })
}
