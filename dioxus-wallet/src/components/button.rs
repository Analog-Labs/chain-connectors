use dioxus::prelude::*;
use dioxus_router::Link;

#[derive(Props)]

pub struct ButtonProps<'a> {
    title: &'a str,
}

pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element {
    cx.render(rsx! {

        button{
            class:"button",

            "{cx.props.title}"
        }
    })
}

#[derive(Props)]

pub struct linkButtonProps<'a> {
    to: &'a str,
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
            Link {
                class:"link-button",
                    to: cx.props.to,
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
