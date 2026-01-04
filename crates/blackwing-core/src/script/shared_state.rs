//! Shared state for collecting effects during script execution.

use std::sync::{Arc, Mutex};

use crate::ui::UiNode;
use crate::{EntityId, FlagId, ResourceId, ScopeKind, Value};
use smol_str::SmolStr;

/// Effects that can be collected during script execution.
/// These are converted to system Effects by the ScriptSystem.
#[derive(Debug, Clone)]
pub enum ScriptEffect {
    // Entity effects
    SpawnEntity {
        kind: SmolStr,
        id: SmolStr,
    },
    DespawnEntity {
        entity_id: EntityId,
    },
    SetComponent {
        entity_id: EntityId,
        component: SmolStr,
        value: Value,
    },
    RemoveComponent {
        entity_id: EntityId,
        component: SmolStr,
    },
    AddTag {
        entity_id: EntityId,
        tag: SmolStr,
    },
    RemoveTag {
        entity_id: EntityId,
        tag: SmolStr,
    },

    // Resource effects
    SetResource {
        resource: ResourceId,
        value: i64,
    },
    ModifyResource {
        resource: ResourceId,
        delta: i64,
    },

    // Flag effects
    SetFlag {
        flag: FlagId,
        value: Value,
    },
    ClearFlag {
        flag: FlagId,
    },

    // Scope effects
    PushScope {
        actor: EntityId,
        kind: ScopeKind,
        id: SmolStr,
    },
    PopScope {
        actor: EntityId,
    },

    // Chronicle
    Chronicle {
        title: SmolStr,
        description: SmolStr,
    },

    // Time
    AdvanceTime {
        ticks: u64,
    },

    // UI effects
    SetUiRoot {
        root: UiNode,
    },
    ShowModal {
        content: UiNode,
        blocking: bool,
    },
    CloseModal,
    ClearUi,
}

/// Shared state accumulated during script execution.
/// Uses Arc<Mutex<_>> to be Send+Sync for Rhai's sync feature.
#[derive(Clone)]
pub struct ScriptState {
    effects: Arc<Mutex<Vec<ScriptEffect>>>,
    rng_state: Arc<Mutex<u64>>,
    actor: Arc<Mutex<Option<EntityId>>>,
}

impl ScriptState {
    /// Create new script state with the given RNG seed
    pub fn new(rng_seed: u64) -> Self {
        Self {
            effects: Arc::new(Mutex::new(Vec::new())),
            rng_state: Arc::new(Mutex::new(rng_seed)),
            actor: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the current actor (for scope effects)
    pub fn set_actor(&self, actor: EntityId) {
        *self.actor.lock().unwrap() = Some(actor);
    }

    /// Get the current actor
    pub fn actor(&self) -> Option<EntityId> {
        self.actor.lock().unwrap().clone()
    }

    /// Push an effect
    pub fn push_effect(&self, effect: ScriptEffect) {
        self.effects.lock().unwrap().push(effect);
    }

    /// Get the RNG state reference for registering functions
    pub fn rng_state(&self) -> Arc<Mutex<u64>> {
        self.rng_state.clone()
    }

    /// Get the effects reference for registering functions
    pub fn effects(&self) -> Arc<Mutex<Vec<ScriptEffect>>> {
        self.effects.clone()
    }

    /// Consume and return all collected effects
    pub fn into_effects(self) -> Vec<ScriptEffect> {
        Arc::try_unwrap(self.effects)
            .map(|m| m.into_inner().unwrap())
            .unwrap_or_else(|arc| arc.lock().unwrap().clone())
    }

    /// Get the final RNG state
    pub fn final_rng_state(&self) -> u64 {
        *self.rng_state.lock().unwrap()
    }
}
