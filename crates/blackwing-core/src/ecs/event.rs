//! World events for tracking all state changes.
//!
//! All mutations to WorldState are represented as events, enabling:
//! - Deterministic replay
//! - Undo/redo
//! - Networking (sync via events)
//! - Save/load (replay events from checkpoint)

use crate::{EntityId, FlagId, ResourceId, Value};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use super::{EntityKey, Scope, ScopeKind};

/// All possible world state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldEvent {
    // === Entity Events ===
    /// An entity was spawned
    EntitySpawned {
        key: EntityKey,
        id: EntityId,
    },

    /// An entity was despawned
    EntityDespawned {
        key: EntityKey,
        id: EntityId,
    },

    /// A component was set on an entity
    ComponentSet {
        entity: EntityKey,
        component: SmolStr,
        old_value: Option<Value>,
        new_value: Value,
    },

    /// A component was removed from an entity
    ComponentRemoved {
        entity: EntityKey,
        component: SmolStr,
        old_value: Value,
    },

    /// A tag was added to an entity
    TagAdded {
        entity: EntityKey,
        tag: SmolStr,
    },

    /// A tag was removed from an entity
    TagRemoved {
        entity: EntityKey,
        tag: SmolStr,
    },

    // === Global State Events ===
    /// A global resource changed
    ResourceChanged {
        resource: ResourceId,
        old_value: i64,
        new_value: i64,
        reason: SmolStr,
    },

    /// A global flag changed
    FlagSet {
        flag: FlagId,
        old_value: Option<Value>,
        new_value: Value,
    },

    // === Scope Events ===
    /// A scope was pushed (entered a new context)
    ScopePushed {
        actor: EntityId,
        scope: Scope,
    },

    /// A scope was popped (exited a context)
    ScopePopped {
        actor: EntityId,
        scope: Scope,
    },

    /// A scope's local state changed
    ScopeStateChanged {
        actor: EntityId,
        scope_kind: ScopeKind,
        key: SmolStr,
        old_value: Option<Value>,
        new_value: Value,
    },

    // === Time Events ===
    /// Game time advanced
    TimeAdvanced {
        old_tick: u64,
        new_tick: u64,
    },

    /// RNG state advanced
    RngAdvanced {
        new_state: u64,
    },

    // === Chronicle Events ===
    /// A chronicle entry was added
    ChronicleAdded {
        entry: ChronicleEntry,
    },

    // === UI Events ===
    /// The root UI was set
    UiSet {
        /// Serialized UiNode (for now, full type support can be added later)
        node: String,
    },

    /// A UI node was updated
    UiUpdated {
        path: String,
        node: String,
    },

    /// A modal was opened
    ModalOpened {
        content: String,
        blocking: bool,
    },

    /// The modal was closed
    ModalClosed,

    /// The UI was cleared
    UiCleared,

    // === Spatial Events (for tilemap-based games) ===

    /// An entity's position was set directly
    PositionSet {
        entity: EntityKey,
        x: f32,
        y: f32,
    },

    /// An entity moved from one position to another
    EntityMoved {
        entity: EntityKey,
        from_x: f32,
        from_y: f32,
        to_x: f32,
        to_y: f32,
    },

    /// The current room changed (screen transition)
    RoomChanged {
        room_id: SmolStr,
        spawn_x: f32,
        spawn_y: f32,
    },

    // === Combat Events ===

    /// An entity took damage
    DamageTaken {
        entity: EntityKey,
        amount: i64,
        source: Option<EntityId>,
        health_remaining: i64,
    },

    /// An entity died
    EntityDied {
        entity: EntityKey,
        killer: Option<EntityId>,
    },

    /// An entity was healed
    Healed {
        entity: EntityKey,
        amount: i64,
        health_now: i64,
    },

    /// Invincibility was set on an entity
    InvincibilitySet {
        entity: EntityKey,
        duration_ms: u32,
    },
}

impl WorldEvent {
    // Factory methods for convenience

    pub fn entity_spawned(key: EntityKey, id: EntityId) -> Self {
        WorldEvent::EntitySpawned { key, id }
    }

    pub fn entity_despawned(key: EntityKey, id: EntityId) -> Self {
        WorldEvent::EntityDespawned { key, id }
    }

    pub fn component_set(
        entity: EntityKey,
        component: impl Into<SmolStr>,
        old_value: Option<Value>,
        new_value: Value,
    ) -> Self {
        WorldEvent::ComponentSet {
            entity,
            component: component.into(),
            old_value,
            new_value,
        }
    }

    pub fn resource_changed(
        resource: ResourceId,
        old_value: i64,
        new_value: i64,
        reason: impl Into<SmolStr>,
    ) -> Self {
        WorldEvent::ResourceChanged {
            resource,
            old_value,
            new_value,
            reason: reason.into(),
        }
    }

    pub fn flag_set(flag: FlagId, old_value: Option<Value>, new_value: Value) -> Self {
        WorldEvent::FlagSet {
            flag,
            old_value,
            new_value,
        }
    }

    pub fn scope_pushed(actor: EntityId, scope: Scope) -> Self {
        WorldEvent::ScopePushed { actor, scope }
    }

    pub fn scope_popped(actor: EntityId, scope: Scope) -> Self {
        WorldEvent::ScopePopped { actor, scope }
    }

    pub fn time_advanced(old_tick: u64, new_tick: u64) -> Self {
        WorldEvent::TimeAdvanced { old_tick, new_tick }
    }

    pub fn rng_advanced(new_state: u64) -> Self {
        WorldEvent::RngAdvanced { new_state }
    }
}

/// A journal/chronicle entry recording game history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleEntry {
    pub id: SmolStr,
    pub entry_type: SmolStr,
    pub tick: u64,
    pub title: SmolStr,
    pub text: SmolStr,
    pub location: Option<EntityId>,
    pub entities: Vec<EntityId>,
}

impl ChronicleEntry {
    pub fn new(
        id: impl Into<SmolStr>,
        entry_type: impl Into<SmolStr>,
        tick: u64,
        title: impl Into<SmolStr>,
        text: impl Into<SmolStr>,
    ) -> Self {
        Self {
            id: id.into(),
            entry_type: entry_type.into(),
            tick,
            title: title.into(),
            text: text.into(),
            location: None,
            entities: Vec::new(),
        }
    }

    pub fn with_location(mut self, location: EntityId) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_entity(mut self, entity: EntityId) -> Self {
        self.entities.push(entity);
        self
    }
}
