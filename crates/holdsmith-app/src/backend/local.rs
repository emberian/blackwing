//! Local backend - runs Controller directly in WASM.

use holdsmith_controller::{AppStateSnapshot, Command, Controller};
use leptos::prelude::*;

use super::Backend;

/// Backend that runs the Controller directly in WASM.
pub struct LocalBackend {
    controller: RwSignal<Controller>,
    state: RwSignal<AppStateSnapshot>,
}

impl LocalBackend {
    /// Create a new local backend with an in-memory filesystem.
    pub fn new() -> Self {
        let controller = Controller::new();
        let initial_state = controller.snapshot();

        Self {
            controller: RwSignal::new(controller),
            state: RwSignal::new(initial_state),
        }
    }

    /// Sync state signal from controller.
    fn sync_state(&self) {
        self.controller.with(|ctrl| {
            self.state.set(ctrl.snapshot());
        });
    }
}

impl Default for LocalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for LocalBackend {
    fn dispatch(&self, cmd: Command) {
        self.controller.update(|ctrl| {
            ctrl.dispatch(cmd);
        });
        self.sync_state();
    }

    fn state(&self) -> ReadSignal<AppStateSnapshot> {
        self.state.read_only()
    }

    fn export_zip(&self) -> Option<Vec<u8>> {
        self.controller.with(|ctrl| ctrl.export_zip().ok())
    }

    fn import_zip(&self, data: &[u8]) -> Result<(), String> {
        let result = self.controller.try_update(|ctrl| {
            ctrl.import_zip(data).map_err(|e| e.to_string())
        });
        self.sync_state();
        result.unwrap_or(Err("Failed to update controller".to_string()))
    }
}
