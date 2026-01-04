//! Top toolbar component.

use leptos::prelude::*;

use crate::app::AppContext;

/// Top toolbar with file operations and project controls.
#[component]
pub fn Toolbar() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    let ctx1 = ctx.clone();
    let new_project = move |_| {
        ctx1.dispatch(holdsmith_controller::Command::NewProject);
    };

    let ctx2 = ctx.clone();
    let save_current = move |_| {
        ctx2.dispatch(holdsmith_controller::Command::SaveCurrentFile);
    };

    let ctx3 = ctx.clone();
    let save_all = move |_| {
        ctx3.dispatch(holdsmith_controller::Command::SaveAll);
    };

    // Export project as zip
    let ctx4 = ctx.clone();
    let export_project = move |_| {
        match ctx4.export_zip() {
            Some(data) => {
                // Trigger download via JS
                crate::bindings::download_blob(&data, "project.zip", "application/zip");
            }
            None => {
                web_sys::console::error_1(&"Export failed".into());
            }
        }
    };

    let ctx5 = ctx.clone();

    view! {
        <header class="toolbar">
            <div class="toolbar-group">
                <span class="logo">"Holdsmith"</span>
            </div>

            <div class="toolbar-group">
                <button on:click=new_project title="New Project">
                    <span class="icon">"+"</span>
                    " New"
                </button>
                <button on:click=save_current title="Save Current File (Ctrl+S)">
                    <span class="icon">"S"</span>
                    " Save"
                </button>
                <button on:click=save_all title="Save All (Ctrl+Shift+S)">
                    " Save All"
                </button>
            </div>

            <div class="toolbar-group">
                <button on:click=export_project title="Export Project as ZIP">
                    " Export"
                </button>
                <label class="file-input-label" title="Import Project from ZIP">
                    " Import"
                    <input
                        type="file"
                        accept=".zip"
                        on:change=move |ev| {
                            crate::bindings::handle_file_import(ev, ctx5.clone());
                        }
                    />
                </label>
            </div>

            <div class="toolbar-spacer"></div>

            <div class="toolbar-group">
                <span class="help-text">"Scene authoring IDE"</span>
            </div>
        </header>
    }
}
