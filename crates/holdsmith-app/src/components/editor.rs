//! Editor panel with CodeMirror integration.

use holdsmith_controller::SeverityViewModel;
use leptos::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::app::AppContext;

/// Editor panel wrapping CodeMirror.
#[component]
pub fn EditorPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    // Get the current file and content
    let ctx1 = ctx.clone();
    let active_file = move || {
        ctx1.state().with(|s| s.project.active_file.clone())
    };

    let ctx3 = ctx.clone();
    let diagnostics = move || {
        ctx3.state().with(|s| s.editor.diagnostics.clone())
    };

    // Clone for is_dirty - used in nested closure
    let ctx4 = ctx.clone();

    // Reference to the CodeMirror container
    let editor_ref = NodeRef::<leptos::html::Div>::new();

    // Track if we've initialized (use Arc<AtomicBool> for thread-safety)
    let initialized = Arc::new(AtomicBool::new(false));
    let ctx_init = ctx.clone();

    Effect::new({
        let initialized = initialized.clone();
        let ctx_init = ctx_init.clone();
        move |_| {
            if !initialized.load(Ordering::Relaxed) {
                if let Some(el) = editor_ref.get() {
                    let ctx_for_change = ctx_init.clone();
                    let on_change = move |new_content: String| {
                        ctx_for_change.dispatch(holdsmith_controller::Command::SetContent { content: new_content });
                    };
                    let c = ctx_init.state().with(|s| s.editor.content.clone());
                    crate::bindings::init_codemirror(&el, &c, on_change);
                    initialized.store(true, Ordering::Relaxed);
                }
            }
        }
    });

    view! {
        <div class="editor-panel">
            <div class="editor-header">
                {move || {
                    let ctx_dirty = ctx4.clone();
                    match active_file() {
                        Some(path) => view! {
                            <span class="filename">
                                {path.clone()}
                                {move || {
                                    let is_dirty = ctx_dirty.state().with(|s| {
                                        if let Some(ref p) = s.project.active_file {
                                            s.project.dirty_files.contains(p)
                                        } else {
                                            false
                                        }
                                    });
                                    if is_dirty { " *" } else { "" }
                                }}
                            </span>
                        }.into_any(),
                        None => view! {
                            <span class="filename placeholder">"No file open"</span>
                        }.into_any(),
                    }
                }}
            </div>

            <div class="editor-container" node_ref=editor_ref>
                // CodeMirror will be mounted here
            </div>

            <div class="diagnostics-gutter">
                {move || {
                    let diags = diagnostics();
                    if diags.is_empty() {
                        view! { <span class="no-diagnostics">"No issues"</span> }.into_any()
                    } else {
                        let error_count = diags.iter()
                            .filter(|d| matches!(d.severity, SeverityViewModel::Error))
                            .count();
                        let warning_count = diags.iter()
                            .filter(|d| matches!(d.severity, SeverityViewModel::Warning))
                            .count();

                        view! {
                            <span class="diagnostic-summary">
                                {if error_count > 0 {
                                    format!("{} error{}", error_count, if error_count == 1 { "" } else { "s" })
                                } else {
                                    String::new()
                                }}
                                {if error_count > 0 && warning_count > 0 { ", " } else { "" }}
                                {if warning_count > 0 {
                                    format!("{} warning{}", warning_count, if warning_count == 1 { "" } else { "s" })
                                } else {
                                    String::new()
                                }}
                            </span>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
