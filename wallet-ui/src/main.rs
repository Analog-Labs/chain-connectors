use dioxus::prelude::*;

fn main() {
    dioxus::desktop::launch_cfg(APP, |c| {
        c.with_window(|w| {
            w.with_resizable(true)
                .with_inner_size(dioxus::desktop::wry::application::dpi::LogicalSize::new(
                    400.0, 800.0,
                ))
                .with_title("Wallet")
        })
    });
}

static APP: Component<()> = |cx| {
    cx.render(rsx! {
                style { [include_str!("./style.css")] }

                div{class:"main-container"}




    })
};
