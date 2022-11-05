use dioxus::{
    core::UiEvent,
    events::{FormData, MouseEvent},
    prelude::*,
};

#[derive(Props)]

pub struct MultiSelectListingRowProps<'a> {
    assetName: &'a str,
    nativePrice: f64,
    assetIconUri: &'a str,
    isSelected: &'a str,
    onSelect: EventHandler<'a, UiEvent<FormData>>,
}

pub fn MultiSelectListingRow<'a>(cx: Scope<'a, MultiSelectListingRowProps<'a>>) -> Element {
    cx.render(rsx! {
        div{
            class:"listing-row-container",
            div{
                class:"left-row-container",
                div{
                    class:"image-container",
                    img{
                        class:"row-image",
                        src:cx.props.assetIconUri
                    }
                }
                div{
                    class:"row-left-title-container",
                    div{
                        class:"row-title",
                        "{cx.props.assetName}",

                    }
                }
            }
            div{
                class:"right-row-container",

                label {
                    class:"switch",
                    input {
                        "type":"checkbox",
                        "checked":"{cx.props.isSelected}",
                        oninput: move |evt| cx.props.onSelect.call(evt)


                    }
                    span{
                        class:"slider round"
                    }

                }


            }



        }

    })
}

#[derive(Props)]

pub struct singleSelectListingRowProps<'a> {
    assetName: &'a str,
    onSelect: EventHandler<'a, MouseEvent>,
    assetIconUri: &'a str,
    nativePrice: f64,
}

pub fn SingleSelectListingRow<'a>(cx: Scope<'a, singleSelectListingRowProps<'a>>) -> Element {
    cx.render(rsx! {
        div{
            onclick:move |evt| cx.props.onSelect.call(evt),
            class:"listing-row-container",
            div{
                class:"left-row-container",
                div{
                    class:"image-container",
                    img{
                        class:"row-image",
                        src:cx.props.assetIconUri
                    }
                }
                div{
                    class:"row-left-title-container",
                    div{
                        class:"row-title",
                        "{cx.props.assetName}",

                    }
                }
            }
            div{
                class:"right-row-container",

                    div{
                        "{cx.props.nativePrice}"
                    }

            }



        }

    })
}
