#![allow(dead_code, non_snake_case)]

use dioxus::{events::MouseEvent, prelude::*};

//----------- Buttons ------------//

#[derive(Props)]
pub struct ButtonProps<'a> {
    title: &'a str,
    onclick: EventHandler<'a, MouseEvent>,
}
pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element {
    cx.render(rsx! {
        style { [include_str!("../styles/button.css")] }
        button{
            onclick: move |evt| cx.props.onclick.call(evt),
            class:"button",
            "{cx.props.title}"
        }
    })
}

#[derive(Props)]
pub struct LinkButtonProps<'a> {
    onClick: EventHandler<'a, MouseEvent>,
    uri: &'a str,
    #[props(optional)]
    title: Option<String>,
    backgroundColor: Option<&'a str>,
}

pub fn LinkButton<'a>(cx: Scope<'a, LinkButtonProps<'a>>) -> Element {
    let renderTitle = match cx.props.title.clone() {
        Some(x) => rsx!(div{class:"button-title", "{x}"}),
        None => rsx!(""),
    };
    let background_color = cx.props.backgroundColor.unwrap_or("");
    cx.render(rsx! {
            div {
                class:"link-button",
                    onclick: move |evt|  cx.props.onClick.call(evt),
                    div{
                        class:"button-icon-container",
                        background_color:background_color,
                        img{
                            class:"button-icon",
                            src:cx.props.uri,
                            }
                        },
                        renderTitle
                        }
    })
}

//-----------Header----------//

#[derive(Props)]
pub struct HeaderProps<'a> {
    title: &'a str,
    onbackclick: EventHandler<'a, MouseEvent>,
}

pub fn Header<'a>(cx: Scope<'a, HeaderProps<'a>>) -> Element {
    let title = cx.props.title.to_uppercase();
    cx.render(rsx! {div{
        class:"header",
        div{
            class:"back-button-container",
            onclick: move |evt| cx.props.onbackclick.call(evt),
            img{
                class:"back-icon",
                src:"https://img.icons8.com/ios-filled/50/000000/left.png"
            }
            div{

                "Back"
            }
        }
        div{
            class:"header-title",
            "{title}",

        }

        div{
            class:"header-right-container"
        }
    }})
}

#[allow(non_snake_case, unused)]
pub fn Loader(cx: Scope) -> Element {
    cx.render(rsx! {
        div{
            class:"loader"
        }
    })
}
