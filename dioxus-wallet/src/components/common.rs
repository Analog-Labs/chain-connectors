//! header with backbutton and title properties
use dioxus::{events::MouseEvent, prelude::*};

#[derive(Props)]
pub struct HeaderProps<'a> {
    title: &'a str,
    onbackclick: EventHandler<'a, MouseEvent>,
}

#[allow(non_snake_case)]
pub fn Header<'a>(cx: Scope<'a, HeaderProps<'a>>) -> Element {
    let title = cx.props.title.to_uppercase();
    cx.render(rsx! {div {
        class: "header",
        div {
            class: "back-button-container",
            onclick: move |evt| cx.props.onbackclick.call(evt),
            img {
                class: "back-icon",
                src: img!("back.png"),
            }
            div {
                "Back"
            }
        }
        div {
            class: "header-title",
            "{title}",
        }
        div {
            class: "header-right-container"
        }
    }})
}
