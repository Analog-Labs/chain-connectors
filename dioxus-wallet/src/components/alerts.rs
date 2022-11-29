use dioxus::prelude::*;
use fermi::*;

pub static ALERTS: AtomRef<Vec<(&'static str, String)>> = |_| vec![];

#[allow(non_snake_case)]
#[inline_props]
pub fn Alerts(cx: Scope) -> Element {
    let alerts = use_atom_ref(&cx, ALERTS);
    cx.render(rsx! {
        div {
            alerts.read().iter().enumerate().map(|(i, (ty, msg))| rsx! {
                div {
                    class: "alert alert-{ty} alert-dismissible",
                    role: "alert",
                    div { "{msg}" }
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
