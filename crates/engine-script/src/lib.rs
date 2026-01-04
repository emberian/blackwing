mod analysis;
mod error;
mod executor;
pub mod world_bindings;

pub use analysis::{analyze_ast, analyze_script, AnalyzedEffect, Branch, ScriptAnalysis, StateRef};
pub use error::ScriptError;
pub use executor::{ScriptExecutor, ScriptResult};
pub use world_bindings::{
    register_world_bindings, WorldEffect, WorldScriptState, WorldSnapshot,
};

// Re-export rhai for advanced users
pub use rhai;
