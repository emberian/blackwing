mod analysis;
pub mod animation_bindings;
pub mod collision_bindings;
pub mod combat_bindings;
mod error;
mod executor;
pub mod input_bindings;
pub mod tilemap_bindings;
pub mod world_bindings;

pub use analysis::{analyze_ast, analyze_script, AnalyzedEffect, Branch, ScriptAnalysis, StateRef};
pub use animation_bindings::{register_animation_bindings, AnimationSnapshot, AnimationState};
pub use collision_bindings::{register_collision_bindings, CollisionSnapshot, EntityHitbox};
pub use combat_bindings::register_combat_bindings;
pub use error::ScriptError;
pub use executor::{ScriptExecutor, ScriptResult};
pub use input_bindings::{register_input_bindings, InputSnapshot};
pub use tilemap_bindings::{register_tilemap_bindings, TilemapSnapshot, TileSnapshot};
pub use world_bindings::{
    register_world_bindings, WorldEffect, WorldScriptState, WorldSnapshot,
};

// Re-export rhai for advanced users
pub use rhai;
