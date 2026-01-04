//! Built-in systems that games can optionally enable.
//!
//! These provide common game functionality:
//! - `scene`: Narrative scenes and choices
//! - `movement`: Location/room navigation
//! - `time`: Time advancement and scheduling
//!
//! More systems can be added as needed:
//! - dialogue: NPC conversations
//! - inventory: Item management
//! - combat: Turn-based or real-time combat
//! - quest: Quest tracking and objectives
//! - npc: NPC behavior and AI

mod scene;
mod time;

pub use scene::SceneSystem;
pub use time::TimeSystem;

use crate::registry::SystemRegistry;

/// Register all built-in systems with the registry.
pub fn register_all(registry: &mut SystemRegistry) {
    registry.register(Box::new(SceneSystem::new()));
    registry.register(Box::new(TimeSystem::new()));
}
