//! Effects represent changes that systems want to make to the world.
//!
//! Systems don't modify world state directly - they return effects that the
//! runtime applies. This enables validation, logging, and deterministic replay.

use crate::ui::UiNode;
use crate::{EntityId, FlagId, ResourceId, ScopeKind, Value};
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

    // === Spatial effects (for tilemap-based games) ===

    /// Set an entity's position directly
    SetPosition {
        entity: EntityId,
        x: f32,
        y: f32,
    },
    /// Move an entity by a delta (with collision resolution handled by runtime)
    MoveBy {
        entity: EntityId,
        dx: f32,
        dy: f32,
    },
    /// Change the current room/screen
    ChangeRoom {
        room_id: SmolStr,
        /// Spawn X position in the new room
        spawn_x: f32,
        /// Spawn Y position in the new room
        spawn_y: f32,
    },
    /// Spawn an entity from template at a specific position in current room
    SpawnAtPosition {
        template_id: EntityId,
        x: f32,
        y: f32,
    },

    // UI effects
    /// Set the root UI node (replaces current UI)
    SetUiRoot {
        root: UiNode,
    },
    /// Update a specific UI node by path
    UpdateUi {
        path: SmolStr,
        node: UiNode,
    },
    /// Show a modal dialog
    ShowModal {
        content: UiNode,
        /// Whether the modal blocks interaction with the rest of the UI
        blocking: bool,
    },
    /// Close the current modal
    CloseModal,
    /// Clear the UI (set to empty)
    ClearUi,

    // === Combat effects ===

    /// Deal damage to an entity (checks invincibility, triggers events)
    DealDamage {
        target: EntityId,
        amount: i64,
        /// Source of damage (for knockback direction, etc.)
        source: Option<EntityId>,
    },
    /// Heal an entity
    Heal {
        target: EntityId,
        amount: i64,
    },
    /// Set entity invincibility for a duration
    SetInvincible {
        entity: EntityId,
        duration_ms: u32,
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

    /// Create a set UI root effect
    pub fn set_ui(root: UiNode) -> Self {
        Self::SetUiRoot { root }
    }

    /// Create a show modal effect
    pub fn show_modal(content: UiNode, blocking: bool) -> Self {
        Self::ShowModal { content, blocking }
    }

    /// Create a close modal effect
    pub fn close_modal() -> Self {
        Self::CloseModal
    }

    /// Create a clear UI effect
    pub fn clear_ui() -> Self {
        Self::ClearUi
    }

    // === Spatial effect constructors ===

    /// Create a set position effect
    pub fn set_position(entity: EntityId, x: f32, y: f32) -> Self {
        Self::SetPosition { entity, x, y }
    }

    /// Create a move by effect
    pub fn move_by(entity: EntityId, dx: f32, dy: f32) -> Self {
        Self::MoveBy { entity, dx, dy }
    }

    /// Create a change room effect
    pub fn change_room(room_id: impl Into<SmolStr>, spawn_x: f32, spawn_y: f32) -> Self {
        Self::ChangeRoom {
            room_id: room_id.into(),
            spawn_x,
            spawn_y,
        }
    }

    /// Create a spawn at position effect
    pub fn spawn_at_position(template_id: EntityId, x: f32, y: f32) -> Self {
        Self::SpawnAtPosition { template_id, x, y }
    }

    // === Combat effect constructors ===

    /// Create a deal damage effect
    pub fn deal_damage(target: EntityId, amount: i64, source: Option<EntityId>) -> Self {
        Self::DealDamage {
            target,
            amount,
            source,
        }
    }

    /// Create a heal effect
    pub fn heal(target: EntityId, amount: i64) -> Self {
        Self::Heal { target, amount }
    }

    /// Create a set invincible effect
    pub fn set_invincible(entity: EntityId, duration_ms: u32) -> Self {
        Self::SetInvincible { entity, duration_ms }
    }
}
