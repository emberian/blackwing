use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{
    CardId, CardInstanceId, ChronicleEntry, FactionId, FlagId, LocationId, ResourceId, SceneId,
    SlotId, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    ResourceChanged {
        resource: ResourceId,
        old_value: i64,
        new_value: i64,
        reason: SmolStr,
    },

    FlagSet {
        flag: FlagId,
        old_value: Option<Value>,
        new_value: Value,
    },

    CardAdded {
        card_id: CardId,
        instance_id: CardInstanceId,
    },

    CardRemoved {
        instance_id: CardInstanceId,
        reason: SmolStr,
    },

    CardMovedToDeck {
        instance_id: CardInstanceId,
    },

    CardMovedToCollection {
        instance_id: CardInstanceId,
    },

    LocationChanged {
        from: Option<LocationId>,
        to: LocationId,
    },

    FactionReputationChanged {
        faction: FactionId,
        old_value: i64,
        new_value: i64,
    },

    SceneStarted {
        scene_id: SceneId,
    },

    PassageEntered {
        scene_id: SceneId,
        passage_index: usize,
    },

    ChoiceMade {
        scene_id: SceneId,
        passage_index: usize,
        choice_index: usize,
    },

    SceneEnded {
        scene_id: SceneId,
    },

    SceneCooldownSet {
        scene_id: SceneId,
        until_cycle: u64,
    },

    CycleAdvanced {
        new_cycle: u64,
    },

    GameOver {
        reason: GameOverReason,
    },

    ChronicleAdded {
        entry: ChronicleEntry,
    },

    ModuleEquipped {
        slot_id: SlotId,
        instance_id: CardInstanceId,
    },

    ModuleUnequipped {
        slot_id: SlotId,
        instance_id: CardInstanceId,
    },

    LocationStateChanged {
        location_id: LocationId,
        status: Option<SmolStr>,
        last_visited: Option<u64>,
    },

    StatChanged {
        key: SmolStr,
        new_value: i64,
    },

    RngStateAdvanced {
        new_state: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameOverReason {
    ResourceDepleted {
        resource: ResourceId,
        message: String,
    },
    Victory {
        message: String,
    },
    Custom {
        message: String,
    },
}

impl Event {
    pub fn resource_changed(
        resource: ResourceId,
        old_value: i64,
        new_value: i64,
        reason: impl Into<SmolStr>,
    ) -> Self {
        Event::ResourceChanged {
            resource,
            old_value,
            new_value,
            reason: reason.into(),
        }
    }

    pub fn flag_set(flag: FlagId, old_value: Option<Value>, new_value: Value) -> Self {
        Event::FlagSet {
            flag,
            old_value,
            new_value,
        }
    }

    pub fn card_added(card_id: CardId, instance_id: CardInstanceId) -> Self {
        Event::CardAdded {
            card_id,
            instance_id,
        }
    }

    pub fn card_removed(instance_id: CardInstanceId, reason: impl Into<SmolStr>) -> Self {
        Event::CardRemoved {
            instance_id,
            reason: reason.into(),
        }
    }

    pub fn location_changed(from: Option<LocationId>, to: LocationId) -> Self {
        Event::LocationChanged { from, to }
    }

    pub fn scene_started(scene_id: SceneId) -> Self {
        Event::SceneStarted { scene_id }
    }

    pub fn scene_ended(scene_id: SceneId) -> Self {
        Event::SceneEnded { scene_id }
    }

    pub fn cycle_advanced(new_cycle: u64) -> Self {
        Event::CycleAdvanced { new_cycle }
    }

    pub fn game_over(reason: GameOverReason) -> Self {
        Event::GameOver { reason }
    }
}
