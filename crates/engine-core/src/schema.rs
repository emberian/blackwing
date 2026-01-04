use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    CardTypeId, ContextId, FactionId, LocationId, ResourceId, SlotId, TagCategoryId, TagId, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSchema {
    pub name: String,
    pub version: String,
    pub resources: Vec<ResourceDef>,
    pub tag_categories: Vec<TagCategory>,
    pub card_types: Vec<CardTypeDef>,
    pub contexts: Vec<ContextDef>,
    #[serde(default)]
    pub slot_types: Vec<SlotTypeDef>,
    #[serde(default)]
    pub location_statuses: Vec<String>,
    pub initial_state: InitialStateDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub id: ResourceId,
    pub name: String,
    #[serde(default)]
    pub min: Option<i64>,
    #[serde(default)]
    pub max: Option<i64>,
    pub default: i64,
    #[serde(default)]
    pub on_zero: Option<OnZeroBehavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnZeroBehavior {
    GameOver { reason: String },
    Clamp,
    AllowNegative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCategory {
    pub id: TagCategoryId,
    pub name: String,
    pub tags: Vec<TagId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardTypeDef {
    pub id: CardTypeId,
    pub name: String,
    #[serde(default)]
    pub has_condition: bool,
    #[serde(default)]
    pub has_cycles_remaining: bool,
    #[serde(default)]
    pub slot_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDef {
    pub id: ContextId,
    pub name: String,
}

/// Definition of a module slot type (e.g., sensor, defense, cargo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotTypeDef {
    pub id: SlotId,
    pub name: String,
    /// How many slots of this type exist by default
    #[serde(default = "default_slot_count")]
    pub count: u32,
}

fn default_slot_count() -> u32 {
    1
}

/// Static definition of a location/port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationDef {
    pub id: LocationId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Tags,
    #[serde(default)]
    pub faction: Option<FactionId>,
    /// Default market price modifiers for cards at this location
    #[serde(default)]
    pub base_market_modifiers: IndexMap<String, f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InitialStateDef {
    #[serde(default)]
    pub resources: IndexMap<ResourceId, i64>,
    #[serde(default)]
    pub flags: IndexMap<String, Value>,
}

impl GameSchema {
    pub fn resource(&self, id: &ResourceId) -> Option<&ResourceDef> {
        self.resources.iter().find(|r| &r.id == id)
    }

    pub fn tag_category(&self, id: &TagCategoryId) -> Option<&TagCategory> {
        self.tag_categories.iter().find(|c| &c.id == id)
    }

    pub fn is_valid_tag(&self, category: &TagCategoryId, tag: &TagId) -> bool {
        self.tag_category(category)
            .map(|c| c.tags.contains(tag))
            .unwrap_or(false)
    }

    pub fn context(&self, id: &ContextId) -> Option<&ContextDef> {
        self.contexts.iter().find(|c| &c.id == id)
    }

    pub fn slot_type(&self, id: &SlotId) -> Option<&SlotTypeDef> {
        self.slot_types.iter().find(|s| &s.id == id)
    }

    pub fn resources_with_game_over(&self) -> impl Iterator<Item = &ResourceDef> {
        self.resources
            .iter()
            .filter(|r| matches!(r.on_zero, Some(OnZeroBehavior::GameOver { .. })))
    }
}

pub type Tags = SmallVec<[TagId; 4]>;
