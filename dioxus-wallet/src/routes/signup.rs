use crate::components::alerts::{Alert, ALERTS};
use crate::components::button::Button;
use crate::components::common::Header;
use crate::helpers::{copy_to_clipboard, salted_hash, save_hash};
use dioxus::prelude::*;
use fermi::use_atom_ref;
use rosetta_client::create_keys;
use rosetta_client::crypto::bip39::Mnemonic;
use rosetta_client::generate_mnemonic;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Signup {
    Intro,
    Create,
    Recover,
    Password,
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Signup(cx: Scope) -> Element {
    let passphrase = use_state(&cx, || "".to_string());
    let step = use_state(&cx, || Signup::Intro);
    let is_recovery = use_state(&cx, || false);
    let component = match step.get() {
        Signup::Intro => rsx!(Intro {
            step_state: step.clone(),
            is_recovery: is_recovery.clone(),
        }),
        Signup::Password => rsx!(Password {
            step_state: step.clone()
            passphrase: passphrase.clone(),
            is_recovery:is_recovery.clone()
        }),
        Signup::Create => rsx!(Create {
            step_state: step.clone(),
            passphrase: passphrase.get().clone(),
        }),
        Signup::Recover => rsx!(Recover {
            step_state: step.clone(),
            passphrase: passphrase.clone(),
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
pub fn Intro(cx: Scope, step_state: UseState<Signup>, is_recovery: UseState<bool>) -> Element {
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
                    onclick: move |_|
                    {
                    is_recovery.set(false);
                    step_state.set(Signup::Password);
                    }
                }
                Button {
                    title: "ALREADY HAVE ONE (RECOVER)",
                    onclick: move |_| {
                        is_recovery.set(true);
                        step_state.set(Signup::Password)}
                }
            }
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Password(
    cx: Scope,
    passphrase: UseState<String>,
    step_state: UseState<Signup>,
    is_recovery: UseState<bool>,
) -> Element {
    let password = use_state(&cx, || "".to_string());
    let isValid = use_state(&cx, || false);
    cx.render(rsx! {
        div {
            class:"main-container",
            Header {
                title:"CREATE PASSWORD",
                onbackclick: move |_| {
                    step_state.set(Signup::Intro)
                }
            }
            h3 { "Create Password" }

            input {
                class: "input",
                id: "amount",
                r#type: "password",
                placeholder:"password",
                value: "{password}",
                autofocus: true,
                oninput: move |e| {
                        password.set(e.value.clone());
                }
            },
            input {
                class: "input",
                id: "amount",
                placeholder:"confirm password",
                autofocus: true,
                r#type: "password",
                oninput: move |e| {
                   let c_password = e.value.clone();
                   let password = password.clone();
                    let is_matched = password.eq(&c_password);
                    isValid.set(is_matched);
                }
            },

            isValid.then(|| rsx! {
                Button {
                    title: "CREATE PASSWORD",
                    onclick: move |_| {
                        let password = password.get().clone();
                        passphrase.set(salted_hash(password).unwrap());
                        match is_recovery.get() {
                            false => {step_state.set(Signup::Create)},
                            true => {step_state.set(Signup::Recover)}

                        }

                }
            }
            })
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Create(cx: Scope, step_state: UseState<Signup>, passphrase: String) -> Element {
    let mnemonic = cx.use_hook(create_mnemonic);
    let mnemonic_string = mnemonic.to_string();
    let alerts = use_atom_ref(&cx, ALERTS);
    cx.render(rsx! {
        div {
            class:"content-container",
            Header {
                title:"CREATE MNEMONIC",
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
                    let passphrase = passphrase.clone();
                    match save_hash(passphrase){
                        Ok(()) => {
                            match create_keys(mnemonic) {
                                Ok(_) => cx.needs_update_any(ScopeId(0)),
                                Err(e) => {
                                    let alert = Alert::error(e.to_string());
                                    alerts.write().push(alert);
                                }
                            };
                        }
                        Err(e) => {
                            let alert = Alert::error(e.to_string());
                            alerts.write().push(alert);
                        }
                    }
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
pub fn Recover(cx: Scope, step_state: UseState<Signup>, passphrase: UseState<String>) -> Element {
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
                            let passphrase = passphrase.get().clone();
                            match save_hash(passphrase) {
                                Ok(()) => {
                                    match create_keys(mnemonic) {
                                        Ok(_) => {cx.needs_update_any(ScopeId(0))},
                                        Err(error) => {
                                        let alert = Alert::error(error.to_string());
                                        alerts.write().push(alert);
                                        }
                                       }
                                }
                                Err(e) => {
                                    let alert = Alert::error(e.to_string());
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

pub fn create_mnemonic() -> Mnemonic {
    generate_mnemonic().unwrap()
}
