//! Built-in systems that games can optionally enable.
//!
//! These provide common game functionality:
//! - `scene`: Narrative scenes and choices
//! - `dialogue`: NPC conversations and branching dialogue trees
//! - `quest`: Quest tracking, stages, and objectives
//! - `inventory`: Item management, containers, equipment
//! - `movement`: Location/room navigation
//! - `time`: Time advancement and scheduling
//!
//! More systems can be added as needed:
//! - combat: Turn-based or real-time combat
//! - npc: NPC behavior and AI

mod dialogue;
mod inventory;
mod movement;
mod quest;
mod scene;
mod time;

pub use dialogue::DialogueSystem;
pub use inventory::InventorySystem;
pub use movement::MovementSystem;
pub use quest::QuestSystem;
pub use scene::SceneSystem;
pub use time::TimeSystem;

use crate::registry::SystemRegistry;

/// Register all built-in systems with the registry.
pub fn register_all(registry: &mut SystemRegistry) {
    registry.register(Box::new(SceneSystem::new()));
    registry.register(Box::new(DialogueSystem::new()));
    registry.register(Box::new(QuestSystem::new()));
    registry.register(Box::new(InventorySystem::new()));
    registry.register(Box::new(MovementSystem::new()));
    registry.register(Box::new(TimeSystem::new()));
}
