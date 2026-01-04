//! Modal dialog component.

use leptos::prelude::*;
use std::sync::Arc;

/// A modal dialog overlay.
#[component]
pub fn Modal(
    /// Whether the modal is open
    open: RwSignal<bool>,
    /// Callback when the modal should close
    on_close: Arc<dyn Fn() + Send + Sync>,
    /// Optional title
    #[prop(optional)]
    title: Option<String>,
    /// Modal content
    children: Children,
) -> impl IntoView {
    let on_overlay_click = {
        let on_close = on_close.clone();
        move |_: web_sys::MouseEvent| on_close()
    };

    let on_modal_click = |ev: web_sys::MouseEvent| {
        // Prevent clicks inside modal from closing it
        ev.stop_propagation();
    };

    let title_clone = title.clone();
    let children_view = children();

    view! {
        <div
            class="bw-modal-overlay"
            style:display=move || if open.get() { "flex" } else { "none" }
            on:click=on_overlay_click
        >
            <div class="bw-modal" on:click=on_modal_click>
                {title_clone.map(|t| view! {
                    <div class="bw-modal-header">{t}</div>
                })}
                {children_view}
            </div>
        </div>
    }
}

/// A simple confirm/cancel modal.
#[component]
pub fn ConfirmModal(
    /// Whether the modal is open
    open: RwSignal<bool>,
    /// Modal title
    title: String,
    /// Message/question
    message: String,
    /// Confirm button text
    #[prop(default = "Confirm".to_string())]
    confirm_text: String,
    /// Cancel button text
    #[prop(default = "Cancel".to_string())]
    cancel_text: String,
    /// Callback when confirmed
    on_confirm: Arc<dyn Fn() + Send + Sync>,
    /// Callback when cancelled/closed
    on_cancel: Arc<dyn Fn() + Send + Sync>,
) -> impl IntoView {
    let on_close = on_cancel.clone();
    let on_confirm_click = on_confirm.clone();
    let on_cancel_click = on_cancel.clone();

    view! {
        <Modal open=open on_close=on_close title=title>
            <p style="color: var(--bw-text-primary); margin-bottom: 1.5rem;">
                {message}
            </p>
            <div style="display: flex; gap: 1rem; justify-content: flex-end;">
                <button
                    class="bw-choice-btn"
                    style="padding: 0.5rem 1rem;"
                    on:click=move |_| on_cancel_click()
                >
                    {cancel_text.clone()}
                </button>
                <button
                    class="bw-choice-btn"
                    style="padding: 0.5rem 1rem; background: var(--bw-accent); border-color: var(--bw-accent);"
                    on:click=move |_| on_confirm_click()
                >
                    {confirm_text.clone()}
                </button>
            </div>
        </Modal>
    }
}
