use dioxus::prelude::*;
use fermi::*;

pub static LOADER: Atom<bool> = |_| false;

#[allow(non_snake_case, unused)]
pub fn Loader(cx: Scope) -> Element {
    let is_visible = use_read(&cx, LOADER);
    if *is_visible {
        cx.render(rsx! {
            div{
                class:"loader-container",
                div {
                    class:"loader",
                    div {
                        class:"loader-spin"
                    }
                    h4{"loading..."}
                }

            }
        })
    } else {
        cx.render(rsx! {
            div { style:"display: none"}
        })
    }
}
