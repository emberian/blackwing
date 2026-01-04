use engine_core::{
    CardInstanceId, ChronicleEntry, ChronicleEntryId, Command, Effect, Event, GameConfig,
    GameSchema, GameState, Navigation, ResourceId, SceneId, TagProvider, Tags,
};
use engine_script::ScriptExecutor;
use smol_str::SmolStr;

use crate::{ContentRegistry, DeckTagProvider, Rng, RuntimeError};

pub struct CommandHandler<'a> {
    schema: &'a GameSchema,
    registry: &'a dyn ContentRegistry,
    config: &'a GameConfig,
}

impl<'a> CommandHandler<'a> {
    pub fn new(
        schema: &'a GameSchema,
        registry: &'a dyn ContentRegistry,
        config: &'a GameConfig,
    ) -> Self {
        Self {
            schema,
            registry,
            config,
        }
    }

    pub fn handle(
        &self,
        state: &GameState,
        command: &Command,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        match command {
            Command::Travel { destination } => {
                let events = vec![Event::location_changed(
                    state.location.clone(),
                    destination.clone(),
                )];
                Ok(events)
            }

            Command::MakeChoice {
                scene_id,
                passage_index,
                choice_index,
            } => self.handle_choice(state, scene_id, *passage_index, *choice_index, rng),

            Command::ContinueScene { scene_id } => Ok(vec![Event::SceneEnded {
                scene_id: scene_id.clone(),
            }]),

            Command::TriggerScene { scene_id } => {
                let scene = self.registry.get_scene(scene_id).ok_or_else(|| {
                    RuntimeError::SceneNotFound {
                        scene_id: scene_id.clone(),
                    }
                })?;

                let mut events = vec![Event::scene_started(scene_id.clone())];

                if scene.cooldown > 0 {
                    events.push(Event::SceneCooldownSet {
                        scene_id: scene_id.clone(),
                        until_cycle: state.cycle + scene.cooldown,
                    });
                }

                Ok(events)
            }

            Command::TradeBuy { card_id, quantity } => {
                self.handle_trade_buy(state, card_id, *quantity, rng)
            }

            Command::TradeSell { instance_id } => self.handle_trade_sell(state, instance_id, rng),

            Command::CardEquip { instance_id } => self.handle_card_equip(state, instance_id),

            Command::CardUnequip { instance_id } => self.handle_card_unequip(state, instance_id),

            Command::CardUpgrade { instance_id } => {
                self.handle_card_upgrade(state, instance_id, rng)
            }

            Command::ModuleInstall {
                instance_id,
                slot_id,
            } => self.handle_module_install(state, instance_id, slot_id),

            Command::ModuleUninstall { slot_id } => self.handle_module_uninstall(state, slot_id),

            Command::ContractAccept { card_id } => {
                self.handle_contract_accept(state, card_id, rng)
            }

            Command::ContractComplete { instance_id } => {
                self.handle_contract_complete(state, instance_id, rng)
            }

            Command::ContractAbandon { instance_id } => {
                self.handle_contract_abandon(state, instance_id, rng)
            }

            Command::CrewHire { card_id } => self.handle_crew_hire(state, card_id, rng),

            Command::CrewDismiss { instance_id } => {
                self.handle_crew_dismiss(state, instance_id, rng)
            }

            Command::Repair { amount } => self.handle_repair(state, *amount, rng),

            Command::Resupply { amount } => self.handle_resupply(state, *amount, rng),

            Command::Refuel { amount } => self.handle_refuel(state, *amount, rng),

            Command::ApplyEffects { effects, reason } => {
                self.apply_effects(state, effects, reason, rng)
            }

            Command::AdvanceCycle => Ok(vec![Event::cycle_advanced(state.cycle + 1)]),

            Command::InitializeGame => {
                let rng_state = rng.state();
                Ok(vec![Event::RngStateAdvanced {
                    new_state: rng_state,
                }])
            }
        }
    }

    fn handle_choice(
        &self,
        state: &GameState,
        scene_id: &SceneId,
        passage_index: usize,
        choice_index: usize,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let scene =
            self.registry
                .get_scene(scene_id)
                .ok_or_else(|| RuntimeError::SceneNotFound {
                    scene_id: scene_id.clone(),
                })?;

        let passage =
            scene
                .passage(passage_index)
                .ok_or_else(|| RuntimeError::PassageNotFound {
                    scene_id: scene_id.clone(),
                    passage_index,
                })?;

        let choice =
            passage
                .choices
                .get(choice_index)
                .ok_or_else(|| RuntimeError::ChoiceNotFound {
                    scene_id: scene_id.clone(),
                    passage_index,
                    choice_index,
                })?;

        let tags = DeckTagProvider::from_state(state, self.registry);

        // Evaluate condition: prefer Rhai if available, fall back to native Requirement
        let condition_met = if let Some(ref rhai_condition) = choice.rhai_condition {
            let executor = ScriptExecutor::new();
            executor
                .eval_condition(rhai_condition, state, &tags)
                .unwrap_or_else(|_| {
                    // If Rhai evaluation fails, fall back to native check
                    choice.requirements.check(state, &tags)
                })
        } else {
            choice.requirements.check(state, &tags)
        };

        if !condition_met {
            return Err(RuntimeError::ChoiceRequirementsNotMet);
        }

        let mut events = vec![Event::ChoiceMade {
            scene_id: scene_id.clone(),
            passage_index,
            choice_index,
        }];

        // Execute effects: prefer Rhai if available, fall back to native effects
        if let Some(ref rhai_effects) = choice.rhai_effects {
            // Execute Rhai script to collect effects
            let executor = ScriptExecutor::new();
            let script_result = executor
                .eval(rhai_effects, state, &tags, rng.state())
                .map_err(|e| RuntimeError::ScriptError {
                    message: e.to_string(),
                })?;
            *rng = Rng::new(script_result.rng_state);

            // Apply the collected effects
            let effect_events = self.apply_effects(state, &script_result.effects, "choice", rng)?;
            events.extend(effect_events);
        } else {
            // Use native effects
            let effect_events = self.apply_effects(state, &choice.effects, "choice", rng)?;
            events.extend(effect_events);
        }

        match &choice.next {
            Navigation::Passage(next_idx) => {
                events.push(Event::PassageEntered {
                    scene_id: scene_id.clone(),
                    passage_index: *next_idx,
                });
            }
            Navigation::End => {
                events.push(Event::scene_ended(scene_id.clone()));
            }
        }

        Ok(events)
    }

    fn apply_effects(
        &self,
        state: &GameState,
        effects: &[Effect],
        reason: &str,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let mut events = Vec::new();

        for effect in effects {
            match effect {
                Effect::ModifyResource { resource, delta } => {
                    let old_value = state.resource(resource);
                    let new_value = old_value + delta;
                    events.push(Event::resource_changed(
                        resource.clone(),
                        old_value,
                        new_value,
                        reason,
                    ));
                }

                Effect::SetResource { resource, value } => {
                    let old_value = state.resource(resource);
                    events.push(Event::resource_changed(
                        resource.clone(),
                        old_value,
                        *value,
                        reason,
                    ));
                }

                Effect::SetFlag { flag, value } => {
                    let old_value = state.flags.get(flag).cloned();
                    events.push(Event::flag_set(flag.clone(), old_value, value.clone()));
                }

                Effect::AddCard { card_id } => {
                    let instance_id =
                        CardInstanceId::new(format!("card-{}-{}", state.cycle, rng.next_u64()));
                    events.push(Event::card_added(card_id.clone(), instance_id));
                }

                Effect::RemoveCards { pattern } => {
                    let pattern_str = pattern.as_str();
                    for inst_id in &state.cards.deck {
                        if let Some(inst) = state.cards.instances.get(inst_id) {
                            let card_id_str = inst.card_id.as_str();
                            let matches = if pattern_str.ends_with('*') {
                                let prefix = &pattern_str[..pattern_str.len() - 1];
                                card_id_str.starts_with(prefix)
                            } else {
                                card_id_str == pattern_str
                            };
                            if matches {
                                events.push(Event::card_removed(inst_id.clone(), reason));
                            }
                        }
                    }
                    for inst_id in &state.cards.collection {
                        if let Some(inst) = state.cards.instances.get(inst_id) {
                            let card_id_str = inst.card_id.as_str();
                            let matches = if pattern_str.ends_with('*') {
                                let prefix = &pattern_str[..pattern_str.len() - 1];
                                card_id_str.starts_with(prefix)
                            } else {
                                card_id_str == pattern_str
                            };
                            if matches {
                                events.push(Event::card_removed(inst_id.clone(), reason));
                            }
                        }
                    }
                }

                Effect::ModifyFactionReputation { faction, delta } => {
                    let old_value = state.faction_reputation(faction);
                    let new_value = (old_value + delta).clamp(-100, 100);
                    events.push(Event::FactionReputationChanged {
                        faction: faction.clone(),
                        old_value,
                        new_value,
                    });
                }

                Effect::AddChronicle { title, text } => {
                    let entry = ChronicleEntry {
                        id: ChronicleEntryId::new(format!(
                            "chr-{}-{}",
                            state.cycle,
                            rng.next_u64()
                        )),
                        entry_type: "effect".into(),
                        cycle: state.cycle,
                        title: title.clone(),
                        text: text.clone(),
                        tags: Tags::new(),
                        location_ref: state.location.clone(),
                        card_refs: Vec::new(),
                        faction_ref: None,
                    };
                    events.push(Event::ChronicleAdded { entry });
                }

                Effect::Damage { resource, amount } => {
                    let old_value = state.resource(resource);
                    let new_value = old_value - amount;
                    events.push(Event::resource_changed(
                        resource.clone(),
                        old_value,
                        new_value,
                        format!("{} (damage)", reason),
                    ));
                }

                Effect::Script { source } => {
                    let tags = DeckTagProvider::from_state(state, self.registry);
                    let (script_events, _goto) =
                        self.execute_script(state, source, &tags, rng, reason)?;
                    events.extend(script_events);
                    // Note: goto navigation is returned but not handled here.
                    // The caller (scene runner) should handle navigation changes.
                }

                Effect::EquipModule { slot_id, card_id } => {
                    let instance_id =
                        CardInstanceId::new(format!("mod-{}-{}", state.cycle, rng.next_u64()));
                    events.push(Event::card_added(card_id.clone(), instance_id.clone()));
                    events.push(Event::ModuleEquipped {
                        slot_id: slot_id.clone(),
                        instance_id,
                    });
                }

                Effect::UnequipModule { slot_id } => {
                    if let Some(instance_id) = state.equipped_module(slot_id) {
                        events.push(Event::ModuleUnequipped {
                            slot_id: slot_id.clone(),
                            instance_id: instance_id.clone(),
                        });
                    }
                }

                Effect::ModifyStat { key, delta } => {
                    let old_value = state.stat(key);
                    let new_value = old_value + delta;
                    events.push(Event::StatChanged {
                        key: key.clone(),
                        new_value,
                    });
                }

                Effect::Compound { effects: inner } => {
                    let inner_events = self.apply_effects(state, inner, reason, rng)?;
                    events.extend(inner_events);
                }
            }
        }

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    // === Trade Handlers ===

    fn handle_trade_buy(
        &self,
        state: &GameState,
        card_id: &engine_core::CardId,
        quantity: u32,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let card_def = self
            .registry
            .get_card(card_id)
            .ok_or_else(|| RuntimeError::CardNotFound {
                card_id: card_id.clone(),
            })?;

        // Check if available at current port
        let location = state.location.as_ref().ok_or(RuntimeError::NotAtPort)?;
        let location_state = state.location_state(location);
        if let Some(loc_state) = location_state {
            if !loc_state.available_cards.contains(card_id) {
                return Err(RuntimeError::CardNotAvailable {
                    card_id: card_id.clone(),
                });
            }
        }

        // Calculate cost
        let base_price = card_def.base_value.unwrap_or(10);
        let modifier = location_state
            .and_then(|ls| ls.market_modifiers.get(card_id).copied())
            .unwrap_or(1.0);
        let total_cost = ((base_price as f64 * modifier) as i64) * (quantity as i64);

        // Check credits
        let credits_id = ResourceId::new("credits");
        let current_credits = state.resource(&credits_id);
        if current_credits < total_cost {
            return Err(RuntimeError::InsufficientCredits {
                required: total_cost,
                available: current_credits,
            });
        }

        let mut events = vec![Event::resource_changed(
            credits_id,
            current_credits,
            current_credits - total_cost,
            "trade buy",
        )];

        // Add cards
        for _ in 0..quantity {
            let instance_id =
                CardInstanceId::new(format!("card-{}-{}", state.cycle, rng.next_u64()));
            events.push(Event::card_added(card_id.clone(), instance_id));
        }

        // Update stats
        events.push(Event::StatChanged {
            key: SmolStr::new("cards_acquired"),
            new_value: state.stat("cards_acquired") + quantity as i64,
        });

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    fn handle_trade_sell(
        &self,
        state: &GameState,
        instance_id: &CardInstanceId,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let instance = state
            .cards
            .instances
            .get(instance_id)
            .ok_or_else(|| RuntimeError::CardInstanceNotFound {
                instance_id: instance_id.clone(),
            })?;

        let card_def = self
            .registry
            .get_card(&instance.card_id)
            .ok_or_else(|| RuntimeError::CardNotFound {
                card_id: instance.card_id.clone(),
            })?;

        // Calculate sell price (70% of buy price, adjusted by condition)
        let base_price = card_def.base_value.unwrap_or(10);
        let location = state.location.as_ref();
        let modifier = location
            .and_then(|loc| state.location_state(loc))
            .and_then(|ls| ls.market_modifiers.get(&instance.card_id).copied())
            .unwrap_or(1.0);
        let condition_factor = instance.condition as f64 / 100.0;
        let sell_price = ((base_price as f64 * modifier * 0.7 * condition_factor) as i64).max(1);

        let credits_id = ResourceId::new("credits");
        let current_credits = state.resource(&credits_id);

        let mut events = vec![
            Event::card_removed(instance_id.clone(), "sold"),
            Event::resource_changed(
                credits_id,
                current_credits,
                current_credits + sell_price,
                "trade sell",
            ),
            Event::StatChanged {
                key: SmolStr::new("total_credits_earned"),
                new_value: state.stat("total_credits_earned") + sell_price,
            },
        ];

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    // === Card Management Handlers ===

    fn handle_card_equip(
        &self,
        state: &GameState,
        instance_id: &CardInstanceId,
    ) -> Result<Vec<Event>, RuntimeError> {
        if !state.cards.instances.contains_key(instance_id) {
            return Err(RuntimeError::CardInstanceNotFound {
                instance_id: instance_id.clone(),
            });
        }

        if state.cards.deck.contains(instance_id) {
            return Err(RuntimeError::AlreadyEquipped {
                instance_id: instance_id.clone(),
            });
        }

        Ok(vec![Event::CardMovedToDeck {
            instance_id: instance_id.clone(),
        }])
    }

    fn handle_card_unequip(
        &self,
        state: &GameState,
        instance_id: &CardInstanceId,
    ) -> Result<Vec<Event>, RuntimeError> {
        if !state.cards.deck.contains(instance_id) {
            return Err(RuntimeError::NotEquipped {
                instance_id: instance_id.clone(),
            });
        }

        Ok(vec![Event::CardMovedToCollection {
            instance_id: instance_id.clone(),
        }])
    }

    fn handle_card_upgrade(
        &self,
        state: &GameState,
        instance_id: &CardInstanceId,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let instance = state
            .cards
            .instances
            .get(instance_id)
            .ok_or_else(|| RuntimeError::CardInstanceNotFound {
                instance_id: instance_id.clone(),
            })?;

        let card_def = self
            .registry
            .get_card(&instance.card_id)
            .ok_or_else(|| RuntimeError::CardNotFound {
                card_id: instance.card_id.clone(),
            })?;

        let upgrade_target = card_def
            .upgrades_to
            .as_ref()
            .ok_or(RuntimeError::CannotUpgrade)?;

        let upgrade_cost = card_def.upgrade_cost.as_ref().ok_or(RuntimeError::CannotUpgrade)?;

        // Check credits
        let credits_id = ResourceId::new("credits");
        let credit_cost = upgrade_cost.get(&credits_id).copied().unwrap_or(0);
        let current_credits = state.resource(&credits_id);

        if current_credits < credit_cost {
            return Err(RuntimeError::InsufficientCredits {
                required: credit_cost,
                available: current_credits,
            });
        }

        // Remove old card, add new one with upgraded ID
        let new_instance_id =
            CardInstanceId::new(format!("card-{}-{}", state.cycle, rng.next_u64()));

        let mut events = vec![
            Event::card_removed(instance_id.clone(), "upgraded"),
            Event::card_added(upgrade_target.clone(), new_instance_id),
        ];

        if credit_cost > 0 {
            events.push(Event::resource_changed(
                credits_id,
                current_credits,
                current_credits - credit_cost,
                "upgrade cost",
            ));
        }

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    // === Module Handlers ===

    fn handle_module_install(
        &self,
        state: &GameState,
        instance_id: &CardInstanceId,
        slot_id: &engine_core::SlotId,
    ) -> Result<Vec<Event>, RuntimeError> {
        let instance = state
            .cards
            .instances
            .get(instance_id)
            .ok_or_else(|| RuntimeError::CardInstanceNotFound {
                instance_id: instance_id.clone(),
            })?;

        let card_def = self
            .registry
            .get_card(&instance.card_id)
            .ok_or_else(|| RuntimeError::CardNotFound {
                card_id: instance.card_id.clone(),
            })?;

        // Check it's a module type
        if card_def.card_type.as_str() != "module" {
            return Err(RuntimeError::NotAModule {
                card_id: instance.card_id.clone(),
            });
        }

        // Check slot is empty
        if state.equipped_modules.contains_key(slot_id) {
            return Err(RuntimeError::SlotOccupied {
                slot_id: slot_id.clone(),
            });
        }

        Ok(vec![Event::ModuleEquipped {
            slot_id: slot_id.clone(),
            instance_id: instance_id.clone(),
        }])
    }

    fn handle_module_uninstall(
        &self,
        state: &GameState,
        slot_id: &engine_core::SlotId,
    ) -> Result<Vec<Event>, RuntimeError> {
        let instance_id = state
            .equipped_modules
            .get(slot_id)
            .ok_or_else(|| RuntimeError::SlotEmpty {
                slot_id: slot_id.clone(),
            })?;

        Ok(vec![Event::ModuleUnequipped {
            slot_id: slot_id.clone(),
            instance_id: instance_id.clone(),
        }])
    }

    // === Contract Handlers ===

    fn handle_contract_accept(
        &self,
        state: &GameState,
        card_id: &engine_core::CardId,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let card_def = self
            .registry
            .get_card(card_id)
            .ok_or_else(|| RuntimeError::CardNotFound {
                card_id: card_id.clone(),
            })?;

        if card_def.card_type.as_str() != "contract" {
            return Err(RuntimeError::NotAContract {
                card_id: card_id.clone(),
            });
        }

        let instance_id =
            CardInstanceId::new(format!("contract-{}-{}", state.cycle, rng.next_u64()));

        let mut events = vec![Event::card_added(card_id.clone(), instance_id)];

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    fn handle_contract_complete(
        &self,
        state: &GameState,
        instance_id: &CardInstanceId,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let instance = state
            .cards
            .instances
            .get(instance_id)
            .ok_or_else(|| RuntimeError::CardInstanceNotFound {
                instance_id: instance_id.clone(),
            })?;

        let card_def = self
            .registry
            .get_card(&instance.card_id)
            .ok_or_else(|| RuntimeError::CardNotFound {
                card_id: instance.card_id.clone(),
            })?;

        let terms = card_def
            .contract_terms
            .as_ref()
            .ok_or(RuntimeError::InvalidContract)?;

        // Check we're at the destination
        if state.location.as_ref() != Some(&terms.destination) {
            return Err(RuntimeError::NotAtDestination {
                required: terms.destination.clone(),
            });
        }

        // Check cargo requirements
        if let Some(ref cargo_req) = terms.cargo_required {
            let owned_count = state
                .deck_card_ids()
                .chain(
                    state
                        .cards
                        .collection
                        .iter()
                        .filter_map(|id| state.cards.instances.get(id))
                        .map(|inst| &inst.card_id),
                )
                .filter(|id| *id == &cargo_req.card_id)
                .count();

            if owned_count < cargo_req.quantity as usize {
                return Err(RuntimeError::InsufficientCargo {
                    required: cargo_req.quantity,
                    available: owned_count as u32,
                });
            }
        }

        let mut events = Vec::new();

        // Remove required cargo
        if let Some(ref cargo_req) = terms.cargo_required {
            let mut to_remove = cargo_req.quantity;
            for inst_id in state.cards.deck.iter().chain(state.cards.collection.iter()) {
                if to_remove == 0 {
                    break;
                }
                if let Some(inst) = state.cards.instances.get(inst_id) {
                    if inst.card_id == cargo_req.card_id {
                        events.push(Event::card_removed(inst_id.clone(), "contract cargo"));
                        to_remove -= 1;
                    }
                }
            }
        }

        // Remove contract
        events.push(Event::card_removed(instance_id.clone(), "contract complete"));

        // Grant rewards
        for (resource_id, &amount) in &terms.reward {
            let current = state.resource(resource_id);
            events.push(Event::resource_changed(
                resource_id.clone(),
                current,
                current + amount,
                "contract reward",
            ));
        }

        // Update stats
        events.push(Event::StatChanged {
            key: SmolStr::new("contracts_completed"),
            new_value: state.stat("contracts_completed") + 1,
        });

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    fn handle_contract_abandon(
        &self,
        state: &GameState,
        instance_id: &CardInstanceId,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let instance = state
            .cards
            .instances
            .get(instance_id)
            .ok_or_else(|| RuntimeError::CardInstanceNotFound {
                instance_id: instance_id.clone(),
            })?;

        let card_def = self.registry.get_card(&instance.card_id);

        let mut events = vec![Event::card_removed(
            instance_id.clone(),
            "contract abandoned",
        )];

        // Apply penalty
        if let Some(def) = card_def {
            if let Some(ref terms) = def.contract_terms {
                if let Some(ref penalty) = terms.penalty {
                    for (resource_id, &amount) in penalty {
                        let current = state.resource(resource_id);
                        events.push(Event::resource_changed(
                            resource_id.clone(),
                            current,
                            (current - amount).max(0),
                            "contract penalty",
                        ));
                    }
                }
            }
        }

        // Default integrity penalty if no specific penalty
        let integrity_id = ResourceId::new("integrity");
        if card_def.map_or(true, |d| {
            d.contract_terms
                .as_ref()
                .map_or(true, |t| t.penalty.is_none())
        }) {
            let current = state.resource(&integrity_id);
            events.push(Event::resource_changed(
                integrity_id,
                current,
                (current - 10).max(0),
                "contract penalty",
            ));
        }

        events.push(Event::StatChanged {
            key: SmolStr::new("contracts_failed"),
            new_value: state.stat("contracts_failed") + 1,
        });

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    // === Crew Handlers ===

    fn handle_crew_hire(
        &self,
        state: &GameState,
        card_id: &engine_core::CardId,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let card_def = self
            .registry
            .get_card(card_id)
            .ok_or_else(|| RuntimeError::CardNotFound {
                card_id: card_id.clone(),
            })?;

        if card_def.card_type.as_str() != "crew" {
            return Err(RuntimeError::NotACrew {
                card_id: card_id.clone(),
            });
        }

        let hire_cost = card_def.base_value.unwrap_or(50);
        let credits_id = ResourceId::new("credits");
        let current_credits = state.resource(&credits_id);

        if current_credits < hire_cost {
            return Err(RuntimeError::InsufficientCredits {
                required: hire_cost,
                available: current_credits,
            });
        }

        let instance_id = CardInstanceId::new(format!("crew-{}-{}", state.cycle, rng.next_u64()));

        let mut events = vec![
            Event::resource_changed(
                credits_id,
                current_credits,
                current_credits - hire_cost,
                "crew hire",
            ),
            Event::card_added(card_id.clone(), instance_id.clone()),
            Event::CardMovedToDeck {
                instance_id: instance_id.clone(),
            },
            Event::StatChanged {
                key: SmolStr::new("cards_acquired"),
                new_value: state.stat("cards_acquired") + 1,
            },
        ];

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    fn handle_crew_dismiss(
        &self,
        state: &GameState,
        instance_id: &CardInstanceId,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        if !state.cards.instances.contains_key(instance_id) {
            return Err(RuntimeError::CardInstanceNotFound {
                instance_id: instance_id.clone(),
            });
        }

        let integrity_id = ResourceId::new("integrity");
        let current_integrity = state.resource(&integrity_id);

        let mut events = vec![
            Event::card_removed(instance_id.clone(), "crew dismissed"),
            Event::resource_changed(
                integrity_id,
                current_integrity,
                (current_integrity - 5).max(0),
                "crew dismissal",
            ),
        ];

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    // === Port Service Handlers ===

    fn handle_repair(
        &self,
        state: &GameState,
        amount: i64,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let cost = amount * self.config.base_repair_cost;
        let credits_id = ResourceId::new("credits");
        let hull_id = ResourceId::new("hull");

        let current_credits = state.resource(&credits_id);
        if current_credits < cost {
            return Err(RuntimeError::InsufficientCredits {
                required: cost,
                available: current_credits,
            });
        }

        let current_hull = state.resource(&hull_id);
        let max_hull = self
            .schema
            .resource(&hull_id)
            .and_then(|r| r.max)
            .unwrap_or(100);
        let new_hull = (current_hull + amount).min(max_hull);

        let mut events = vec![
            Event::resource_changed(credits_id, current_credits, current_credits - cost, "repair"),
            Event::resource_changed(hull_id, current_hull, new_hull, "repair"),
        ];

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    fn handle_resupply(
        &self,
        state: &GameState,
        amount: i64,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let cost = amount * self.config.base_supply_cost;
        let credits_id = ResourceId::new("credits");
        let supplies_id = ResourceId::new("supplies");

        let current_credits = state.resource(&credits_id);
        if current_credits < cost {
            return Err(RuntimeError::InsufficientCredits {
                required: cost,
                available: current_credits,
            });
        }

        let current_supplies = state.resource(&supplies_id);

        let mut events = vec![
            Event::resource_changed(
                credits_id,
                current_credits,
                current_credits - cost,
                "resupply",
            ),
            Event::resource_changed(
                supplies_id,
                current_supplies,
                current_supplies + amount,
                "resupply",
            ),
        ];

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    fn handle_refuel(
        &self,
        state: &GameState,
        amount: i64,
        rng: &mut Rng,
    ) -> Result<Vec<Event>, RuntimeError> {
        let cost = amount * self.config.base_fuel_cost;
        let credits_id = ResourceId::new("credits");
        let fuel_id = ResourceId::new("fuel");

        let current_credits = state.resource(&credits_id);
        if current_credits < cost {
            return Err(RuntimeError::InsufficientCredits {
                required: cost,
                available: current_credits,
            });
        }

        let current_fuel = state.resource(&fuel_id);

        let mut events = vec![
            Event::resource_changed(credits_id, current_credits, current_credits - cost, "refuel"),
            Event::resource_changed(fuel_id, current_fuel, current_fuel + amount, "refuel"),
        ];

        events.push(Event::RngStateAdvanced {
            new_state: rng.state(),
        });

        Ok(events)
    }

    // === Script Execution ===

    fn execute_script(
        &self,
        state: &GameState,
        source: &str,
        tags: &impl TagProvider,
        rng: &mut Rng,
        reason: &str,
    ) -> Result<(Vec<Event>, Option<SmolStr>), RuntimeError> {
        let executor = ScriptExecutor::new();

        let script_result = executor
            .eval(source, state, tags, rng.state())
            .map_err(|e| RuntimeError::ScriptError {
                message: e.to_string(),
            })?;

        // Update RNG state from script execution
        // Note: We create a new Rng with the script's final state to sync
        *rng = Rng::new(script_result.rng_state);

        // Convert collected effects to events
        let mut events = Vec::new();
        if !script_result.effects.is_empty() {
            let effect_events = self.apply_effects(state, &script_result.effects, reason, rng)?;
            events.extend(effect_events);
        }

        Ok((events, script_result.goto))
    }
}
