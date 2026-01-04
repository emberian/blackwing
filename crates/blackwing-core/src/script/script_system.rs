//! ScriptSystem wraps a Rhai script to implement the System trait.

use rhai::{Dynamic, Scope, AST};
use smol_str::SmolStr;

use crate::system::{Command, Effect, System, SystemContext, SystemError, WorldView};
use crate::{EntityId, ScopeKind};

use super::executor::ScriptExecutor;
use super::shared_state::{ScriptEffect, ScriptState};

/// A system defined by a Rhai script.
///
/// The script should define:
/// - `SYSTEM_ID`: String constant with the system ID
/// - `HANDLES`: Array of command kinds this system handles
/// - `fn handle_command(command)`: Handler for commands
/// - `fn on_tick(delta)`: Optional tick handler
pub struct ScriptSystem {
    id: SmolStr,
    handles: Vec<SmolStr>,
    ast: AST,
    executor: ScriptExecutor,
    priority: i32,
    enabled: bool,
}

impl ScriptSystem {
    /// Create a new ScriptSystem from source code.
    ///
    /// Returns an error if the script doesn't compile or lacks required constants.
    pub fn from_source(source: &str) -> Result<Self, String> {
        let executor = ScriptExecutor::new();
        let ast = executor.compile(source).map_err(|e| e.to_string())?;

        // Extract constants by running the script in a scope and reading variables
        let (id, handles, priority, enabled) = Self::extract_metadata(&ast)?;

        Ok(Self {
            id,
            handles,
            ast,
            executor,
            priority,
            enabled,
        })
    }

    /// Create a ScriptSystem with a pre-compiled AST
    pub fn from_ast(
        id: impl Into<SmolStr>,
        handles: Vec<SmolStr>,
        ast: AST,
    ) -> Self {
        Self {
            id: id.into(),
            handles,
            ast,
            executor: ScriptExecutor::new(),
            priority: 0,
            enabled: true,
        }
    }

    /// Extract metadata from the compiled script by evaluating it
    fn extract_metadata(ast: &AST) -> Result<(SmolStr, Vec<SmolStr>, i32, bool), String> {
        let engine = rhai::Engine::new();
        let mut scope = Scope::new();

        // Run the script to populate constants
        engine.run_ast_with_scope(&mut scope, ast)
            .map_err(|e| e.to_string())?;

        // Extract SYSTEM_ID
        let id = scope
            .get_value::<rhai::ImmutableString>("SYSTEM_ID")
            .map(|s| SmolStr::new(s.as_str()))
            .unwrap_or_else(|| SmolStr::new("unknown"));

        // Extract HANDLES
        let handles = scope
            .get_value::<rhai::Array>("HANDLES")
            .map(|arr| {
                arr.into_iter()
                    .filter_map(|d| d.into_immutable_string().ok())
                    .map(|s| SmolStr::new(s.as_str()))
                    .collect()
            })
            .unwrap_or_default();

        // Extract optional PRIORITY
        let priority = scope
            .get_value::<i64>("PRIORITY")
            .map(|n| n as i32)
            .unwrap_or(0);

        // Extract optional ENABLED
        let enabled = scope
            .get_value::<bool>("ENABLED")
            .unwrap_or(true);

        Ok((id, handles, priority, enabled))
    }

    /// Get the handles for command routing
    pub fn handles_list(&self) -> &[SmolStr] {
        &self.handles
    }

    /// Check if this system handles a specific command
    pub fn handles_command(&self, kind: &str) -> bool {
        self.handles.iter().any(|h| h == kind)
    }

    /// Convert script effects to system effects
    fn convert_effects(script_effects: Vec<ScriptEffect>) -> Vec<Effect> {
        script_effects.into_iter().map(Self::convert_effect).collect()
    }

    fn convert_effect(se: ScriptEffect) -> Effect {
        match se {
            ScriptEffect::SpawnEntity { kind, id } => {
                Effect::SpawnEntity {
                    id: EntityId::new(kind.as_str(), id.as_str()),
                }
            }
            ScriptEffect::DespawnEntity { entity_id } => {
                Effect::DespawnEntity { id: entity_id }
            }
            ScriptEffect::SetComponent { entity_id, component, value } => {
                Effect::SetComponent {
                    entity: entity_id,
                    component,
                    value,
                }
            }
            ScriptEffect::RemoveComponent { entity_id, component } => {
                Effect::RemoveComponent {
                    entity: entity_id,
                    component,
                }
            }
            ScriptEffect::AddTag { entity_id, tag } => {
                Effect::AddTag {
                    entity: entity_id,
                    tag,
                }
            }
            ScriptEffect::RemoveTag { entity_id, tag } => {
                Effect::RemoveTag {
                    entity: entity_id,
                    tag,
                }
            }
            ScriptEffect::SetResource { resource, value } => {
                Effect::SetResource { resource, value }
            }
            ScriptEffect::ModifyResource { resource, delta } => {
                Effect::ModifyResource { resource, delta }
            }
            ScriptEffect::SetFlag { flag, value } => {
                Effect::SetFlag { flag, value }
            }
            ScriptEffect::ClearFlag { flag } => {
                Effect::ClearFlag { flag }
            }
            ScriptEffect::PushScope { actor, kind, id } => {
                Effect::PushScope { actor, kind, id }
            }
            ScriptEffect::PopScope { actor } => {
                Effect::PopScope { actor }
            }
            ScriptEffect::Chronicle { title, description } => {
                Effect::Chronicle { title, description }
            }
            ScriptEffect::AdvanceTime { ticks } => {
                Effect::AdvanceTime { ticks }
            }
            // UI effects
            ScriptEffect::SetUiRoot { root } => {
                Effect::SetUiRoot { root }
            }
            ScriptEffect::ShowModal { content, blocking } => {
                Effect::ShowModal { content, blocking }
            }
            ScriptEffect::CloseModal => {
                Effect::CloseModal
            }
            ScriptEffect::ClearUi => {
                Effect::ClearUi
            }
        }
    }

    /// Convert a Command to Rhai Dynamic for passing to script functions
    fn command_to_dynamic(command: &Command) -> Dynamic {
        let mut map = rhai::Map::new();
        map.insert("kind".into(), Dynamic::from(command.kind.to_string()));

        // Convert args to Dynamic map
        let mut args_map = rhai::Map::new();
        for (k, v) in &command.args {
            args_map.insert(k.to_string().into(), super::executor::value_to_dynamic_public(v));
        }
        map.insert("args".into(), Dynamic::from(args_map));

        Dynamic::from(map)
    }
}

impl System for ScriptSystem {
    fn id(&self) -> &str {
        &self.id
    }

    fn handles_commands(&self) -> &[&str] {
        // Return empty - use handles_commands_iter() instead for ScriptSystem
        &[]
    }

    fn handles_commands_iter(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(self.handles.iter().map(|s| s.as_str()))
    }

    fn handle_command(
        &self,
        world: &WorldView<'_>,
        command: &Command,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        // Check if we handle this command
        if !self.handles.iter().any(|h| h == &command.kind) {
            return Err(SystemError::UnknownCommand(command.kind.clone()));
        }

        // Create script state with a seed from the RNG
        let seed = ctx.rng.next_u64();
        let state = ScriptState::new(seed);
        state.set_actor(ctx.actor.clone());

        // Convert command to Dynamic
        let cmd_dyn = Self::command_to_dynamic(command);

        // Call the handle_command function
        let result: Result<(Dynamic, Vec<ScriptEffect>), _> = self.executor.call_fn(
            &self.ast,
            world.state(),
            state,
            "handle_command",
            (cmd_dyn,),
        );

        match result {
            Ok((_, effects)) => Ok(Self::convert_effects(effects)),
            Err(e) => {
                // Check if function doesn't exist - not an error, just no handler
                let err_str = e.to_string();
                if err_str.contains("Function not found") {
                    Ok(vec![])
                } else {
                    Err(SystemError::ScriptError(err_str))
                }
            }
        }
    }

    fn tick(
        &self,
        world: &WorldView<'_>,
        delta_ticks: u64,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        // Create script state
        let seed = ctx.rng.next_u64();
        let state = ScriptState::new(seed);
        state.set_actor(ctx.actor.clone());

        // Call the on_tick function
        let result: Result<(Dynamic, Vec<ScriptEffect>), _> = self.executor.call_fn(
            &self.ast,
            world.state(),
            state,
            "on_tick",
            (delta_ticks as i64,),
        );

        match result {
            Ok((_, effects)) => Ok(Self::convert_effects(effects)),
            Err(e) => {
                // Check if function doesn't exist - not an error, tick is optional
                let err_str = e.to_string();
                if err_str.contains("Function not found") {
                    Ok(vec![])
                } else {
                    Err(SystemError::ScriptError(err_str))
                }
            }
        }
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn enabled_by_default(&self) -> bool {
        self.enabled
    }

    fn scope_filter(&self) -> &[ScopeKind] {
        // Script systems don't have compile-time scope filters
        // They can check scope in the script itself
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rng, WorldState};

    #[test]
    fn create_script_system() {
        let source = r#"
            const SYSTEM_ID = "test_system";
            const HANDLES = ["test_cmd"];

            fn handle_command(cmd) {
                if cmd.kind == "test_cmd" {
                    chronicle("Test", "Handled!");
                }
            }
        "#;

        let system = ScriptSystem::from_source(source).unwrap();
        assert_eq!(system.id(), "test_system");
        assert!(system.handles_command("test_cmd"));
    }

    #[test]
    fn script_system_handles_command() {
        let source = r#"
            const SYSTEM_ID = "gold_system";
            const HANDLES = ["add_gold"];

            fn handle_command(cmd) {
                let amount = cmd.args["amount"];
                modify_resource("gold", amount);
            }
        "#;

        let system = ScriptSystem::from_source(source).unwrap();
        let state = WorldState::new();
        let world = WorldView::new(&state);
        let mut rng = Rng::new(12345);
        let actor = EntityId::new("actor", "player");
        let mut ctx = SystemContext::new(actor, &mut rng);

        let mut cmd = Command::new("add_gold");
        cmd.args.insert("amount".into(), crate::Value::Int(100));

        let effects = system.handle_command(&world, &cmd, &mut ctx).unwrap();
        assert_eq!(effects.len(), 1);
        assert!(matches!(&effects[0], Effect::ModifyResource { delta: 100, .. }));
    }

    #[test]
    fn extract_priority_and_enabled() {
        let source = r#"
            const SYSTEM_ID = "priority_test";
            const HANDLES = ["cmd"];
            const PRIORITY = 10;
            const ENABLED = false;
        "#;

        let system = ScriptSystem::from_source(source).unwrap();
        assert_eq!(system.priority(), 10);
        assert!(!system.enabled_by_default());
    }
}
