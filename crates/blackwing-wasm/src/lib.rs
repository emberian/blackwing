//! WASM bindings for the generalized game engine.
//!
//! This crate provides a JavaScript-friendly API for running game bundles
//! in the browser via WebAssembly.
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import init, { GameRunner } from 'blackwing-wasm';
//!
//! await init();
//!
//! const bundleData = await fetch('/game.bundle').then(r => r.arrayBuffer());
//! const game = new GameRunner(new Uint8Array(bundleData), Date.now());
//!
//! // Get initial state
//! const state = JSON.parse(game.get_state());
//!
//! // Dispatch a command
//! const events = JSON.parse(game.dispatch(JSON.stringify({
//!     kind: "go",
//!     args: { direction: "north" }
//! })));
//! ```

use blackwing_bundle::{standard_systems, GameBundle};
use blackwing_core::{Command, EntityId, Value};
use engine_runtime::{GeneralizedRuntime, RuntimeEvent};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in WASM
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// A game runner that can be used from JavaScript.
#[wasm_bindgen]
pub struct GameRunner {
    runtime: GeneralizedRuntime,
    bundle: GameBundle,
    player_actor: EntityId,
}

/// Command format for JavaScript interop
#[derive(Debug, Serialize, Deserialize)]
struct JsCommand {
    kind: String,
    #[serde(default)]
    args: std::collections::HashMap<String, serde_json::Value>,
}

/// Convert JSON value to engine Value
fn json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => {
            // Check if it's an entity reference (e.g., "npc:bob")
            if s.contains(':') && !s.starts_with("http") {
                let parts: Vec<&str> = s.splitn(2, ':').collect();
                if parts.len() == 2 {
                    return Value::EntityRef(EntityId::new(parts[0], parts[1]));
                }
            }
            Value::String(s.into())
        }
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(map) => {
            let mut result = std::collections::HashMap::new();
            for (k, v) in map {
                result.insert(k, json_to_value(v));
            }
            Value::Map(result)
        }
    }
}

#[wasm_bindgen]
impl GameRunner {
    /// Create a new game runner from bundle JSON.
    ///
    /// The bundle should be serialized as JSON for now.
    /// In the future, we'll support binary bundle format.
    #[wasm_bindgen(constructor)]
    pub fn new(bundle_json: &str, seed: u64) -> Result<GameRunner, JsValue> {
        let bundle: GameBundle = serde_json::from_str(bundle_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse bundle: {}", e)))?;

        // Create runtime from bundle's initial state
        let state = bundle.create_initial_state();
        let mut runtime = GeneralizedRuntime::with_state(state, seed);

        // Set game mode from manifest
        runtime.set_mode(bundle.manifest.game_mode.clone());

        // Register standard systems
        if let Ok(systems) = standard_systems() {
            for system in systems {
                runtime.register_system(Box::new(system));
            }
        }

        // Compile and register script systems from bundle
        if let Ok(script_systems) = bundle.compile_systems() {
            for system in script_systems {
                runtime.register_system(Box::new(system));
            }
        }

        let player_actor = runtime.spawn_actor("player");

        // Set player at initial location if specified
        if let Some(loc) = &bundle.manifest.initial_location {
            if let Some(key) = runtime.state().entities.key_of(loc) {
                runtime.state_mut().entities.set_component(
                    key,
                    "position",
                    Value::EntityRef(loc.clone()),
                );
            }
        }

        Ok(GameRunner {
            runtime,
            bundle,
            player_actor,
        })
    }

    /// Create an empty game runner for testing.
    #[wasm_bindgen]
    pub fn new_empty(seed: u64) -> GameRunner {
        let bundle = GameBundle::new("Empty Game");

        // Create runtime from bundle's initial state
        let state = bundle.create_initial_state();
        let mut runtime = GeneralizedRuntime::with_state(state, seed);

        // Register standard systems
        if let Ok(systems) = standard_systems() {
            for system in systems {
                runtime.register_system(Box::new(system));
            }
        }

        let player_actor = runtime.spawn_actor("player");

        GameRunner {
            runtime,
            bundle,
            player_actor,
        }
    }

    /// Get the current game state as JSON.
    #[wasm_bindgen]
    pub fn get_state(&self) -> String {
        // For now, just return basic state info
        let state = self.runtime.state();
        let info = serde_json::json!({
            "tick": state.time.tick(),
            "resources": state.resources,
            "flags": state.flags,
            "entity_count": state.entities.len(),
        });
        serde_json::to_string(&info).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get the player actor ID.
    #[wasm_bindgen]
    pub fn get_player_id(&self) -> String {
        self.player_actor.as_qualified()
    }

    /// Dispatch a command from the player.
    ///
    /// The command should be JSON with format:
    /// ```json
    /// { "kind": "go", "args": { "direction": "north" } }
    /// ```
    ///
    /// Returns events as JSON array.
    #[wasm_bindgen]
    pub fn dispatch(&mut self, command_json: &str) -> Result<String, JsValue> {
        let js_cmd: JsCommand = serde_json::from_str(command_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse command: {}", e)))?;

        // Convert to engine Command
        let mut cmd = Command::new(&js_cmd.kind);
        for (key, value) in js_cmd.args {
            cmd = cmd.with_arg(&key, json_to_value(value));
        }

        // Dispatch
        let events = self
            .runtime
            .dispatch(self.player_actor.clone(), cmd)
            .map_err(|e| JsValue::from_str(&format!("Command failed: {}", e)))?;

        // Convert events to JSON
        let json_events: Vec<serde_json::Value> = events
            .iter()
            .map(|e| event_to_json(e))
            .collect();

        serde_json::to_string(&json_events)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize events: {}", e)))
    }

    /// Tick the game (for real-time modes).
    #[wasm_bindgen]
    pub fn tick(&mut self, delta_ms: u64) -> Result<String, JsValue> {
        let events = self
            .runtime
            .tick(delta_ms)
            .map_err(|e| JsValue::from_str(&format!("Tick failed: {}", e)))?;

        let json_events: Vec<serde_json::Value> = events
            .iter()
            .map(|e| event_to_json(e))
            .collect();

        serde_json::to_string(&json_events)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize events: {}", e)))
    }

    /// Get available commands based on current context.
    #[wasm_bindgen]
    pub fn get_available_commands(&self) -> String {
        // Get commands from all enabled systems (use iter for script systems)
        let systems = self.runtime.systems();
        let commands: Vec<String> = systems
            .enabled_systems()
            .flat_map(|s| s.handles_commands_iter())
            .map(|s| s.to_string())
            .collect();

        serde_json::to_string(&commands).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get the game mode.
    #[wasm_bindgen]
    pub fn get_mode(&self) -> String {
        match self.runtime.mode() {
            blackwing_core::GameMode::TurnBased => "turn_based".to_string(),
            blackwing_core::GameMode::RealTime { tick_rate_ms } => {
                format!("real_time:{}", tick_rate_ms)
            }
            blackwing_core::GameMode::Hybrid { tick_rate_ms, .. } => {
                format!("hybrid:{}", tick_rate_ms)
            }
        }
    }

    /// Get the event log as JSON.
    #[wasm_bindgen]
    pub fn get_event_log(&self) -> String {
        let events: Vec<serde_json::Value> = self
            .runtime
            .event_log()
            .iter()
            .map(|e| event_to_json(e))
            .collect();

        serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string())
    }

    /// Add an entity template to the bundle (for hot-reload).
    #[wasm_bindgen]
    pub fn add_template(&mut self, template_json: &str) -> Result<(), JsValue> {
        let template: blackwing_bundle::EntityTemplate = serde_json::from_str(template_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse template: {}", e)))?;

        self.bundle.templates.push(template);
        Ok(())
    }

    /// Add a dialogue tree to the bundle (for hot-reload).
    #[wasm_bindgen]
    pub fn add_dialogue(&mut self, dialogue_json: &str) -> Result<(), JsValue> {
        let dialogue: blackwing_bundle::DialogueTree = serde_json::from_str(dialogue_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse dialogue: {}", e)))?;

        self.bundle.dialogues.push(dialogue);
        Ok(())
    }

    /// Add a quest definition to the bundle (for hot-reload).
    #[wasm_bindgen]
    pub fn add_quest(&mut self, quest_json: &str) -> Result<(), JsValue> {
        let quest: blackwing_bundle::QuestDef = serde_json::from_str(quest_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse quest: {}", e)))?;

        self.bundle.quests.push(quest);
        Ok(())
    }

    /// Spawn an entity from a template.
    #[wasm_bindgen]
    pub fn spawn_from_template(&mut self, template_id: &str) -> Result<String, JsValue> {
        let parts: Vec<&str> = template_id.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(JsValue::from_str("Invalid template ID format (expected kind:id)"));
        }

        let entity_id = EntityId::new(parts[0], parts[1]);

        let template = self
            .bundle
            .get_template(&entity_id)
            .ok_or_else(|| JsValue::from_str(&format!("Template {} not found", template_id)))?
            .clone();

        // Spawn the entity
        let key = self.runtime.state_mut().entities.spawn(template.id.clone());

        // Apply components
        for (comp, val) in &template.components {
            self.runtime
                .state_mut()
                .entities
                .set_component(key, comp.clone(), val.clone());
        }

        // Apply tags
        for tag in &template.tags {
            self.runtime.state_mut().entities.add_tag(key, tag.clone());
        }

        Ok(template.id.as_qualified())
    }
}

/// Convert a RuntimeEvent to JSON
fn event_to_json(event: &RuntimeEvent) -> serde_json::Value {
    match event {
        RuntimeEvent::World(world_event) => {
            serde_json::json!({
                "type": "World",
                "event": format!("{:?}", world_event),
            })
        }
        RuntimeEvent::ActorOutput { actor, message } => {
            serde_json::json!({
                "type": "ActorOutput",
                "actor": actor.as_qualified(),
                "message": message,
            })
        }
        RuntimeEvent::ModeChanged(mode) => {
            serde_json::json!({
                "type": "ModeChanged",
                "mode": format!("{:?}", mode),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty_runner() {
        let runner = GameRunner::new_empty(12345);
        assert_eq!(runner.get_player_id(), "actor:player");
    }

    #[test]
    fn get_available_commands() {
        let runner = GameRunner::new_empty(12345);
        let commands = runner.get_available_commands();
        // Should have commands from all built-in systems
        assert!(commands.contains("go"));
        assert!(commands.contains("take"));
        assert!(commands.contains("start_dialogue"));
    }

    #[test]
    fn get_state() {
        let runner = GameRunner::new_empty(12345);
        let state_json = runner.get_state();
        // Parse the JSON to verify structure
        let state: serde_json::Value = serde_json::from_str(&state_json).unwrap();
        assert_eq!(state["tick"], 0);
        assert!(state["entity_count"].as_u64().is_some());
    }

    #[test]
    fn get_mode() {
        let runner = GameRunner::new_empty(12345);
        let mode = runner.get_mode();
        assert_eq!(mode, "turn_based");
    }

    #[test]
    fn json_to_value_conversion() {
        // Test basic types
        assert_eq!(json_to_value(serde_json::json!(null)), Value::Null);
        assert_eq!(json_to_value(serde_json::json!(true)), Value::Bool(true));
        assert_eq!(json_to_value(serde_json::json!(42)), Value::Int(42));
        assert_eq!(json_to_value(serde_json::json!(3.14)), Value::Float(3.14));
        assert_eq!(
            json_to_value(serde_json::json!("hello")),
            Value::String("hello".into())
        );

        // Test entity ref parsing
        let entity_ref = json_to_value(serde_json::json!("npc:bob"));
        match entity_ref {
            Value::EntityRef(id) => {
                assert_eq!(id.kind.as_str(), "npc");
                assert_eq!(id.id.as_str(), "bob");
            }
            _ => panic!("Expected EntityRef"),
        }

        // Test array
        let arr = json_to_value(serde_json::json!([1, 2, 3]));
        match arr {
            Value::Array(v) => assert_eq!(v.len(), 3),
            _ => panic!("Expected Array"),
        }
    }

    // Note: dispatch/tick tests require wasm32 target because they return
    // Result<String, JsValue> which uses wasm_bindgen internals.
    // These functions are tested via wasm-pack test in CI.
}
