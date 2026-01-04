//! Tauri backend - communicates with native backend via IPC.

use holdsmith_controller::{AppStateSnapshot, Command, CommandResult};
use leptos::prelude::*;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::Backend;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// Backend that communicates with the native Tauri backend via IPC.
pub struct TauriBackend {
    state: RwSignal<AppStateSnapshot>,
}

impl TauriBackend {
    /// Create a new Tauri backend.
    pub fn new() -> Self {
        let state = RwSignal::new(AppStateSnapshot::default());

        // Fetch initial state
        let state_clone = state;
        spawn_local(async move {
            if let Ok(snapshot) = Self::fetch_snapshot().await {
                state_clone.set(snapshot);
            }
        });

        Self { state }
    }

    /// Fetch the current state snapshot from the native backend.
    async fn fetch_snapshot() -> Result<AppStateSnapshot, String> {
        let args = to_value(&serde_json::json!({})).map_err(|e| e.to_string())?;
        let result = tauri_invoke("get_snapshot", args).await;
        from_value(result).map_err(|e| e.to_string())
    }

    /// Dispatch a command to the native backend.
    async fn dispatch_command(cmd: &Command) -> Result<CommandResult, String> {
        let args = to_value(&serde_json::json!({ "cmd": cmd })).map_err(|e| e.to_string())?;
        let result = tauri_invoke("dispatch_command", args).await;
        from_value(result).map_err(|e| e.to_string())
    }

    /// Refresh state after a command.
    #[allow(dead_code)]
    fn refresh_state(&self) {
        let state = self.state;
        spawn_local(async move {
            if let Ok(snapshot) = Self::fetch_snapshot().await {
                state.set(snapshot);
            }
        });
    }
}

impl Default for TauriBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for TauriBackend {
    fn dispatch(&self, cmd: Command) {
        let state = self.state;
        spawn_local(async move {
            let _ = Self::dispatch_command(&cmd).await;
            // Refresh state after command
            if let Ok(snapshot) = Self::fetch_snapshot().await {
                state.set(snapshot);
            }
        });
    }

    fn state(&self) -> ReadSignal<AppStateSnapshot> {
        self.state.read_only()
    }

    fn export_zip(&self) -> Option<Vec<u8>> {
        // For Tauri, this would need to be async. For now, return None
        // and handle export via a separate async method or Tauri command.
        // TODO: Implement async export via Tauri command
        None
    }

    fn import_zip(&self, data: &[u8]) -> Result<(), String> {
        let data = data.to_vec();
        let state = self.state;
        spawn_local(async move {
            let args = to_value(&serde_json::json!({ "data": data }))
                .map_err(|e| e.to_string())
                .unwrap();
            let _ = tauri_invoke("import_zip", args).await;
            // Refresh state
            if let Ok(snapshot) = Self::fetch_snapshot().await {
                state.set(snapshot);
            }
        });
        Ok(())
    }
}
