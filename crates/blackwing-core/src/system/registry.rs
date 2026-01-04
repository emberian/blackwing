//! System registry manages registered systems and routes commands.

use rustc_hash::FxHashMap;
use smol_str::SmolStr;

use super::command::Command;
use super::context::{SystemContext, WorldView};
use super::effect::Effect;
use super::error::SystemError;
use super::traits::{BoxedSystem, System, SystemConfig};

/// Registry of game systems.
///
/// The registry:
/// - Stores all registered systems
/// - Routes commands to appropriate systems
/// - Manages system enable/disable state
/// - Provides system iteration for ticking
pub struct SystemRegistry {
    systems: Vec<BoxedSystem>,
    by_id: FxHashMap<SmolStr, usize>,
    command_handlers: FxHashMap<SmolStr, Vec<usize>>,
    enabled: FxHashMap<SmolStr, bool>,
}

impl SystemRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            by_id: FxHashMap::default(),
            command_handlers: FxHashMap::default(),
            enabled: FxHashMap::default(),
        }
    }

    /// Register a system
    pub fn register(&mut self, system: BoxedSystem) {
        let id: SmolStr = system.id().into();
        let index = self.systems.len();

        // Index by ID
        self.by_id.insert(id.clone(), index);

        // Index by handled commands (use iter for dynamic systems like ScriptSystem)
        for cmd_kind in system.handles_commands_iter() {
            self.command_handlers
                .entry(cmd_kind.into())
                .or_default()
                .push(index);
        }

        // Set default enabled state
        self.enabled.insert(id, system.enabled_by_default());

        self.systems.push(system);
    }

    /// Apply configuration to systems
    pub fn configure(&mut self, configs: &[SystemConfig]) {
        for config in configs {
            self.enabled
                .insert(config.system_id.clone(), config.enabled);
        }
    }

    /// Check if a system is enabled
    pub fn is_enabled(&self, system_id: &str) -> bool {
        self.enabled.get(system_id).copied().unwrap_or(false)
    }

    /// Enable or disable a system
    pub fn set_enabled(&mut self, system_id: &str, enabled: bool) {
        self.enabled.insert(system_id.into(), enabled);
    }

    /// Get a system by ID
    pub fn get(&self, system_id: &str) -> Option<&dyn System> {
        self.by_id
            .get(system_id)
            .map(|&idx| self.systems[idx].as_ref())
    }

    /// Iterate over all enabled systems
    pub fn enabled_systems(&self) -> impl Iterator<Item = &dyn System> {
        self.systems.iter().filter_map(|sys| {
            if self.is_enabled(sys.id()) {
                Some(sys.as_ref())
            } else {
                None
            }
        })
    }

    /// Route a command to the appropriate system(s) and collect effects.
    ///
    /// Returns effects from the first system that successfully handles the command.
    pub fn dispatch(
        &self,
        world: &WorldView<'_>,
        command: &Command,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        let handlers = self
            .command_handlers
            .get(&command.kind)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        if handlers.is_empty() {
            return Err(SystemError::UnknownCommand(command.kind.clone()));
        }

        // Sort handlers by priority (higher first)
        let mut sorted_handlers: Vec<_> = handlers
            .iter()
            .filter_map(|&idx| {
                let sys = &self.systems[idx];
                if self.is_enabled(sys.id()) {
                    Some((idx, sys.priority()))
                } else {
                    None
                }
            })
            .collect();
        sorted_handlers.sort_by(|a, b| b.1.cmp(&a.1));

        // Try handlers in priority order
        for (idx, _) in sorted_handlers {
            let system = &self.systems[idx];

            // Check scope filter
            let scope_filter = system.scope_filter();
            if !scope_filter.is_empty() {
                if let Some(scopes) = world.scopes(&ctx.actor) {
                    if let Some(current) = scopes.current() {
                        if !scope_filter.contains(&current.kind) {
                            continue;
                        }
                    }
                }
            }

            match system.handle_command(world, command, ctx) {
                Ok(effects) => return Ok(effects),
                Err(SystemError::NotInScope) => continue,
                Err(e) => return Err(e),
            }
        }

        Err(SystemError::UnknownCommand(command.kind.clone()))
    }

    /// Tick all enabled systems
    pub fn tick(
        &self,
        world: &WorldView<'_>,
        delta_ticks: u64,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        let mut all_effects = Vec::new();

        for system in self.enabled_systems() {
            let effects = system.tick(world, delta_ticks, ctx)?;
            all_effects.extend(effects);
        }

        Ok(all_effects)
    }

    /// Register Rhai functions from all systems
    pub fn register_rhai(&self, engine: &mut rhai::Engine) {
        for system in &self.systems {
            system.register_rhai(engine);
        }
    }

    /// Get the number of registered systems
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
}

impl Default for SystemRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityId, Rng, WorldState};

    struct TestSystem {
        id: &'static str,
        commands: Vec<&'static str>,
    }

    impl System for TestSystem {
        fn id(&self) -> &str {
            self.id
        }

        fn handles_commands(&self) -> &[&str] {
            &self.commands
        }

        fn handle_command(
            &self,
            _world: &WorldView<'_>,
            command: &Command,
            _ctx: &mut SystemContext<'_>,
        ) -> Result<Vec<Effect>, SystemError> {
            Ok(vec![Effect::chronicle(
                format!("{} handled", self.id),
                format!("Command: {}", command.kind),
            )])
        }
    }

    #[test]
    fn register_and_get() {
        let mut registry = SystemRegistry::new();
        registry.register(Box::new(TestSystem {
            id: "test",
            commands: vec!["foo"],
        }));

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn dispatch_command() {
        let mut registry = SystemRegistry::new();
        registry.register(Box::new(TestSystem {
            id: "test",
            commands: vec!["foo", "bar"],
        }));

        let state = WorldState::new();
        let world = WorldView::new(&state);
        let mut rng = Rng::new(12345);
        let actor = EntityId::new("actor", "player");
        let mut ctx = SystemContext::new(actor, &mut rng);

        let effects = registry
            .dispatch(&world, &Command::new("foo"), &mut ctx)
            .unwrap();
        assert_eq!(effects.len(), 1);

        let result = registry.dispatch(&world, &Command::new("unknown"), &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn enable_disable() {
        let mut registry = SystemRegistry::new();
        registry.register(Box::new(TestSystem {
            id: "test",
            commands: vec!["foo"],
        }));

        assert!(registry.is_enabled("test"));

        registry.set_enabled("test", false);
        assert!(!registry.is_enabled("test"));

        let state = WorldState::new();
        let world = WorldView::new(&state);
        let mut rng = Rng::new(12345);
        let actor = EntityId::new("actor", "player");
        let mut ctx = SystemContext::new(actor, &mut rng);

        // Should fail because system is disabled
        let result = registry.dispatch(&world, &Command::new("foo"), &mut ctx);
        assert!(result.is_err());
    }
}
