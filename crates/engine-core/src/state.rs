use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{
    CardId, CardInstanceId, ChronicleEntryId, Event, FactionId, FlagId, GameOverReason, GameSchema,
    LocationId, OnZeroBehavior, ResourceId, SceneId, SlotId, Tags, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub cycle: u64,
    pub resources: IndexMap<ResourceId, i64>,
    pub flags: FxHashMap<FlagId, Value>,
    pub cards: CardsState,
    pub location: Option<LocationId>,
    pub locations: FxHashMap<LocationId, LocationState>,
    pub factions: FxHashMap<FactionId, i64>,
    pub scene_cooldowns: FxHashMap<SceneId, u64>,
    pub equipped_modules: IndexMap<SlotId, CardInstanceId>,
    pub chronicle: Vec<ChronicleEntry>,
    pub stats: FxHashMap<SmolStr, i64>,
    pub rng_state: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardsState {
    pub instances: FxHashMap<CardInstanceId, CardInstance>,
    pub collection: Vec<CardInstanceId>,
    pub deck: Vec<CardInstanceId>,
    pub active_crew: Vec<CardInstanceId>,
    pub active_contracts: Vec<CardInstanceId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInstance {
    pub instance_id: CardInstanceId,
    pub card_id: CardId,
    pub level: u32,
    pub condition: u32,
    pub acquired_at: u64,
    pub cycles_remaining: Option<u32>,
    pub custom_data: FxHashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationState {
    pub status: SmolStr,
    pub last_visited: Option<u64>,
    pub market_modifiers: FxHashMap<CardId, f64>,
    pub available_cards: Vec<CardId>,
    pub available_contracts: Vec<CardId>,
    pub market_refreshed_at: Option<u64>,
}

impl Default for LocationState {
    fn default() -> Self {
        Self {
            status: SmolStr::new_static("unknown"),
            last_visited: None,
            market_modifiers: FxHashMap::default(),
            available_cards: Vec::new(),
            available_contracts: Vec::new(),
            market_refreshed_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleEntry {
    pub id: ChronicleEntryId,
    pub entry_type: SmolStr,
    pub cycle: u64,
    pub title: SmolStr,
    pub text: SmolStr,
    pub tags: Tags,
    pub location_ref: Option<LocationId>,
    pub card_refs: Vec<CardInstanceId>,
    pub faction_ref: Option<FactionId>,
}

impl GameState {
    pub fn resource(&self, id: &ResourceId) -> i64 {
        self.resources.get(id).copied().unwrap_or(0)
    }

    pub fn set_resource(&mut self, id: ResourceId, value: i64) {
        self.resources.insert(id, value);
    }

    pub fn modify_resource(&mut self, id: &ResourceId, delta: i64) {
        let current = self.resource(id);
        self.resources.insert(id.clone(), current + delta);
    }

    pub fn flag(&self, id: &FlagId) -> &Value {
        static NULL: Value = Value::Null;
        self.flags.get(id).unwrap_or(&NULL)
    }

    pub fn flag_is_truthy(&self, id: &FlagId) -> bool {
        self.flag(id).is_truthy()
    }

    pub fn set_flag(&mut self, id: FlagId, value: Value) {
        self.flags.insert(id, value);
    }

    pub fn faction_reputation(&self, id: &FactionId) -> i64 {
        self.factions.get(id).copied().unwrap_or(0)
    }

    pub fn modify_faction_reputation(&mut self, id: &FactionId, delta: i64) {
        let current = self.faction_reputation(id);
        let new_val = (current + delta).clamp(-100, 100);
        self.factions.insert(id.clone(), new_val);
    }

    pub fn is_scene_on_cooldown(&self, id: &SceneId, cooldown: u64) -> bool {
        if cooldown == 0 {
            return false;
        }
        self.scene_cooldowns
            .get(id)
            .map(|&until| self.cycle < until)
            .unwrap_or(false)
    }

    pub fn set_scene_cooldown(&mut self, id: SceneId, cooldown: u64) {
        if cooldown > 0 {
            self.scene_cooldowns.insert(id, self.cycle + cooldown);
        }
    }

    pub fn card_instance(&self, id: &CardInstanceId) -> Option<&CardInstance> {
        self.cards.instances.get(id)
    }

    pub fn deck_card_ids(&self) -> impl Iterator<Item = &CardId> {
        self.cards
            .deck
            .iter()
            .filter_map(|inst_id| self.cards.instances.get(inst_id))
            .map(|inst| &inst.card_id)
    }

    pub fn equipped_module(&self, slot: &SlotId) -> Option<&CardInstanceId> {
        self.equipped_modules.get(slot)
    }

    pub fn equipped_module_card_ids(&self) -> impl Iterator<Item = &CardId> + '_ {
        self.equipped_modules
            .values()
            .filter_map(|inst_id| self.cards.instances.get(inst_id))
            .map(|inst| &inst.card_id)
    }

    pub fn location_state(&self, id: &LocationId) -> Option<&LocationState> {
        self.locations.get(id)
    }

    pub fn location_state_mut(&mut self, id: &LocationId) -> &mut LocationState {
        self.locations
            .entry(id.clone())
            .or_insert_with(LocationState::default)
    }

    pub fn stat(&self, key: &str) -> i64 {
        self.stats.get(key).copied().unwrap_or(0)
    }

    pub fn increment_stat(&mut self, key: impl Into<SmolStr>, delta: i64) {
        let key = key.into();
        let current = self.stats.get(&key).copied().unwrap_or(0);
        self.stats.insert(key, current + delta);
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            cycle: 0,
            resources: IndexMap::new(),
            flags: FxHashMap::default(),
            cards: CardsState::default(),
            location: None,
            locations: FxHashMap::default(),
            factions: FxHashMap::default(),
            scene_cooldowns: FxHashMap::default(),
            equipped_modules: IndexMap::new(),
            chronicle: Vec::new(),
            stats: FxHashMap::default(),
            rng_state: 0,
        }
    }
}

impl GameState {
    pub fn from_schema(schema: &GameSchema, rng_seed: u64) -> Self {
        let mut state = Self {
            cycle: 0,
            resources: IndexMap::new(),
            flags: FxHashMap::default(),
            cards: CardsState::default(),
            location: None,
            locations: FxHashMap::default(),
            factions: FxHashMap::default(),
            scene_cooldowns: FxHashMap::default(),
            equipped_modules: IndexMap::new(),
            chronicle: Vec::new(),
            stats: FxHashMap::default(),
            rng_state: rng_seed,
        };

        for resource_def in &schema.resources {
            state
                .resources
                .insert(resource_def.id.clone(), resource_def.default);
        }

        for (flag_id, value) in &schema.initial_state.flags {
            state.flags.insert(FlagId::new(flag_id), value.clone());
        }

        for (resource_id, value) in &schema.initial_state.resources {
            state.resources.insert(resource_id.clone(), *value);
        }

        state
    }

    pub fn apply_event(&mut self, event: &Event, schema: &GameSchema) -> Option<GameOverReason> {
        match event {
            Event::ResourceChanged {
                resource,
                new_value,
                ..
            } => {
                let clamped = self.clamp_resource(schema, resource, *new_value);
                self.resources.insert(resource.clone(), clamped);

                if let Some(def) = schema.resource(resource) {
                    if clamped <= def.min.unwrap_or(i64::MIN) {
                        if let Some(OnZeroBehavior::GameOver { reason }) = &def.on_zero {
                            return Some(GameOverReason::ResourceDepleted {
                                resource: resource.clone(),
                                message: reason.clone(),
                            });
                        }
                    }
                }
            }

            Event::FlagSet {
                flag, new_value, ..
            } => {
                self.flags.insert(flag.clone(), new_value.clone());
            }

            Event::CardAdded {
                card_id,
                instance_id,
            } => {
                let instance = CardInstance {
                    instance_id: instance_id.clone(),
                    card_id: card_id.clone(),
                    level: 1,
                    condition: 100,
                    acquired_at: self.cycle,
                    cycles_remaining: None,
                    custom_data: FxHashMap::default(),
                };
                self.cards.instances.insert(instance_id.clone(), instance);
                self.cards.collection.push(instance_id.clone());
            }

            Event::CardRemoved { instance_id, .. } => {
                self.cards.instances.remove(instance_id);
                self.cards.collection.retain(|id| id != instance_id);
                self.cards.deck.retain(|id| id != instance_id);
                self.cards.active_crew.retain(|id| id != instance_id);
                self.cards.active_contracts.retain(|id| id != instance_id);
            }

            Event::CardMovedToDeck { instance_id } => {
                self.cards.collection.retain(|id| id != instance_id);
                if !self.cards.deck.contains(instance_id) {
                    self.cards.deck.push(instance_id.clone());
                }
            }

            Event::CardMovedToCollection { instance_id } => {
                self.cards.deck.retain(|id| id != instance_id);
                self.cards.active_crew.retain(|id| id != instance_id);
                if !self.cards.collection.contains(instance_id) {
                    self.cards.collection.push(instance_id.clone());
                }
            }

            Event::LocationChanged { to, .. } => {
                self.location = Some(to.clone());
            }

            Event::FactionReputationChanged {
                faction, new_value, ..
            } => {
                self.factions.insert(faction.clone(), *new_value);
            }

            Event::SceneCooldownSet {
                scene_id,
                until_cycle,
            } => {
                self.scene_cooldowns.insert(scene_id.clone(), *until_cycle);
            }

            Event::CycleAdvanced { new_cycle } => {
                self.cycle = *new_cycle;
            }

            Event::RngStateAdvanced { new_state } => {
                self.rng_state = *new_state;
            }

            Event::GameOver { reason } => {
                return Some(reason.clone());
            }

            Event::SceneStarted { .. }
            | Event::PassageEntered { .. }
            | Event::ChoiceMade { .. }
            | Event::SceneEnded { .. } => {}

            Event::ChronicleAdded { entry } => {
                self.chronicle.push(entry.clone());
            }

            Event::ModuleEquipped {
                slot_id,
                instance_id,
            } => {
                self.equipped_modules
                    .insert(slot_id.clone(), instance_id.clone());
            }

            Event::ModuleUnequipped { slot_id, .. } => {
                self.equipped_modules.shift_remove(slot_id);
            }

            Event::LocationStateChanged {
                location_id,
                status,
                last_visited,
            } => {
                let loc = self.location_state_mut(location_id);
                if let Some(s) = status {
                    loc.status = s.clone();
                }
                if let Some(v) = last_visited {
                    loc.last_visited = Some(*v);
                }
            }

            Event::StatChanged { key, new_value } => {
                self.stats.insert(key.clone(), *new_value);
            }
        }

        None
    }

    fn clamp_resource(&self, schema: &GameSchema, resource: &ResourceId, value: i64) -> i64 {
        if let Some(def) = schema.resource(resource) {
            let min = def.min.unwrap_or(i64::MIN);
            let max = def.max.unwrap_or(i64::MAX);
            value.clamp(min, max)
        } else {
            value
        }
    }
}
