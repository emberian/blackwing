mod analysis;
mod error;
mod executor;

pub use analysis::{analyze_ast, analyze_script, AnalyzedEffect, Branch, ScriptAnalysis, StateRef};
pub use error::ScriptError;
pub use executor::{ScriptExecutor, ScriptResult};

// Re-export rhai for advanced users
pub use rhai;
