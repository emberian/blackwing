//! Status bar component.

use leptos::prelude::*;

use crate::app::AppContext;

/// Bottom status bar showing current state.
#[component]
pub fn StatusBar() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    let ctx1 = ctx.clone();
    let active_file = move || {
        ctx1.state().with(|s| s.project.active_file.clone())
    };

    let ctx2 = ctx.clone();
    let cursor_pos = move || ctx2.state().with(|s| s.editor.cursor);

    let ctx3 = ctx.clone();
    let analyzing = move || ctx3.state().with(|s| s.analyzer.analyzing);

    let ctx4 = ctx.clone();
    let dirty_count = move || {
        ctx4.state().with(|s| s.project.dirty_files.len())
    };

    let ctx5 = ctx.clone();
    let player_active = move || ctx5.state().with(|s| s.player.active);

    let ctx6 = ctx.clone();
    let debugger_active = move || ctx6.state().with(|s| s.debugger.active);

    view! {
        <footer class="status-bar">
            <div class="status-left">
                {move || {
                    match active_file() {
                        Some(path) => view! {
                            <span class="status-item">{path}</span>
                        }.into_any(),
                        None => view! {
                            <span class="status-item muted">"No file"</span>
                        }.into_any(),
                    }
                }}

                {move || {
                    let (line, col) = cursor_pos();
                    view! {
                        <span class="status-item">
                            "Ln " {line + 1} ", Col " {col + 1}
                        </span>
                    }
                }}
            </div>

            <div class="status-center">
                {move || if analyzing() {
                    view! { <span class="status-item analyzing">"Analyzing..."</span> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </div>

            <div class="status-right">
                {move || {
                    let count = dirty_count();
                    if count > 0 {
                        view! {
                            <span class="status-item dirty">
                                {count} " unsaved"
                            </span>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}

                {move || if player_active() {
                    view! { <span class="status-item player-active">"Playing"</span> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}

                {move || if debugger_active() {
                    view! { <span class="status-item debugger-active">"Debugging"</span> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}

                <span class="status-item version">"Holdsmith v0.1"</span>
            </div>
        </footer>
    }
}
