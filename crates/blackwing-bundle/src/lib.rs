//! Game bundles with script support for the Blackwing engine.
//!
//! A `GameBundle` is a self-contained package that includes:
//! - A manifest describing the game
//! - A schema defining resources, entity types, and components
//! - Content (entity templates, dialogues, quests)
//! - Scripts (Rhai systems, init scripts)
//!
//! Bundles can be loaded from:
//! - A directory (for development)
//! - A zip archive (.bwg file)
//! - Serialized bytes (for WASM)
//!
//! # Bundle Structure
//!
//! ```text
//! game.bwg/
//! ├── manifest.toml
//! ├── schema/
//! │   ├── components.toml
//! │   └── resources.toml
//! ├── scripts/
//! │   ├── systems/
//! │   │   ├── combat.rhai
//! │   │   └── weather.rhai
//! │   └── init.rhai
//! ├── content/
//! │   ├── entities/
//! │   │   ├── npcs.toml
//! │   │   └── items.toml
//! │   ├── dialogues/
//! │   └── quests/
//! ├── ui/
//! │   └── layouts/
//! └── assets/
//! ```

// === Content types ===
mod content;
pub use content::{
    ContentRegistry, DialogueNode, DialogueResponse, DialogueTree, EntityTemplate, Objective,
    QuestDef, QuestStage, SimpleRegistry,
};

// === Bundle types ===
mod error;
mod manifest;
mod schema;

pub use error::BundleError;
pub use manifest::BundleManifest;
pub use schema::{BundleSchema, ComponentDef, EntityTypeDef, InitialStateDef, ResourceDef, ValueType};

// === Bundle loader ===
mod bundle;
mod loader;

pub use bundle::GameBundle;
pub use loader::BundleLoader;

// === Standard systems ===
pub mod systems;
pub use systems::{standard_system, standard_system_names, standard_systems, StandardSystemError};

// Re-export commonly used types
pub use blackwing_core::{EntityId, ScriptSystem, Value};

// Re-export tilemap types for convenience
pub use blackwing_tilemap::{
    CollisionType, Direction, Room, RoomExit, SpawnPoint, TileAnimation, TileDef, TileId, TileSet,
    WorldMap,
};
