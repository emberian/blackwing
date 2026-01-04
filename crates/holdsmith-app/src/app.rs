//! Root application component.

use std::sync::Arc;

use holdsmith_controller::{AppStateSnapshot, Command};
use leptos::prelude::*;

use crate::backend::{create_backend, Backend};
use crate::components::{
    AnalyzerPanel, BrowserPanel, DebuggerPanel, EditorPanel, PlayerPanel, StatusBar, Toolbar,
};

/// The root application state, wrapping the backend.
#[derive(Clone)]
pub struct AppContext {
    backend: Arc<dyn Backend>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            backend: Arc::from(create_backend()),
        }
    }

    /// Dispatch a command to the backend.
    pub fn dispatch(&self, cmd: Command) {
        self.backend.dispatch(cmd);
    }

    /// Get the reactive state signal.
    pub fn state(&self) -> ReadSignal<AppStateSnapshot> {
        self.backend.state()
    }

    /// Export the project as a zip file.
    pub fn export_zip(&self) -> Option<Vec<u8>> {
        self.backend.export_zip()
    }

    /// Import a project from a zip file.
    pub fn import_zip(&self, data: &[u8]) -> Result<(), String> {
        self.backend.import_zip(data)
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Which panel is currently active in the right sidebar.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum RightPanel {
    #[default]
    Analyzer,
    Debugger,
    Player,
}

/// Root application component.
#[component]
pub fn App() -> impl IntoView {
    // Create the global app context
    let ctx = AppContext::new();
    provide_context(ctx.clone());

    // Track which right panel is active
    let right_panel = RwSignal::new(RightPanel::Analyzer);

    view! {
        <div class="app">
            <Toolbar />

            <div class="main-content">
                // Left sidebar: File browser
                <aside class="sidebar left">
                    <BrowserPanel />
                </aside>

                // Center: Editor
                <main class="editor-area">
                    <EditorPanel />
                </main>

                // Right sidebar: Analyzer/Debugger/Player
                <aside class="sidebar right">
                    <div class="panel-tabs">
                        <button
                            class:active=move || right_panel.get() == RightPanel::Analyzer
                            on:click=move |_| right_panel.set(RightPanel::Analyzer)
                        >
                            "Analyzer"
                        </button>
                        <button
                            class:active=move || right_panel.get() == RightPanel::Debugger
                            on:click=move |_| right_panel.set(RightPanel::Debugger)
                        >
                            "Debugger"
                        </button>
                        <button
                            class:active=move || right_panel.get() == RightPanel::Player
                            on:click=move |_| right_panel.set(RightPanel::Player)
                        >
                            "Player"
                        </button>
                    </div>

                    <div class="panel-content">
                        {move || match right_panel.get() {
                            RightPanel::Analyzer => view! { <AnalyzerPanel /> }.into_any(),
                            RightPanel::Debugger => view! { <DebuggerPanel /> }.into_any(),
                            RightPanel::Player => view! { <PlayerPanel /> }.into_any(),
                        }}
                    </div>
                </aside>
            </div>

            <StatusBar />
        </div>
    }
}
