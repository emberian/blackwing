//! Content types and registry for the generalized game engine.
//!
//! This crate provides:
//! - `EntityTemplate`: Reusable entity blueprints
//! - `DialogueTree`: Branching conversation trees
//! - `QuestDef`: Quest definitions with stages and objectives
//! - `ContentRegistry`: Trait for accessing game content
//!
//! # Example
//!
//! ```
//! use engine_content::{EntityTemplate, DialogueTree, QuestDef, SimpleRegistry, ContentRegistry};
//! use engine_primitives::{EntityId, Value};
//!
//! let mut registry = SimpleRegistry::new();
//!
//! // Add an NPC template
//! registry.add_template(
//!     EntityTemplate::new(EntityId::new("npc", "merchant"))
//!         .with_component("name", Value::String("Bob".into()))
//!         .with_component("gold", Value::Int(500))
//!         .with_tag("friendly")
//!         .with_tag("merchant")
//! );
//!
//! // Add a dialogue
//! registry.add_dialogue(DialogueTree::new("merchant_greeting"));
//!
//! // Query content
//! let merchant = registry.get_template(&EntityId::new("npc", "merchant"));
//! assert!(merchant.is_some());
//! ```

mod dialogue;
mod quest;
mod registry;
mod template;

pub use dialogue::{DialogueNode, DialogueResponse, DialogueTree};
pub use quest::{Objective, QuestDef, QuestStage};
pub use registry::{ContentRegistry, SimpleRegistry};
pub use template::EntityTemplate;

// Re-export commonly used types
pub use engine_primitives::{EntityId, Value};
