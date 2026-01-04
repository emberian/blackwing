//! Holdsmith IDE - Leptos web application.
//!
//! This crate provides the web UI for the Holdsmith IDE, built with Leptos
//! and CodeMirror for the editor.

mod app;
pub mod backend;
mod bindings;
mod components;

pub use app::App;
pub use backend::{create_backend, is_tauri_environment, Backend, LocalBackend};

#[cfg(feature = "tauri")]
pub use backend::TauriBackend;

use wasm_bindgen::prelude::*;

/// Initialize the application (called from JavaScript).
#[wasm_bindgen(start)]
pub fn main() {
    // Set up better panic messages in debug builds
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Mount the Leptos app
    leptos::mount::mount_to_body(App);
}
