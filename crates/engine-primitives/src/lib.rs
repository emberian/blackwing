//! Core primitives for the generalized game engine.
//!
//! This crate provides the fundamental building blocks used throughout the engine:
//! - Entity and resource identifiers
//! - Dynamic values for component data
//! - Game time (discrete and real-time)
//! - Deterministic random number generation
//! - Tag system for entity classification

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
