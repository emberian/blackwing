//! Core engine for Blackwing: ECS, systems, and scripting.
//!
//! This crate provides the fundamental building blocks:
//! - **Primitives**: Entity/resource identifiers, dynamic values, tags, time, RNG
//! - **ECS**: Entity storage, components, scopes, world state, queries
//! - **Systems**: System trait, command/effect handling, registry
//! - **Script**: Rhai integration with API modules (coming soon)

// === Primitives ===
mod id;
mod rng;
mod tags;
mod time;
mod value;

pub use id::*;
pub use rng::Rng;
pub use tags::{CategorizedTags, Tags, TagsExt};
pub use time::{GameMode, GameTime};
pub use value::Value;

// === ECS ===
pub mod ecs;

// Re-export commonly used ECS types at top level for convenience
pub use ecs::{
    ChronicleEntry, Entity, EntityKey, EntityQuery, EntityStorage, QueryExt, Scope, ScopeKind,
    ScopeStack, WorldEvent, WorldState,
};

// Query helpers
pub use ecs::{component_eq, component_gt, component_lt};

// === Systems ===
pub mod system;

// Re-export commonly used system types at top level
pub use system::{
    commands, BoxedSystem, Command, Effect, System, SystemConfig, SystemContext, SystemError,
    SystemRegistry, WorldView,
};

// === Script ===
pub mod script;

// Re-export commonly used script types at top level
pub use script::{ScriptEffect, ScriptExecutor, ScriptState, ScriptSystem};

// === UI ===
pub mod ui;

// Re-export commonly used UI types at top level
pub use ui::{UiChoice, UiNode, UiResource};
