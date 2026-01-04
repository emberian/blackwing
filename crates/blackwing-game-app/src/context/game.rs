//! Game context for managing game state.

use leptos::prelude::*;
use std::sync::{Arc, RwLock};

/// Current game screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Narrative,
    Inventory,
    Navigation,
    Chronicle,
    Settings,
    GameOver,
}

/// Snapshot of game state for UI rendering.
#[derive(Clone, Debug, Default)]
pub struct GameStateSnapshot {
    pub cycle: u64,
    pub location: Option<String>,
    pub credits: i64,
    pub fuel: i64,
    pub fuel_max: i64,
    pub hull: i64,
    pub hull_max: i64,
    pub passage_text: String,
    pub choices: Vec<ChoiceInfo>,
    pub in_scene: bool,
}

/// Information about an available choice.
#[derive(Clone, Debug)]
pub struct ChoiceInfo {
    pub index: usize,
    pub text: String,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
}

/// Game context providing access to game state and actions.
#[derive(Clone)]
pub struct GameContext {
    /// Current screen
    screen: RwSignal<Screen>,
    /// Game state snapshot (updated after each action)
    state: RwSignal<GameStateSnapshot>,
    /// Whether a game is loaded
    loaded: RwSignal<bool>,
    /// The underlying game runner (thread-safe)
    runner: Arc<RwLock<Option<blackwing_wasm::GameRunner>>>,
}

impl GameContext {
    /// Create a new game context.
    pub fn new() -> Self {
        Self {
            screen: RwSignal::new(Screen::default()),
            state: RwSignal::new(GameStateSnapshot::default()),
            loaded: RwSignal::new(false),
            runner: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the current screen signal.
    pub fn screen(&self) -> RwSignal<Screen> {
        self.screen
    }

    /// Get the game state signal.
    pub fn state(&self) -> RwSignal<GameStateSnapshot> {
        self.state
    }

    /// Set the current screen.
    pub fn set_screen(&self, screen: Screen) {
        self.screen.set(screen);
    }

    /// Check if a game is loaded.
    pub fn is_loaded(&self) -> bool {
        self.loaded.get()
    }

    /// Start a new game with the given bundle JSON.
    pub fn new_game(&self, bundle_json: &str, seed: u64) -> Result<(), String> {
        let runner = blackwing_wasm::GameRunner::new(bundle_json, seed)
            .map_err(|e| format!("{:?}", e))?;

        {
            let mut guard = self.runner.write().unwrap();
            *guard = Some(runner);
        }

        self.loaded.set(true);
        self.refresh_state();
        self.screen.set(Screen::Narrative);

        Ok(())
    }

    /// Make a choice in the current scene.
    pub fn make_choice(&self, index: usize) -> Result<(), String> {
        {
            let mut guard = self.runner.write().unwrap();
            let runner = guard.as_mut().ok_or("No game loaded")?;

            let command = serde_json::json!({
                "kind": "choose",
                "args": { "index": index }
            });

            runner
                .dispatch(&command.to_string())
                .map_err(|e| format!("{:?}", e))?;
        }

        self.refresh_state();
        Ok(())
    }

    /// Travel to a destination.
    pub fn travel(&self, destination: &str) -> Result<(), String> {
        {
            let mut guard = self.runner.write().unwrap();
            let runner = guard.as_mut().ok_or("No game loaded")?;

            let command = serde_json::json!({
                "kind": "go",
                "args": { "destination": destination }
            });

            runner
                .dispatch(&command.to_string())
                .map_err(|e| format!("{:?}", e))?;
        }

        self.refresh_state();
        self.screen.set(Screen::Narrative);
        Ok(())
    }

    /// Refresh the state snapshot from the game runner.
    fn refresh_state(&self) {
        let guard = self.runner.read().unwrap();
        if let Some(runner) = guard.as_ref() {
            let state_json = runner.get_state();
            if let Ok(state) = serde_json::from_str::<serde_json::Value>(&state_json) {
                let snapshot = GameStateSnapshot {
                    cycle: state["tick"].as_u64().unwrap_or(0),
                    location: state["location"].as_str().map(String::from),
                    credits: state["resources"]["credits"].as_i64().unwrap_or(0),
                    fuel: state["resources"]["fuel"].as_i64().unwrap_or(0),
                    fuel_max: 100,
                    hull: state["resources"]["hull"].as_i64().unwrap_or(100),
                    hull_max: 100,
                    passage_text: String::new(),
                    choices: Vec::new(),
                    in_scene: false,
                };
                self.state.set(snapshot);
            }
        }
    }
}

impl Default for GameContext {
    fn default() -> Self {
        Self::new()
    }
}
