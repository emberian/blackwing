//! File browser panel.

use leptos::prelude::*;
use std::sync::Arc;

use crate::app::AppContext;
use holdsmith_controller::FileTreeNode;

/// File browser panel showing the project file tree.
#[component]
pub fn BrowserPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    let ctx_tree = ctx.clone();
    let file_tree = move || {
        ctx_tree.state().with(|s| s.project.file_tree.clone())
    };

    // New file dialog state
    let show_new_file = RwSignal::new(false);
    let new_file_name = RwSignal::new(String::new());

    let ctx_create = ctx.clone();
    let do_create = Arc::new(move || {
        let name = new_file_name.get();
        if !name.is_empty() {
            let path = if name.ends_with(".scene") {
                name.clone()
            } else {
                format!("{}.scene", name)
            };
            ctx_create.dispatch(holdsmith_controller::Command::NewFile { path });
            show_new_file.set(false);
            new_file_name.set(String::new());
        }
    });

    let do_create_click = do_create.clone();
    let do_create_key = do_create.clone();

    view! {
        <div class="browser-panel">
            <div class="browser-header">
                <span>"Files"</span>
                <button
                    class="new-file-btn"
                    on:click=move |_| show_new_file.set(true)
                    title="New File"
                >
                    "+"
                </button>
            </div>

            {move || if show_new_file.get() {
                let do_create_key = do_create_key.clone();
                let do_create_click = do_create_click.clone();
                view! {
                    <div class="new-file-dialog">
                        <input
                            type="text"
                            placeholder="filename.scene"
                            prop:value=move || new_file_name.get()
                            on:input=move |ev| {
                                new_file_name.set(event_target_value(&ev));
                            }
                            on:keydown={
                                let do_create = do_create_key.clone();
                                move |ev: web_sys::KeyboardEvent| {
                                    if ev.key() == "Enter" {
                                        do_create();
                                    } else if ev.key() == "Escape" {
                                        show_new_file.set(false);
                                    }
                                }
                            }
                        />
                        <div class="dialog-buttons">
                            <button on:click={
                                let do_create = do_create_click.clone();
                                move |_| do_create()
                            }>"Create"</button>
                            <button on:click=move |_| show_new_file.set(false)>"Cancel"</button>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <div class="file-tree">
                {move || {
                    let tree = file_tree();
                    if tree.is_empty() {
                        view! {
                            <div class="empty-tree">
                                <p>"No files yet"</p>
                                <p class="hint">"Click + to create a new scene file"</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <ul class="file-list">
                                {tree.into_iter().map(|node| view! {
                                    <FileTreeItem node=node />
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// A single item in the file tree.
#[component]
fn FileTreeItem(node: FileTreeNode) -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    let path = node.path.clone();
    let name = node.name.clone();
    let is_dir = node.is_dir;
    let children = node.children;

    let ctx1 = ctx.clone();
    let path1 = path.clone();
    let is_active = move || {
        ctx1.state().with(|s| {
            s.project.active_file.as_ref() == Some(&path1)
        })
    };

    let ctx2 = ctx.clone();
    let path2 = path.clone();
    let is_dirty = Arc::new(move || {
        ctx2.state().with(|s| {
            s.project.dirty_files.contains(&path2)
        })
    });

    let ctx3 = ctx.clone();
    let path3 = path.clone();
    let open_file = move |_| {
        if !is_dir {
            ctx3.dispatch(holdsmith_controller::Command::OpenFile { path: path3.clone() });
        }
    };

    let ctx4 = ctx.clone();
    let path4 = path.clone();
    let name_for_delete = name.clone();
    let delete_file = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        if !is_dir {
            if crate::bindings::confirm(&format!("Delete {}?", name_for_delete)) {
                ctx4.dispatch(holdsmith_controller::Command::DeleteFile { path: path4.clone() });
            }
        }
    };

    if is_dir {
        view! {
            <li class="tree-item directory">
                <span class="item-name">
                    <span class="icon">"/"</span>
                    {name}
                </span>
                <ul class="file-list nested">
                    {children.into_iter().map(|child| view! {
                        <FileTreeItem node=child />
                    }).collect::<Vec<_>>()}
                </ul>
            </li>
        }.into_any()
    } else {
        let is_dirty_for_class = is_dirty.clone();
        let is_dirty_for_text = is_dirty.clone();
        view! {
            <li
                class="tree-item file"
                class:active=is_active
                class:dirty=move || is_dirty_for_class()
                on:click=open_file
            >
                <span class="item-name">
                    <span class="icon">"S"</span>
                    {name}
                    {move || if is_dirty_for_text() { " *" } else { "" }}
                </span>
                <button
                    class="delete-btn"
                    on:click=delete_file
                    title="Delete file"
                >
                    "x"
                </button>
            </li>
        }.into_any()
    }
}
