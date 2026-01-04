//! Game systems for the generalized engine.
//!
//! This crate provides:
//! - The `System` trait for implementing game logic
//! - `SystemRegistry` for managing and routing to systems
//! - `Command` and `Effect` types for system I/O
//! - Built-in systems for common game functionality
//!
//! # Architecture
//!
//! Systems are the building blocks of game logic. They:
//! - Handle commands from actors (players, NPCs, etc.)
//! - Run per-tick logic for real-time behavior
//! - Return effects that modify world state
//! - Register Rhai functions for scripting
//!
//! # Example
//!
//! ```ignore
//! use engine_systems::{System, SystemRegistry, Command, Effect};
//!
//! struct MySystem;
//!
//! impl System for MySystem {
//!     fn id(&self) -> &str { "my_system" }
//!
//!     fn handles_commands(&self) -> &[&str] { &["my_command"] }
//!
//!     fn handle_command(&self, world: &WorldView, cmd: &Command, ctx: &mut SystemContext)
//!         -> Result<Vec<Effect>, SystemError>
//!     {
//!         Ok(vec![Effect::chronicle("Did thing", "Description")])
//!     }
//! }
//!
//! let mut registry = SystemRegistry::new();
//! registry.register(Box::new(MySystem));
//! ```

pub mod builtin;
mod command;
mod context;
mod effect;
mod error;
mod registry;
mod system;

pub use command::{commands, Command};
pub use context::{SystemContext, WorldView};
pub use effect::Effect;
pub use error::SystemError;
pub use registry::SystemRegistry;
pub use system::{BoxedSystem, System, SystemConfig};

// Re-export commonly used types
pub use engine_world::ScopeKind;
