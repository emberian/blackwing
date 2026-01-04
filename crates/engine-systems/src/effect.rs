//! Effects represent changes that systems want to make to the world.
//!
//! Systems don't modify world state directly - they return effects that the
//! runtime applies. This enables validation, logging, and deterministic replay.

use engine_primitives::{EntityId, FlagId, ResourceId, Value};
use engine_world::ScopeKind;
use smol_str::SmolStr;

/// An effect that a system wants to apply to the world.
#[derive(Debug, Clone)]
pub enum Effect {
    // Entity effects
    SpawnEntity {
        id: EntityId,
    },
    DespawnEntity {
        id: EntityId,
    },
    SetComponent {
        entity: EntityId,
        component: SmolStr,
        value: Value,
    },
    RemoveComponent {
        entity: EntityId,
        component: SmolStr,
    },
    AddTag {
        entity: EntityId,
        tag: SmolStr,
    },
    RemoveTag {
        entity: EntityId,
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
    SetScopeLocal {
        actor: EntityId,
        key: SmolStr,
        value: Value,
    },

    // Chronicle effects
    Chronicle {
        title: SmolStr,
        description: SmolStr,
    },

    // Time effects
    AdvanceTime {
        ticks: u64,
    },

    // Script execution
    RunScript {
        script: SmolStr,
    },

    // Composite effects
    Batch {
        effects: Vec<Effect>,
    },
}

impl Effect {
    /// Create a spawn entity effect
    pub fn spawn(id: EntityId) -> Self {
        Self::SpawnEntity { id }
    }

    /// Create a despawn entity effect
    pub fn despawn(id: EntityId) -> Self {
        Self::DespawnEntity { id }
    }

    /// Create a set component effect
    pub fn set_component(entity: EntityId, component: impl Into<SmolStr>, value: Value) -> Self {
        Self::SetComponent {
            entity,
            component: component.into(),
            value,
        }
    }

    /// Create a modify resource effect
    pub fn modify_resource(resource: ResourceId, delta: i64) -> Self {
        Self::ModifyResource { resource, delta }
    }

    /// Create a set flag effect
    pub fn set_flag(flag: FlagId, value: Value) -> Self {
        Self::SetFlag { flag, value }
    }

    /// Create a chronicle entry effect
    pub fn chronicle(title: impl Into<SmolStr>, description: impl Into<SmolStr>) -> Self {
        Self::Chronicle {
            title: title.into(),
            description: description.into(),
        }
    }

    /// Create a push scope effect
    pub fn push_scope(actor: EntityId, kind: ScopeKind, id: impl Into<SmolStr>) -> Self {
        Self::PushScope {
            actor,
            kind,
            id: id.into(),
        }
    }

    /// Create a pop scope effect
    pub fn pop_scope(actor: EntityId) -> Self {
        Self::PopScope { actor }
    }

    /// Create a batch of effects
    pub fn batch(effects: Vec<Effect>) -> Self {
        Self::Batch { effects }
    }
}
