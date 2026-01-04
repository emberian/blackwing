use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{CardId, FactionId, FlagId, ResourceId, SlotId, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    ModifyResource { resource: ResourceId, delta: i64 },

    SetResource { resource: ResourceId, value: i64 },

    SetFlag { flag: FlagId, value: Value },

    AddCard { card_id: CardId },

    RemoveCards { pattern: SmolStr },

    ModifyFactionReputation { faction: FactionId, delta: i64 },

    AddChronicle { title: SmolStr, text: SmolStr },

    Damage { resource: ResourceId, amount: i64 },

    Script { source: SmolStr },

    EquipModule { slot_id: SlotId, card_id: CardId },

    UnequipModule { slot_id: SlotId },

    ModifyStat { key: SmolStr, delta: i64 },

    Compound { effects: Vec<Effect> },
}

impl Effect {
    pub fn modify_resource(resource: impl Into<ResourceId>, delta: i64) -> Self {
        Effect::ModifyResource {
            resource: resource.into(),
            delta,
        }
    }

    pub fn set_resource(resource: impl Into<ResourceId>, value: i64) -> Self {
        Effect::SetResource {
            resource: resource.into(),
            value,
        }
    }

    pub fn set_flag(flag: impl Into<FlagId>, value: impl Into<Value>) -> Self {
        Effect::SetFlag {
            flag: flag.into(),
            value: value.into(),
        }
    }

    pub fn add_card(card_id: impl Into<CardId>) -> Self {
        Effect::AddCard {
            card_id: card_id.into(),
        }
    }

    pub fn remove_cards(pattern: impl Into<SmolStr>) -> Self {
        Effect::RemoveCards {
            pattern: pattern.into(),
        }
    }

    pub fn modify_reputation(faction: impl Into<FactionId>, delta: i64) -> Self {
        Effect::ModifyFactionReputation {
            faction: faction.into(),
            delta,
        }
    }

    pub fn chronicle(title: impl Into<SmolStr>, text: impl Into<SmolStr>) -> Self {
        Effect::AddChronicle {
            title: title.into(),
            text: text.into(),
        }
    }

    pub fn damage(resource: impl Into<ResourceId>, amount: i64) -> Self {
        Effect::Damage {
            resource: resource.into(),
            amount,
        }
    }

    pub fn script(source: impl Into<SmolStr>) -> Self {
        Effect::Script {
            source: source.into(),
        }
    }

    pub fn equip_module(slot_id: impl Into<SlotId>, card_id: impl Into<CardId>) -> Self {
        Effect::EquipModule {
            slot_id: slot_id.into(),
            card_id: card_id.into(),
        }
    }

    pub fn unequip_module(slot_id: impl Into<SlotId>) -> Self {
        Effect::UnequipModule {
            slot_id: slot_id.into(),
        }
    }

    pub fn modify_stat(key: impl Into<SmolStr>, delta: i64) -> Self {
        Effect::ModifyStat {
            key: key.into(),
            delta,
        }
    }

    pub fn compound(effects: Vec<Effect>) -> Self {
        Effect::Compound { effects }
    }

    pub fn flatten(&self) -> Vec<&Effect> {
        match self {
            Effect::Compound { effects } => effects.iter().flat_map(|e| e.flatten()).collect(),
            other => vec![other],
        }
    }
}
