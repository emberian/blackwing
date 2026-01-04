//! Blackwing game frontend.
//!
//! This is the main entry point for the Blackwing game UI,
//! built with Leptos and compiled to WASM.

pub mod app;
pub mod components;
pub mod context;
pub mod screens;

use wasm_bindgen::prelude::*;

/// Initialize the game application.
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize tracing for logging
    tracing_wasm::set_as_global_default();

    // Inject shared UI styles
    blackwing_ui::inject_styles();

    tracing::info!("Blackwing game initialized");
}

/// Mount the game application to the DOM.
#[wasm_bindgen]
pub fn mount(root_id: &str) {
    let root = web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
        .get_element_by_id(root_id)
        .expect("root element not found");

    leptos::mount::mount_to(root.unchecked_into(), app::App).forget();
}
