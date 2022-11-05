use dioxus::{events::MouseEvent, prelude::*};
use dioxus_router::Link;

#[derive(Props)]

//----------- Buttons ------------//

pub struct ButtonProps<'a> {
    title: &'a str,
}

pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element {
    cx.render(rsx! {
        style { [include_str!("../styles/button.css")] }


        button{
            class:"button",

            "COPY"
        }
    })
}

#[derive(Props)]

pub struct linkButtonProps<'a> {
    onClick: EventHandler<'a, MouseEvent>,
    uri: &'a str,
    #[props(optional)]
    title: Option<String>,
    backgroundColor: Option<&'a str>,
}

pub fn LinkButton<'a>(cx: Scope<'a, linkButtonProps<'a>>) -> Element {
    let renderTitle = match cx.props.title.clone() {
        Some(x) => rsx!(div{class:"button-title", "{x}"}),
        None => rsx!(""),
    };
    let background_color = match cx.props.backgroundColor.clone() {
        Some(x) => x,
        None => "",
    };
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
            "{cx.props.title}"

        }
    }})
}
