//! Card container component.

use leptos::prelude::*;

/// A card container for content sections.
#[component]
pub fn Card(
    /// Optional header text
    #[prop(optional, into)]
    header: Option<String>,
    /// Card content
    children: Children,
) -> impl IntoView {
    view! {
        <div class="bw-card">
            {header.map(|h| view! {
                <div class="bw-card-header">{h}</div>
            })}
            <div class="bw-card-body">
                {children()}
            </div>
        </div>
    }
}
