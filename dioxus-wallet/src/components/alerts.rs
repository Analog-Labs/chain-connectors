use dioxus::prelude::*;
use fermi::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alert {
    class: &'static str,
    msg: String,
}

impl Alert {
    fn new(class: &'static str, msg: String) -> Self {
        Self { class, msg }
    }
    pub fn info(msg: String) -> Self {
        Self::new("success", msg)
    }
    pub fn error(msg: String) -> Self {
        Self::new("danger", msg)
    }
    pub fn warning(msg: String) -> Self {
        Self::new("warning", msg)
    }
}

pub static ALERTS: AtomRef<Vec<Alert>> = |_| vec![];

#[allow(non_snake_case)]
#[inline_props]
pub fn Alerts(cx: Scope) -> Element {
    let alerts = use_atom_ref(cx, ALERTS);
    cx.render(rsx! {
        div {
           class:"alert-container",
            alerts.read().iter().enumerate().map(|(i, alert)| rsx! {
                div {
                    class: "alert alert-{alert.class} alert-dismissible",
                    role: "alert",
                    div {
                        class:"alert-message",
                        "{alert.msg}"
                    }
                    button {
                        r#type: "button",
                        class: "close",
                        aria_label: "Close",
                        onclick: move |_| {
                            alerts.write().remove(i);
                        },
                        span {
                            aria_hidden: true,
                            "\u{00d7}"
                        }
                    }
                }
            })
        }
    })
}
