//! Standard game systems implemented in Rhai.
//!
//! These systems provide common game functionality that can be:
//! - Used as-is by game authors
//! - Customized by overriding specific commands
//! - Replaced entirely with custom systems
//!
//! # Available Systems
//!
//! - **dialogue**: Dialogue trees with branching conversations
//! - **inventory**: Item pickup, drop, give, equip, use
//! - **movement**: Navigation between locations
//!
//! # Usage
//!
//! ```ignore
//! use blackwing_bundle::systems::standard_systems;
//!
//! // Get all standard systems as ScriptSystems
//! let systems = standard_systems()?;
//! for system in systems {
//!     runtime.register_system(Box::new(system));
//! }
//! ```

use blackwing_core::ScriptSystem;

/// Embedded Rhai script for the dialogue system
const DIALOGUE_SYSTEM: &str = include_str!("dialogue.rhai");

/// Embedded Rhai script for the inventory system
const INVENTORY_SYSTEM: &str = include_str!("inventory.rhai");

/// Embedded Rhai script for the movement system
const MOVEMENT_SYSTEM: &str = include_str!("movement.rhai");

/// Load all standard systems.
///
/// Returns a vector of `ScriptSystem` instances ready to be registered.
pub fn standard_systems() -> Result<Vec<ScriptSystem>, StandardSystemError> {
    let mut systems = Vec::new();

    // Load dialogue system
    systems.push(
        ScriptSystem::from_source(DIALOGUE_SYSTEM).map_err(|e| StandardSystemError::CompileError {
            system: "dialogue".into(),
            message: e,
        })?,
    );

    // Load inventory system
    systems.push(
        ScriptSystem::from_source(INVENTORY_SYSTEM).map_err(|e| {
            StandardSystemError::CompileError {
                system: "inventory".into(),
                message: e,
            }
        })?,
    );

    // Load movement system
    systems.push(
        ScriptSystem::from_source(MOVEMENT_SYSTEM).map_err(|e| {
            StandardSystemError::CompileError {
                system: "movement".into(),
                message: e,
            }
        })?,
    );

    Ok(systems)
}

/// Load a specific standard system by name.
pub fn standard_system(name: &str) -> Result<ScriptSystem, StandardSystemError> {
    let source = match name {
        "dialogue" => DIALOGUE_SYSTEM,
        "inventory" => INVENTORY_SYSTEM,
        "movement" => MOVEMENT_SYSTEM,
        _ => return Err(StandardSystemError::UnknownSystem(name.to_string())),
    };

    ScriptSystem::from_source(source).map_err(|e| StandardSystemError::CompileError {
        system: name.into(),
        message: e,
    })
}

/// Get the names of all available standard systems.
pub fn standard_system_names() -> &'static [&'static str] {
    &["dialogue", "inventory", "movement"]
}

/// Errors when loading standard systems.
#[derive(Debug, Clone)]
pub enum StandardSystemError {
    CompileError { system: String, message: String },
    UnknownSystem(String),
}

impl std::fmt::Display for StandardSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompileError { system, message } => {
                write!(f, "Failed to compile {} system: {}", system, message)
            }
            Self::UnknownSystem(name) => {
                write!(f, "Unknown standard system: {}", name)
            }
        }
    }
}

impl std::error::Error for StandardSystemError {}
