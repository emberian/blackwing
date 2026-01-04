//! World state - the complete game state.
//!
//! WorldState is the central data structure containing all game state:
//! - Entities and their components
//! - Global resources and flags
//! - Per-actor scope stacks
//! - Game time
//! - Chronicle/journal

use engine_primitives::{ActorId, EntityId, FlagId, GameTime, ResourceId, Rng, Value};
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{ChronicleEntry, EntityKey, EntityStorage, Scope, ScopeKind, ScopeStack, WorldEvent};

/// The complete world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// Game time (discrete or real-time)
    pub time: GameTime,

    /// RNG state for deterministic replay
    pub rng_state: u64,

    /// Global resources (credits, world_danger_level, etc.)
    pub resources: IndexMap<ResourceId, i64>,

    /// Global flags (story state, completed quests, etc.)
    pub flags: FxHashMap<FlagId, Value>,

    /// All entities and their components
    pub entities: EntityStorage,

    /// Per-actor scope stacks (nested contexts)
    pub scopes: FxHashMap<ActorId, ScopeStack>,

    /// Chronicle/journal entries
    pub chronicle: Vec<ChronicleEntry>,
}

impl WorldState {
    /// Create a new empty world state
    pub fn new() -> Self {
        Self {
            time: GameTime::default(),
            rng_state: 0,
            resources: IndexMap::new(),
            flags: FxHashMap::default(),
            entities: EntityStorage::new(),
            scopes: FxHashMap::default(),
            chronicle: Vec::new(),
        }
    }

    /// Create with a seed for RNG
    pub fn with_seed(seed: u64) -> Self {
        let mut state = Self::new();
        state.rng_state = seed;
        state
    }

    // === Resource Operations ===

    /// Get a resource value
    pub fn resource(&self, id: &ResourceId) -> i64 {
        self.resources.get(id).copied().unwrap_or(0)
    }

    /// Set a resource value
    pub fn set_resource(&mut self, id: ResourceId, value: i64) {
        self.resources.insert(id, value);
    }

    /// Modify a resource by delta
    pub fn modify_resource(&mut self, id: &ResourceId, delta: i64) {
        let current = self.resource(id);
        self.resources.insert(id.clone(), current + delta);
    }

    // === Flag Operations ===

    /// Get a flag value
    pub fn flag(&self, id: &FlagId) -> &Value {
        static NULL: Value = Value::Null;
        self.flags.get(id).unwrap_or(&NULL)
    }

    /// Check if a flag is truthy
    pub fn flag_is_truthy(&self, id: &FlagId) -> bool {
        self.flag(id).is_truthy()
    }

    /// Set a flag value
    pub fn set_flag(&mut self, id: FlagId, value: Value) {
        self.flags.insert(id, value);
    }

    // === Scope Operations ===

    /// Get the scope stack for an actor
    pub fn scope_stack(&self, actor: &ActorId) -> Option<&ScopeStack> {
        self.scopes.get(actor)
    }

    /// Get the scope stack for an actor mutably
    pub fn scope_stack_mut(&mut self, actor: &ActorId) -> &mut ScopeStack {
        self.scopes
            .entry(actor.clone())
            .or_insert_with(ScopeStack::new)
    }

    /// Push a scope for an actor
    pub fn push_scope(&mut self, actor: &ActorId, scope: Scope) {
        self.scope_stack_mut(actor).push(scope);
    }

    /// Pop a scope for an actor
    pub fn pop_scope(&mut self, actor: &ActorId) -> Option<Scope> {
        self.scopes.get_mut(actor)?.pop()
    }

    /// Get the current scope for an actor
    pub fn current_scope(&self, actor: &ActorId) -> Option<&Scope> {
        self.scopes.get(actor)?.current()
    }

    /// Check if an actor is in a scope of the given kind
    pub fn actor_is_in(&self, actor: &ActorId, kind: &ScopeKind) -> bool {
        self.scopes
            .get(actor)
            .map(|s| s.is_in(kind))
            .unwrap_or(false)
    }

    // === Entity Convenience Methods ===

    /// Spawn an entity
    pub fn spawn(&mut self, id: EntityId) -> EntityKey {
        self.entities.spawn(id)
    }

    /// Despawn an entity by key
    pub fn despawn(&mut self, key: EntityKey) -> Option<crate::Entity> {
        self.entities.despawn(key)
    }

    /// Get an entity by ID
    pub fn entity(&self, id: &EntityId) -> Option<&crate::Entity> {
        self.entities.get_by_id(id)
    }

    /// Check if an entity exists
    pub fn has_entity(&self, id: &EntityId) -> bool {
        self.entities.contains_id(id)
    }

    // === Chronicle Operations ===

    /// Add a chronicle entry
    pub fn add_chronicle(&mut self, entry: ChronicleEntry) {
        self.chronicle.push(entry);
    }

    // === Time Operations ===

    /// Get the current tick
    pub fn tick(&self) -> u64 {
        self.time.tick()
    }

    /// Advance time
    pub fn advance_time(&mut self, delta_ms: u64) {
        self.time.advance(delta_ms);
    }

    // === Event Application ===

    /// Apply a world event to mutate state
    pub fn apply_event(&mut self, event: &WorldEvent) {
        match event {
            WorldEvent::EntitySpawned { id, .. } => {
                self.entities.spawn(id.clone());
            }

            WorldEvent::EntityDespawned { key, .. } => {
                self.entities.despawn(*key);
            }

            WorldEvent::ComponentSet {
                entity,
                component,
                new_value,
                ..
            } => {
                self.entities
                    .set_component(*entity, component.clone(), new_value.clone());
            }

            WorldEvent::ComponentRemoved {
                entity, component, ..
            } => {
                self.entities.remove_component(*entity, component);
            }

            WorldEvent::TagAdded { entity, tag } => {
                self.entities.add_tag(*entity, tag.clone());
            }

            WorldEvent::TagRemoved { entity, tag } => {
                self.entities.remove_tag(*entity, tag);
            }

            WorldEvent::ResourceChanged {
                resource,
                new_value,
                ..
            } => {
                self.resources.insert(resource.clone(), *new_value);
            }

            WorldEvent::FlagSet { flag, new_value, .. } => {
                self.flags.insert(flag.clone(), new_value.clone());
            }

            WorldEvent::ScopePushed { actor, scope } => {
                self.push_scope(actor, scope.clone());
            }

            WorldEvent::ScopePopped { actor, .. } => {
                self.pop_scope(actor);
            }

            WorldEvent::ScopeStateChanged {
                actor,
                scope_kind,
                key,
                new_value,
                ..
            } => {
                if let Some(stack) = self.scopes.get_mut(actor) {
                    if let Some(scope) = stack.find_mut(scope_kind) {
                        scope.set(key.clone(), new_value.clone());
                    }
                }
            }

            WorldEvent::TimeAdvanced { new_tick, .. } => {
                // For discrete time, set tick directly
                if let GameTime::Discrete { tick } = &mut self.time {
                    *tick = *new_tick;
                }
            }

            WorldEvent::RngAdvanced { new_state } => {
                self.rng_state = *new_state;
            }

            WorldEvent::ChronicleAdded { entry } => {
                self.chronicle.push(entry.clone());
            }
        }
    }

    /// Create an RNG from current state
    pub fn rng(&self) -> Rng {
        Rng::new(self.rng_state)
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resources() {
        let mut state = WorldState::new();
        let credits = ResourceId::new("credits");

        assert_eq!(state.resource(&credits), 0);

        state.set_resource(credits.clone(), 100);
        assert_eq!(state.resource(&credits), 100);

        state.modify_resource(&credits, -30);
        assert_eq!(state.resource(&credits), 70);
    }

    #[test]
    fn flags() {
        let mut state = WorldState::new();
        let flag = FlagId::new("quest_started");

        assert!(!state.flag_is_truthy(&flag));

        state.set_flag(flag.clone(), Value::Bool(true));
        assert!(state.flag_is_truthy(&flag));
    }

    #[test]
    fn scopes() {
        let mut state = WorldState::new();
        let player = ActorId::new("actor", "player");

        state.push_scope(&player, Scope::new(ScopeKind::Location, "tavern"));
        state.push_scope(&player, Scope::new(ScopeKind::Conversation, "barkeep"));

        assert!(state.actor_is_in(&player, &ScopeKind::Location));
        assert!(state.actor_is_in(&player, &ScopeKind::Conversation));

        let popped = state.pop_scope(&player).unwrap();
        assert_eq!(popped.kind, ScopeKind::Conversation);
        assert!(!state.actor_is_in(&player, &ScopeKind::Conversation));
    }

    #[test]
    fn apply_events() {
        let mut state = WorldState::new();
        let credits = ResourceId::new("credits");

        let event = WorldEvent::resource_changed(credits.clone(), 0, 100, "initial");
        state.apply_event(&event);

        assert_eq!(state.resource(&credits), 100);
    }
}
