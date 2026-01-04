//! The core System trait that all game systems implement.

use crate::ScopeKind;
use smol_str::SmolStr;

use super::command::Command;
use super::context::{SystemContext, WorldView};
use super::effect::Effect;
use super::error::SystemError;

/// A game system that handles commands and ticks.
///
/// Systems are the building blocks of game logic. They:
/// - Handle specific command types (e.g., movement, combat, dialogue)
/// - Run per-tick logic (e.g., AI behavior, timers)
/// - Register Rhai functions for scripting
/// - Filter by scope (only active in certain contexts)
pub trait System: Send + Sync {
    /// Unique identifier for this system
    fn id(&self) -> &str;

    /// Which scope kinds this system is active in.
    /// Empty means active in all scopes.
    fn scope_filter(&self) -> &[ScopeKind] {
        &[]
    }

    /// Command kinds this system handles.
    /// Used for routing commands to the appropriate system.
    fn handles_commands(&self) -> &[&str] {
        &[]
    }

    /// Iterator over handled command kinds.
    /// Override this for dynamic systems (like ScriptSystem) that can't return &[&str].
    fn handles_commands_iter(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(self.handles_commands().iter().copied())
    }

    /// Handle a command from an actor.
    ///
    /// Returns effects to apply to the world, or an error.
    fn handle_command(
        &self,
        world: &WorldView<'_>,
        command: &Command,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError>;

    /// Tick logic that runs every game tick.
    ///
    /// Not all systems need tick logic - default is a no-op.
    fn tick(
        &self,
        _world: &WorldView<'_>,
        _delta_ticks: u64,
        _ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        Ok(vec![])
    }

    /// Register Rhai functions for this system.
    ///
    /// Called once during engine initialization.
    fn register_rhai(&self, _engine: &mut rhai::Engine) {
        // Default: no Rhai functions
    }

    /// Priority for command handling (higher = handles first).
    /// Used when multiple systems could handle the same command.
    fn priority(&self) -> i32 {
        0
    }

    /// Whether this system is enabled by default.
    fn enabled_by_default(&self) -> bool {
        true
    }
}

/// A boxed system for dynamic dispatch
pub type BoxedSystem = Box<dyn System>;

/// System configuration from game schema
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub system_id: SmolStr,
    pub enabled: bool,
    pub config: indexmap::IndexMap<SmolStr, crate::Value>,
}

impl SystemConfig {
    pub fn new(system_id: impl Into<SmolStr>) -> Self {
        Self {
            system_id: system_id.into(),
            enabled: true,
            config: indexmap::IndexMap::new(),
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_config(mut self, key: impl Into<SmolStr>, value: crate::Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }
}
