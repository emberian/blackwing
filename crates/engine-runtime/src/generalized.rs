//! Generalized runtime for the multi-genre game engine.
//!
//! This runtime supports:
//! - Multiple game types (MUD, visual novel, space trader, etc.)
//! - Multi-actor gameplay
//! - Turn-based or real-time modes
//! - System-based command dispatch

use blackwing_core::{
    BoxedSystem, ChronicleEntry, Command, Effect, EntityId, GameMode, Rng, Scope, ScopeStack,
    SystemContext, SystemError, SystemRegistry, Value, WorldEvent, WorldState, WorldView,
};
use thiserror::Error;

/// Errors that can occur in the generalized runtime.
#[derive(Debug, Error)]
pub enum GeneralizedRuntimeError {
    #[error("System error: {0}")]
    System(#[from] SystemError),

    #[error("Actor not found: {0}")]
    ActorNotFound(String),

    #[error("Game over: {0}")]
    GameOver(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Script error: {0}")]
    ScriptError(String),
}

/// Event emitted by the runtime.
#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    /// A world event occurred
    World(WorldEvent),
    /// An actor received output (for MUD-style games)
    ActorOutput { actor: EntityId, message: String },
    /// Game mode changed
    ModeChanged(GameMode),
}

/// The generalized runtime for the game engine.
pub struct GeneralizedRuntime {
    state: WorldState,
    systems: SystemRegistry,
    event_log: Vec<RuntimeEvent>,
    rng: Rng,
    mode: GameMode,
}

impl GeneralizedRuntime {
    /// Create a new runtime with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: WorldState::new(),
            systems: SystemRegistry::new(),
            event_log: Vec::new(),
            rng: Rng::new(seed),
            mode: GameMode::TurnBased,
        }
    }

    /// Create a runtime with initial state.
    pub fn with_state(state: WorldState, seed: u64) -> Self {
        Self {
            state,
            systems: SystemRegistry::new(),
            event_log: Vec::new(),
            rng: Rng::new(seed),
            mode: GameMode::TurnBased,
        }
    }

    /// Get the current world state.
    pub fn state(&self) -> &WorldState {
        &self.state
    }

    /// Get mutable access to world state.
    pub fn state_mut(&mut self) -> &mut WorldState {
        &mut self.state
    }

    /// Get the system registry.
    pub fn systems(&self) -> &SystemRegistry {
        &self.systems
    }

    /// Get mutable access to the system registry.
    pub fn systems_mut(&mut self) -> &mut SystemRegistry {
        &mut self.systems
    }

    /// Get the event log.
    pub fn event_log(&self) -> &[RuntimeEvent] {
        &self.event_log
    }

    /// Get the current game mode.
    pub fn mode(&self) -> &GameMode {
        &self.mode
    }

    /// Set the game mode.
    pub fn set_mode(&mut self, mode: GameMode) {
        self.event_log.push(RuntimeEvent::ModeChanged(mode.clone()));
        self.mode = mode;
    }

    /// Get the RNG.
    pub fn rng(&self) -> &Rng {
        &self.rng
    }

    /// Get mutable RNG.
    pub fn rng_mut(&mut self) -> &mut Rng {
        &mut self.rng
    }

    /// Register a system.
    pub fn register_system(&mut self, system: BoxedSystem) {
        self.systems.register(system);
    }

    /// Dispatch a command from an actor.
    ///
    /// The command is routed to the appropriate system, which returns effects.
    /// Effects are then applied to the world state.
    pub fn dispatch(
        &mut self,
        actor: EntityId,
        command: Command,
    ) -> Result<Vec<RuntimeEvent>, GeneralizedRuntimeError> {
        let world = WorldView::new(&self.state);
        let mut ctx = SystemContext::new(actor.clone(), &mut self.rng);

        // Dispatch to systems
        let effects = self.systems.dispatch(&world, &command, &mut ctx)?;

        // Apply effects
        let events = self.apply_effects(&actor, effects)?;

        // Log events
        self.event_log.extend(events.clone());

        Ok(events)
    }

    /// Tick the game (for real-time modes).
    ///
    /// Advances time and runs tick logic on all systems.
    pub fn tick(&mut self, delta_ms: u64) -> Result<Vec<RuntimeEvent>, GeneralizedRuntimeError> {
        let delta_ticks = match &self.mode {
            GameMode::TurnBased => 1,
            GameMode::RealTime { tick_rate_ms } => delta_ms / (*tick_rate_ms).max(1),
            GameMode::Hybrid { tick_rate_ms, .. } => delta_ms / (*tick_rate_ms).max(1),
        };

        if delta_ticks == 0 {
            return Ok(vec![]);
        }

        // For now, use a "system" actor for tick operations
        let system_actor = EntityId::new("system", "tick");
        let world = WorldView::new(&self.state);
        let mut ctx = SystemContext::new(system_actor.clone(), &mut self.rng);

        // Tick all systems
        let effects = self.systems.tick(&world, delta_ticks, &mut ctx)?;

        // Apply effects
        let events = self.apply_effects(&system_actor, effects)?;

        // Advance time
        self.state.advance_time(delta_ticks);

        self.event_log.extend(events.clone());

        Ok(events)
    }

    /// Apply effects and return generated events.
    fn apply_effects(
        &mut self,
        actor: &EntityId,
        effects: Vec<Effect>,
    ) -> Result<Vec<RuntimeEvent>, GeneralizedRuntimeError> {
        let mut events = Vec::new();

        for effect in effects {
            match effect {
                Effect::SpawnEntity { id } => {
                    let key = self.state.entities.spawn(id.clone());
                    events.push(RuntimeEvent::World(WorldEvent::entity_spawned(key, id)));
                }

                Effect::DespawnEntity { id } => {
                    if let Some(key) = self.state.entities.key_of(&id) {
                        self.state.entities.despawn(key);
                        events.push(RuntimeEvent::World(WorldEvent::entity_despawned(key, id)));
                    }
                }

                Effect::SetComponent {
                    entity,
                    component,
                    value,
                } => {
                    if let Some(key) = self.state.entities.key_of(&entity) {
                        let old = self
                            .state
                            .entities
                            .get_component(key, &component)
                            .cloned();
                        self.state
                            .entities
                            .set_component(key, component.clone(), value.clone());
                        events.push(RuntimeEvent::World(WorldEvent::component_set(
                            key, component, old, value,
                        )));
                    }
                }

                Effect::RemoveComponent { entity, component } => {
                    if let Some(key) = self.state.entities.key_of(&entity) {
                        if let Some(old_val) = self.state.entities.remove_component(key, &component)
                        {
                            events.push(RuntimeEvent::World(WorldEvent::ComponentRemoved {
                                entity: key,
                                component,
                                old_value: old_val,
                            }));
                        }
                    }
                }

                Effect::AddTag { entity, tag } => {
                    if let Some(key) = self.state.entities.key_of(&entity) {
                        self.state.entities.add_tag(key, tag.clone());
                        events.push(RuntimeEvent::World(WorldEvent::TagAdded { entity: key, tag }));
                    }
                }

                Effect::RemoveTag { entity, tag } => {
                    if let Some(key) = self.state.entities.key_of(&entity) {
                        self.state.entities.remove_tag(key, &tag);
                        events.push(RuntimeEvent::World(WorldEvent::TagRemoved {
                            entity: key,
                            tag,
                        }));
                    }
                }

                Effect::SetResource { resource, value } => {
                    let old = self.state.resources.get(&resource).copied().unwrap_or(0);
                    self.state.resources.insert(resource.clone(), value);
                    events.push(RuntimeEvent::World(WorldEvent::resource_changed(
                        resource, old, value, "set",
                    )));
                }

                Effect::ModifyResource { resource, delta } => {
                    let old = self.state.resources.get(&resource).copied().unwrap_or(0);
                    let new = old.saturating_add(delta);
                    self.state.resources.insert(resource.clone(), new);
                    events.push(RuntimeEvent::World(WorldEvent::resource_changed(
                        resource, old, new, "modify",
                    )));
                }

                Effect::SetFlag { flag, value } => {
                    let old = self.state.flags.get(&flag).cloned();
                    self.state.flags.insert(flag.clone(), value.clone());
                    events.push(RuntimeEvent::World(WorldEvent::flag_set(flag, old, value)));
                }

                Effect::ClearFlag { flag } => {
                    let old = self.state.flags.remove(&flag);
                    if old.is_some() {
                        events.push(RuntimeEvent::World(WorldEvent::flag_set(
                            flag,
                            old,
                            Value::Null,
                        )));
                    }
                }

                Effect::PushScope {
                    actor: scope_actor,
                    kind,
                    id,
                } => {
                    let scope = Scope::new(kind, id);
                    self.state
                        .scopes
                        .entry(scope_actor.clone())
                        .or_insert_with(ScopeStack::new)
                        .push(scope.clone());
                    events.push(RuntimeEvent::World(WorldEvent::scope_pushed(
                        scope_actor,
                        scope,
                    )));
                }

                Effect::PopScope {
                    actor: scope_actor,
                } => {
                    if let Some(stack) = self.state.scopes.get_mut(&scope_actor) {
                        if let Some(scope) = stack.pop() {
                            events.push(RuntimeEvent::World(WorldEvent::scope_popped(
                                scope_actor,
                                scope,
                            )));
                        }
                    }
                }

                Effect::SetScopeLocal {
                    actor: scope_actor,
                    key,
                    value,
                } => {
                    if let Some(stack) = self.state.scopes.get_mut(&scope_actor) {
                        if let Some(scope) = stack.current_mut() {
                            scope.set(key, value);
                        }
                    }
                }

                Effect::Chronicle { title, description } => {
                    let tick = self.state.time.tick();
                    let entry = ChronicleEntry::new(
                        format!("entry_{}", self.state.chronicle.len()),
                        "narrative",
                        tick,
                        title.clone(),
                        description,
                    );
                    self.state.chronicle.push(entry.clone());
                    events.push(RuntimeEvent::World(WorldEvent::ChronicleAdded { entry }));
                }

                Effect::AdvanceTime { ticks } => {
                    let old_tick = self.state.time.tick();
                    self.state.advance_time(ticks);
                    let new_tick = self.state.time.tick();
                    events.push(RuntimeEvent::World(WorldEvent::time_advanced(
                        old_tick, new_tick,
                    )));
                }

                Effect::RunScript { script } => {
                    // Script execution would go here
                    events.push(RuntimeEvent::ActorOutput {
                        actor: actor.clone(),
                        message: format!("Script: {}", script),
                    });
                }

                Effect::Batch { effects } => {
                    // Recursively apply batched effects
                    events.extend(self.apply_effects(actor, effects)?);
                }

                // UI effects - these are passed through as events for the frontend
                Effect::SetUiRoot { root } => {
                    events.push(RuntimeEvent::World(WorldEvent::UiSet {
                        node: format!("{:?}", root),
                    }));
                }

                Effect::UpdateUi { path, node } => {
                    events.push(RuntimeEvent::World(WorldEvent::UiUpdated {
                        path: path.to_string(),
                        node: format!("{:?}", node),
                    }));
                }

                Effect::ShowModal { content, blocking } => {
                    events.push(RuntimeEvent::World(WorldEvent::ModalOpened {
                        content: format!("{:?}", content),
                        blocking,
                    }));
                }

                Effect::CloseModal => {
                    events.push(RuntimeEvent::World(WorldEvent::ModalClosed));
                }

                Effect::ClearUi => {
                    events.push(RuntimeEvent::World(WorldEvent::UiCleared));
                }

                // === Spatial effects ===

                Effect::SetPosition { entity, x, y } => {
                    if let Some(key) = self.state.entities.key_of(&entity) {
                        self.state
                            .entities
                            .set_component(key, "position_x", Value::Float(x.into()));
                        self.state
                            .entities
                            .set_component(key, "position_y", Value::Float(y.into()));
                        events.push(RuntimeEvent::World(WorldEvent::PositionSet {
                            entity: key,
                            x,
                            y,
                        }));
                    }
                }

                Effect::MoveBy { entity, dx, dy } => {
                    if let Some(key) = self.state.entities.key_of(&entity) {
                        // Get current position
                        let current_x = self
                            .state
                            .entities
                            .get_component(key, "position_x")
                            .and_then(|v| v.as_float())
                            .unwrap_or(0.0) as f32;
                        let current_y = self
                            .state
                            .entities
                            .get_component(key, "position_y")
                            .and_then(|v| v.as_float())
                            .unwrap_or(0.0) as f32;

                        let new_x = current_x + dx;
                        let new_y = current_y + dy;

                        self.state
                            .entities
                            .set_component(key, "position_x", Value::Float(new_x.into()));
                        self.state
                            .entities
                            .set_component(key, "position_y", Value::Float(new_y.into()));

                        events.push(RuntimeEvent::World(WorldEvent::EntityMoved {
                            entity: key,
                            from_x: current_x,
                            from_y: current_y,
                            to_x: new_x,
                            to_y: new_y,
                        }));
                    }
                }

                Effect::ChangeRoom {
                    room_id,
                    spawn_x,
                    spawn_y,
                } => {
                    events.push(RuntimeEvent::World(WorldEvent::RoomChanged {
                        room_id,
                        spawn_x,
                        spawn_y,
                    }));
                }

                Effect::SpawnAtPosition {
                    template_id,
                    x,
                    y,
                } => {
                    // Spawn the entity and set its position
                    let key = self.state.entities.spawn(template_id.clone());
                    self.state
                        .entities
                        .set_component(key, "position_x", Value::Float(x.into()));
                    self.state
                        .entities
                        .set_component(key, "position_y", Value::Float(y.into()));
                    events.push(RuntimeEvent::World(WorldEvent::entity_spawned(
                        key,
                        template_id,
                    )));
                    events.push(RuntimeEvent::World(WorldEvent::PositionSet {
                        entity: key,
                        x,
                        y,
                    }));
                }

                // === Combat effects ===

                Effect::DealDamage {
                    target,
                    amount,
                    source,
                } => {
                    if let Some(key) = self.state.entities.key_of(&target) {
                        // Check if target is invincible
                        let invincible = self
                            .state
                            .entities
                            .get_component(key, "invincible")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        if !invincible {
                            // Get current health
                            let current_hp = self
                                .state
                                .entities
                                .get_component(key, "health")
                                .and_then(|v| v.as_int())
                                .unwrap_or(0);

                            let new_hp = (current_hp - amount).max(0);
                            self.state
                                .entities
                                .set_component(key, "health", Value::Int(new_hp));

                            events.push(RuntimeEvent::World(WorldEvent::DamageTaken {
                                entity: key,
                                amount,
                                source: source.clone(),
                                health_remaining: new_hp,
                            }));

                            // Check for death
                            if new_hp <= 0 {
                                events.push(RuntimeEvent::World(WorldEvent::EntityDied {
                                    entity: key,
                                    killer: source,
                                }));
                            }
                        }
                    }
                }

                Effect::Heal { target, amount } => {
                    if let Some(key) = self.state.entities.key_of(&target) {
                        let current_hp = self
                            .state
                            .entities
                            .get_component(key, "health")
                            .and_then(|v| v.as_int())
                            .unwrap_or(0);

                        let max_hp = self
                            .state
                            .entities
                            .get_component(key, "max_health")
                            .and_then(|v| v.as_int())
                            .unwrap_or(current_hp + amount); // Default to allowing full heal

                        let new_hp = (current_hp + amount).min(max_hp);
                        self.state
                            .entities
                            .set_component(key, "health", Value::Int(new_hp));

                        events.push(RuntimeEvent::World(WorldEvent::Healed {
                            entity: key,
                            amount: new_hp - current_hp,
                            health_now: new_hp,
                        }));
                    }
                }

                Effect::SetInvincible { entity, duration_ms } => {
                    if let Some(key) = self.state.entities.key_of(&entity) {
                        self.state
                            .entities
                            .set_component(key, "invincible", Value::Bool(true));
                        self.state
                            .entities
                            .set_component(key, "invincible_timer", Value::Int(duration_ms as i64));

                        events.push(RuntimeEvent::World(WorldEvent::InvincibilitySet {
                            entity: key,
                            duration_ms,
                        }));
                    }
                }
            }
        }

        Ok(events)
    }

    /// Fork this runtime for parallel exploration (e.g., AI lookahead).
    pub fn fork(&mut self) -> GeneralizedRuntime {
        GeneralizedRuntime {
            state: self.state.clone(),
            // Systems are not cloned - the fork needs systems re-registered
            systems: SystemRegistry::new(),
            event_log: self.event_log.clone(),
            rng: self.rng.fork(),
            mode: self.mode.clone(),
        }
    }

    /// Spawn a player actor entity.
    pub fn spawn_actor(&mut self, id: &str) -> EntityId {
        let actor_id = EntityId::new("actor", id);
        self.state.entities.spawn(actor_id.clone());
        self.state
            .scopes
            .insert(actor_id.clone(), ScopeStack::new());
        actor_id
    }

    /// Get all actors.
    pub fn actors(&self) -> impl Iterator<Item = &EntityId> {
        self.state.scopes.keys()
    }

    /// Filter events visible to a specific actor.
    ///
    /// In MUD-style games, actors only see events relevant to them.
    pub fn events_for_actor<'a>(
        &'a self,
        _actor: &'a EntityId,
        events: &'a [RuntimeEvent],
    ) -> impl Iterator<Item = &'a RuntimeEvent> {
        // For now, return all events. In a full implementation, we'd filter
        // based on the actor's location, scope, etc.
        events.iter()
    }
}

impl Default for GeneralizedRuntime {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blackwing_core::System;

    struct EchoSystem;

    impl System for EchoSystem {
        fn id(&self) -> &str {
            "echo"
        }

        fn handles_commands(&self) -> &[&str] {
            &["echo", "spawn_test"]
        }

        fn handle_command(
            &self,
            _world: &WorldView<'_>,
            command: &Command,
            _ctx: &mut SystemContext<'_>,
        ) -> Result<Vec<Effect>, SystemError> {
            match command.kind.as_str() {
                "echo" => {
                    let msg = command.get_str("message").unwrap_or("no message");
                    Ok(vec![Effect::chronicle("Echo", msg)])
                }
                "spawn_test" => {
                    let id = EntityId::new("test", "entity");
                    Ok(vec![
                        Effect::spawn(id.clone()),
                        Effect::set_component(id.clone(), "name", Value::String("Test".into())),
                        Effect::chronicle("Spawned", "Created test entity"),
                    ])
                }
                _ => Ok(vec![]),
            }
        }
    }

    #[test]
    fn basic_dispatch() {
        let mut runtime = GeneralizedRuntime::new(12345);
        runtime.register_system(Box::new(EchoSystem));

        let actor = runtime.spawn_actor("player");
        let events = runtime
            .dispatch(
                actor,
                Command::new("echo").with_arg("message", Value::String("Hello".into())),
            )
            .unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            RuntimeEvent::World(WorldEvent::ChronicleAdded { entry }) => {
                assert_eq!(entry.title.as_str(), "Echo");
            }
            _ => panic!("Expected chronicle event"),
        }
    }

    #[test]
    fn spawn_entity() {
        let mut runtime = GeneralizedRuntime::new(12345);
        runtime.register_system(Box::new(EchoSystem));

        let actor = runtime.spawn_actor("player");
        let events = runtime
            .dispatch(actor, Command::new("spawn_test"))
            .unwrap();

        assert_eq!(events.len(), 3); // spawn + set_component + chronicle

        // Verify entity exists
        let test_id = EntityId::new("test", "entity");
        assert!(runtime.state().entities.contains_id(&test_id));

        // Verify component was set
        let key = runtime.state().entities.key_of(&test_id).unwrap();
        let name = runtime.state().entities.get_component(key, "name");
        assert_eq!(name.and_then(|v| v.as_str()), Some("Test"));
    }

    #[test]
    fn resource_modification() {
        use blackwing_core::ResourceId;

        let mut runtime = GeneralizedRuntime::new(12345);

        // Manually add resource
        let gold_id = ResourceId::new("gold");
        runtime.state_mut().resources.insert(gold_id.clone(), 100);

        // Verify
        assert_eq!(runtime.state().resources.get(&gold_id).copied(), Some(100));
    }

    #[test]
    fn fork_runtime() {
        use blackwing_core::ResourceId;

        let mut runtime = GeneralizedRuntime::new(12345);
        let gold_id = ResourceId::new("gold");
        runtime.state_mut().resources.insert(gold_id.clone(), 100);

        let mut forked = runtime.fork();
        forked.state_mut().resources.insert(gold_id.clone(), 50);

        // Original should be unchanged
        assert_eq!(runtime.state().resources.get(&gold_id).copied(), Some(100));
        // Forked should have new value
        assert_eq!(forked.state().resources.get(&gold_id).copied(), Some(50));
    }
}
