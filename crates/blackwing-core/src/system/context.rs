//! System context provides systems with access to world state and utilities.

use crate::{EntityId, EntityStorage, GameTime, ResourceId, Rng, ScopeStack, Value, WorldState};

/// Context provided to systems when handling commands or ticking.
pub struct SystemContext<'a> {
    /// The actor performing the command (for command handling)
    pub actor: EntityId,
    /// Random number generator for this operation
    pub rng: &'a mut Rng,
}

impl<'a> SystemContext<'a> {
    /// Create a new system context
    pub fn new(actor: EntityId, rng: &'a mut Rng) -> Self {
        Self { actor, rng }
    }

    /// Generate a random integer in range [0, max)
    pub fn random(&mut self, max: u32) -> u32 {
        self.rng.next_u32_range(max)
    }

    /// Generate a random float in range [0, 1)
    pub fn random_float(&mut self) -> f64 {
        self.rng.next_f64()
    }

    /// Check if a random chance succeeds (0.0 to 1.0)
    pub fn chance(&mut self, probability: f64) -> bool {
        self.rng.chance(probability)
    }
}

/// Read-only view of the world state for systems.
pub struct WorldView<'a> {
    state: &'a WorldState,
}

impl<'a> WorldView<'a> {
    pub fn new(state: &'a WorldState) -> Self {
        Self { state }
    }

    /// Get the underlying world state
    pub fn state(&self) -> &WorldState {
        self.state
    }

    /// Get the entity storage
    pub fn entities(&self) -> &EntityStorage {
        &self.state.entities
    }

    /// Get the current game time
    pub fn time(&self) -> &GameTime {
        &self.state.time
    }

    /// Check a flag value
    pub fn flag(&self, flag: &str) -> Option<&Value> {
        self.state.flags.get(&flag.into())
    }

    /// Get a resource value
    pub fn resource(&self, resource: &str) -> i64 {
        let key: ResourceId = resource.into();
        self.state.resources.get(&key).copied().unwrap_or(0)
    }

    /// Get an actor's scope stack
    pub fn scopes(&self, actor: &EntityId) -> Option<&ScopeStack> {
        self.state.scopes.get(actor)
    }
}
