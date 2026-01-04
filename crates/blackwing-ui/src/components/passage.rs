//! Passage view component for displaying narrative text.

use leptos::prelude::*;

/// Displays narrative passage text.
#[component]
pub fn PassageView(
    /// The text content to display
    text: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="bw-passage">
            {move || {
                text.get()
                    .split("\n\n")
                    .map(|para| {
                        view! { <p>{para.to_string()}</p> }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}

/// A simpler passage view for static text.
#[component]
pub fn StaticPassage(
    /// The text to display
    text: String,
) -> impl IntoView {
    view! {
        <div class="bw-passage">
            {text.split("\n\n").map(|para| {
                view! { <p>{para.to_string()}</p> }
            }).collect::<Vec<_>>()}
        </div>
    }
}
