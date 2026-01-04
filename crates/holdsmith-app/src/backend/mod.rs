//! Backend abstraction for dual-mode operation (Tauri IPC vs pure WASM).

mod local;

#[cfg(feature = "tauri")]
mod tauri;

pub use local::LocalBackend;

#[cfg(feature = "tauri")]
pub use self::tauri::TauriBackend;

use holdsmith_controller::{AppStateSnapshot, Command};
use leptos::prelude::*;

/// Backend trait for abstracting over local (WASM) and Tauri (IPC) modes.
/// Send + Sync required for Leptos reactivity system.
pub trait Backend: Send + Sync + 'static {
    /// Dispatch a command to the controller.
    fn dispatch(&self, cmd: Command);

    /// Get a reactive signal of the current state.
    fn state(&self) -> ReadSignal<AppStateSnapshot>;

    /// Export the project as a zip file.
    fn export_zip(&self) -> Option<Vec<u8>>;

    /// Import a project from a zip file.
    fn import_zip(&self, data: &[u8]) -> Result<(), String>;
}

/// Check if running inside a Tauri webview.
pub fn is_tauri_environment() -> bool {
    #[cfg(feature = "tauri")]
    {
        use wasm_bindgen::JsValue;
        web_sys::window()
            .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__TAURI__")).ok())
            .map(|v| !v.is_undefined())
            .unwrap_or(false)
    }
    #[cfg(not(feature = "tauri"))]
    {
        false
    }
}

/// Create the appropriate backend based on the environment.
pub fn create_backend() -> Box<dyn Backend> {
    #[cfg(feature = "tauri")]
    if is_tauri_environment() {
        web_sys::console::log_1(&"[holdsmith] Using TauriBackend".into());
        return Box::new(TauriBackend::new());
    }
    web_sys::console::log_1(&"[holdsmith] Using LocalBackend".into());
    Box::new(LocalBackend::new())
}
