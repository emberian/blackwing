//! Commands represent actions that actors want to perform.
//!
//! Commands are dispatched to systems based on their kind. Systems can handle
//! commands and return effects to apply to the world.

use engine_primitives::{EntityId, Value};
use indexmap::IndexMap;
use smol_str::SmolStr;

/// A command from an actor to perform some action.
#[derive(Debug, Clone)]
pub struct Command {
    /// The kind of command (used for routing to systems)
    pub kind: SmolStr,
    /// Command arguments
    pub args: IndexMap<SmolStr, Value>,
}

impl Command {
    /// Create a new command with the given kind
    pub fn new(kind: impl Into<SmolStr>) -> Self {
        Self {
            kind: kind.into(),
            args: IndexMap::new(),
        }
    }

    /// Add an argument to the command
    pub fn with_arg(mut self, key: impl Into<SmolStr>, value: Value) -> Self {
        self.args.insert(key.into(), value);
        self
    }

    /// Get an argument by key
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.args.get(key)
    }

    /// Get a string argument
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.args.get(key).and_then(|v| v.as_str())
    }

    /// Get an int argument
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.args.get(key).and_then(|v| v.as_int())
    }

    /// Get an entity ID argument
    pub fn get_entity(&self, key: &str) -> Option<&EntityId> {
        self.args.get(key).and_then(|v| v.as_entity_ref())
    }
}

/// Common command kinds
pub mod commands {
    use super::*;

    /// Navigate to a location
    pub fn travel(destination: EntityId) -> Command {
        Command::new("travel").with_arg("destination", Value::EntityRef(destination))
    }

    /// Make a choice in a scene
    pub fn make_choice(scene_id: impl Into<SmolStr>, passage: usize, choice: usize) -> Command {
        Command::new("make_choice")
            .with_arg("scene_id", Value::String(scene_id.into()))
            .with_arg("passage", Value::Int(passage as i64))
            .with_arg("choice", Value::Int(choice as i64))
    }

    /// Interact with an entity
    pub fn interact(target: EntityId) -> Command {
        Command::new("interact").with_arg("target", Value::EntityRef(target))
    }

    /// Use an item
    pub fn use_item(item: EntityId) -> Command {
        Command::new("use_item").with_arg("item", Value::EntityRef(item))
    }

    /// Say something (for dialogue/chat)
    pub fn say(message: impl Into<SmolStr>) -> Command {
        Command::new("say").with_arg("message", Value::String(message.into()))
    }

    /// Attack a target
    pub fn attack(target: EntityId) -> Command {
        Command::new("attack").with_arg("target", Value::EntityRef(target))
    }

    /// Pick up an item
    pub fn pickup(item: EntityId) -> Command {
        Command::new("pickup").with_arg("item", Value::EntityRef(item))
    }

    /// Drop an item
    pub fn drop(item: EntityId) -> Command {
        Command::new("drop").with_arg("item", Value::EntityRef(item))
    }

    /// Look at surroundings or a target
    pub fn look(target: Option<EntityId>) -> Command {
        let mut cmd = Command::new("look");
        if let Some(t) = target {
            cmd = cmd.with_arg("target", Value::EntityRef(t));
        }
        cmd
    }

    /// Wait/pass time
    pub fn wait(ticks: u64) -> Command {
        Command::new("wait").with_arg("ticks", Value::Int(ticks as i64))
    }
}
