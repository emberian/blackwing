//! Rhai scripting integration for the game engine.
//!
//! This module provides:
//! - `ScriptSystem`: A System implementation that wraps Rhai scripts
//! - `ScriptExecutor`: Executes Rhai scripts with world state access
//! - API modules for ECS, world state, and time operations
//!
//! # Example
//!
//! ```rhai
//! // Define a system in Rhai
//! const SYSTEM_ID = "my_combat";
//! const HANDLES = ["attack", "defend"];
//!
//! fn handle_command(world, command, ctx) {
//!     if command.kind == "attack" {
//!         let target = command.args["target"];
//!         let damage = rng_int(5, 15);
//!         modify_resource(target + ":health", -damage);
//!         chronicle("Attack", "Dealt " + damage + " damage!");
//!     }
//! }
//!
//! fn on_tick(world, delta, ctx) {
//!     // Per-tick logic (optional)
//! }
//! ```

mod executor;
mod script_system;
mod shared_state;

pub use executor::{ScriptExecutor, value_to_dynamic_public};
pub use script_system::ScriptSystem;
pub use shared_state::{ScriptEffect, ScriptState};

// Re-export rhai for advanced users
pub use rhai;
