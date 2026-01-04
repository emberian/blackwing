use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{CardId, ContextId, FactionId, LocationId, ResourceId, SceneId, SlotId, Tags};

/// A scene (event/scenelet) in the game.
///
/// All conditions and effects are expressed as Rhai scripts.
/// The DSL syntax is pure sugar that compiles to Rhai.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: SceneId,
    pub title: SmolStr,
    pub tags: Tags,
    pub context: Option<ContextId>,
    pub weight: u32,
    pub cooldown: u64,
    pub passages: Vec<Passage>,
    /// Rhai script for scene requirements. Empty string means always available.
    #[serde(default)]
    pub rhai_requirements: SmolStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passage {
    pub text: SmolStr,
    pub choices: Vec<Choice>,
    /// Rhai source for passage-level effects (executed on passage entry).
    /// Generated from inline `{ }` Rhai blocks in the DSL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rhai_on_enter: Option<SmolStr>,
}

/// A choice within a passage.
///
/// Conditions and effects are Rhai scripts. Empty string means no condition/no effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub text: SmolStr,
    pub next: Navigation,
    /// Rhai script for the condition. Empty string means always available.
    #[serde(default)]
    pub rhai_condition: SmolStr,
    /// Rhai script for effects. Empty string means no effects.
    #[serde(default)]
    pub rhai_effects: SmolStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Navigation {
    Passage(usize),
    End,
}

impl Default for Navigation {
    fn default() -> Self {
        Navigation::End
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDef {
    pub id: CardId,
    pub name: SmolStr,
    pub description: SmolStr,
    pub card_type: SmolStr,
    pub tags: Tags,
    pub rarity: Rarity,
    pub base_value: Option<i64>,
    pub effects: CardEffects,
    #[serde(default)]
    pub contract_terms: Option<ContractTerms>,
    #[serde(default)]
    pub journey_behavior: Option<JourneyBehavior>,
    #[serde(default)]
    pub install_requirements: Option<InstallRequirements>,
    #[serde(default)]
    pub upgrades_to: Option<CardId>,
    #[serde(default)]
    pub upgrade_cost: Option<IndexMap<ResourceId, i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTerms {
    pub destination: LocationId,
    #[serde(default)]
    pub cargo_required: Option<CargoRequirement>,
    pub cycle_limit: u32,
    pub reward: IndexMap<ResourceId, i64>,
    #[serde(default)]
    pub penalty: Option<IndexMap<ResourceId, i64>>,
    #[serde(default)]
    pub reputation_reward: Option<ReputationChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoRequirement {
    pub card_id: CardId,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChange {
    pub faction: FactionId,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyBehavior {
    #[serde(default)]
    pub decay_chance: f64,
    #[serde(default)]
    pub event_chance: f64,
    #[serde(default)]
    pub event_pool: Vec<SceneId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRequirements {
    #[serde(default)]
    pub min_resources: IndexMap<ResourceId, i64>,
    #[serde(default)]
    pub required_modules: Vec<CardId>,
    #[serde(default)]
    pub excludes_modules: Vec<CardId>,
    pub slot_type: SlotId,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardEffects {
    #[serde(default)]
    pub modifiers: IndexMap<SmolStr, i64>,
    #[serde(default)]
    pub grants_tags: Tags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl Default for Rarity {
    fn default() -> Self {
        Rarity::Common
    }
}

pub trait ContentRegistry: Send + Sync {
    fn get_scene(&self, id: &SceneId) -> Option<&Scene>;
    fn get_card(&self, id: &CardId) -> Option<&CardDef>;
    fn scenes(&self) -> Box<dyn Iterator<Item = &Scene> + '_>;
    fn cards(&self) -> Box<dyn Iterator<Item = &CardDef> + '_>;
    fn scene_count(&self) -> usize;
    fn card_count(&self) -> usize;
}

impl Scene {
    pub fn entry_passage(&self) -> Option<&Passage> {
        self.passages.first()
    }

    pub fn passage(&self, index: usize) -> Option<&Passage> {
        self.passages.get(index)
    }

    pub fn is_terminal(&self, passage_index: usize) -> bool {
        self.passages
            .get(passage_index)
            .map(|p| {
                p.choices.is_empty() || p.choices.iter().all(|c| matches!(c.next, Navigation::End))
            })
            .unwrap_or(true)
    }
}

impl Passage {
    pub fn is_terminal(&self) -> bool {
        self.choices.is_empty()
    }
}
