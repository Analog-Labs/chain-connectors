#![allow(dead_code, non_snake_case)]

use dioxus::{
    core::UiEvent,
    events::{FormData, MouseEvent},
    prelude::*,
};

#[derive(Props)]
pub struct MultiSelectListingRowProps<'a> {
    assetName: &'a str,
    assetIconUri: &'a str,
    isSelected: bool,
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
pub struct SingleSelectListingRowProps<'a> {
    assetName: &'a str,
    onSelect: EventHandler<'a, MouseEvent>,
    assetIconUri: &'a str,
    nativePrice: String
}

pub fn SingleSelectListingRow<'a>(cx: Scope<'a, SingleSelectListingRowProps<'a>>) -> Element {
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

#[derive(Props)]
pub struct DashListingRowProps<'a> {
    assetName: &'a str,
    assetSymbol: &'a str,
    marketCap: &'a str,
    fiatPrice: f64,
    nativePrice: String,
    assetIconUri: &'a str,
}

pub fn DashListingRow<'a>(cx: Scope<'a, DashListingRowProps<'a>>) -> Element {
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
                        img{
                            class:"arrow-down",
                            src:"https://img.icons8.com/ios-glyphs/30/000000/long-arrow-down.png"
                        }
                        div{
                            class:"row-title-2",
                            "{cx.props.marketCap}",
                        }
                    }
                    div{
                        class:"row-subtitle",
                        "{cx.props.assetSymbol}"
                    }
                }
            }
            div{
                class:"right-row-container",
                div{
                    class:"row-right-title-container",
                div {
                    class:"row-title",
                    "{cx.props.fiatPrice}",
                }
                div {
                    class:"row-subtitle",
                    "{cx.props.nativePrice}"
                }
            }

            }

        }

    })
}
