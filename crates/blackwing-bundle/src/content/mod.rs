//! Content types and registry for the game engine.
//!
//! This module provides:
//! - `EntityTemplate`: Reusable entity blueprints
//! - `DialogueTree`: Branching conversation trees
//! - `QuestDef`: Quest definitions with stages and objectives
//! - `ContentRegistry`: Trait for accessing game content

mod dialogue;
mod quest;
mod registry;
mod template;

pub use dialogue::{DialogueNode, DialogueResponse, DialogueTree};
pub use quest::{Objective, QuestDef, QuestStage};
pub use registry::{ContentRegistry, SimpleRegistry};
pub use template::EntityTemplate;
