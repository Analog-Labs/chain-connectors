use dioxus::{events::MouseEvent, prelude::*};

#[derive(Props)]
pub struct ButtonProps<'a> {
    title: &'a str,
    onclick: EventHandler<'a, MouseEvent>,
}

#[allow(non_snake_case)]
pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element {
    cx.render(rsx! {
        button{
            onclick:|evt| {cx.props.onclick.call(evt)},
            class:"button",
            "{cx.props.title}"
        }
    })
}

#[derive(Props)]
pub struct LinkButtonProps<'a> {
    onclick: EventHandler<'a, MouseEvent>,
    uri: &'a str,
    #[props(optional)]
    title: Option<String>,
    background_color: Option<&'a str>,
}

#[allow(non_snake_case)]
pub fn LinkButton<'a>(cx: Scope<'a, LinkButtonProps<'a>>) -> Element {
    let renderTitle = match cx.props.title.clone() {
        Some(x) => rsx!(div { class: "button-title", "{x}" }),
        None => rsx!(""),
    };
    let background_color = cx.props.background_color.unwrap_or("");
    cx.render(rsx! {
        div {
            class: "link-button",
            onclick: |evt| { cx.props.onclick.call(evt) },
            div {
                class: "button-icon-container",
                background_color: background_color,
                img {
                    class: "button-icon",
                    src: cx.props.uri,
                }
            },
            renderTitle
        }
    })
}
