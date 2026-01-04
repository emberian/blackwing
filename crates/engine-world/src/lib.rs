//! World state management for the generalized game engine.
//!
//! This crate provides the core data structures for game state:
//! - Entity storage with component system
//! - Scope stacks for nested contexts
//! - World state combining all state
//! - Events for tracking state changes
//! - Query system for finding entities

mod entity;
mod event;
mod query;
mod scope;
mod state;

pub use entity::{Entity, EntityKey, EntityStorage};
pub use event::{ChronicleEntry, WorldEvent};
pub use query::{component_eq, component_gt, component_lt, EntityQuery, QueryExt};
pub use scope::{Scope, ScopeKind, ScopeStack};
pub use state::WorldState;

// Re-export commonly used types from engine-primitives
pub use engine_primitives::{
    ActorId, EntityId, FlagId, GameMode, GameTime, ResourceId, Rng, Tags, TagsExt, Value,
};
