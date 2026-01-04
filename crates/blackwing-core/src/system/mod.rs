//! Game systems for the engine.
//!
//! This module provides:
//! - The `System` trait for implementing game logic
//! - `SystemRegistry` for managing and routing to systems
//! - `Command` and `Effect` types for system I/O
//!
//! # Architecture
//!
//! Systems are the building blocks of game logic. They:
//! - Handle commands from actors (players, NPCs, etc.)
//! - Run per-tick logic for real-time behavior
//! - Return effects that modify world state
//! - Register Rhai functions for scripting

mod command;
mod context;
mod effect;
mod error;
mod registry;
mod traits;

pub use command::{commands, Command};
pub use context::{SystemContext, WorldView};
pub use effect::Effect;
pub use error::SystemError;
pub use registry::SystemRegistry;
pub use traits::{BoxedSystem, System, SystemConfig};
