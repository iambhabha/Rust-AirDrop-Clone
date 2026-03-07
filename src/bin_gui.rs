use dioxus::prelude::*;

fn app() -> Element {
    rsx! {
        div { "Hello World" }
    }
}

fn main() {
    dioxus::launch(app);
}
