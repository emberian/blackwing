use engine_core::{CardInstanceId, Event, GameConfig, GameState, ResourceId};
use smol_str::SmolStr;

use crate::{ContentRegistry, Rng};

/// State tracking for an in-progress journey
#[derive(Debug, Clone)]
pub struct JourneyState {
    pub destination: engine_core::LocationId,
    pub events_remaining: u32,
    pub total_events: u32,
}

impl JourneyState {
    pub fn new(
        destination: engine_core::LocationId,
        event_count: u32,
    ) -> Self {
        Self {
            destination,
            events_remaining: event_count,
            total_events: event_count,
        }
    }

    pub fn consume_event(&mut self) {
        self.events_remaining = self.events_remaining.saturating_sub(1);
    }

    pub fn is_complete(&self) -> bool {
        self.events_remaining == 0
    }
}

/// Calculate fuel efficiency multiplier from equipped cards
pub fn calculate_fuel_efficiency(state: &GameState, registry: &dyn ContentRegistry) -> f64 {
    let mut efficiency = 1.0;

    // Check deck cards
    for card_id in state.deck_card_ids() {
        if let Some(card_def) = registry.get_card(card_id) {
            if let Some(&modifier) = card_def.effects.modifiers.get("fuel_efficiency") {
                efficiency += modifier as f64 / 100.0;
            }
        }
    }

    // Check equipped modules
    for card_id in state.equipped_module_card_ids() {
        if let Some(card_def) = registry.get_card(card_id) {
            if let Some(&modifier) = card_def.effects.modifiers.get("fuel_efficiency") {
                efficiency += modifier as f64 / 100.0;
            }
        }
    }

    efficiency
}

/// Process journey wear (supplies consumption, hull degradation)
pub fn process_journey_wear(
    state: &GameState,
    config: &GameConfig,
) -> Vec<Event> {
    let mut events = Vec::new();
    let supplies_id = ResourceId::new("supplies");
    let hull_id = ResourceId::new("hull");
    let integrity_id = ResourceId::new("integrity");

    // Calculate crew-adjusted supply cost
    let crew_count = state.cards.active_crew.len() as i64;
    let supply_cost = config.journey_supply_cost + crew_count;

    let current_supplies = state.resource(&supplies_id);
    let new_supplies = (current_supplies - supply_cost).max(0);
    let starving = new_supplies == 0 && current_supplies > 0;

    // Supplies consumption
    events.push(Event::resource_changed(
        supplies_id,
        current_supplies,
        new_supplies,
        "journey supplies",
    ));

    // Hull wear
    let current_hull = state.resource(&hull_id);
    let new_hull = (current_hull - config.journey_hull_wear).max(0);
    events.push(Event::resource_changed(
        hull_id,
        current_hull,
        new_hull,
        "journey wear",
    ));

    // Starvation damage to integrity
    if starving {
        let current_integrity = state.resource(&integrity_id);
        let new_integrity = (current_integrity - config.starvation_integrity_damage).max(0);
        events.push(Event::resource_changed(
            integrity_id,
            current_integrity,
            new_integrity,
            "starvation",
        ));
    }

    events
}

/// Process cargo decay for items with decay_chance
pub fn process_cargo_decay(
    state: &GameState,
    registry: &dyn ContentRegistry,
    rng: &mut Rng,
) -> (Vec<Event>, Vec<SmolStr>) {
    let mut events = Vec::new();
    let mut decayed_cards = Vec::new();

    // Check all cards in deck and collection
    let all_cards: Vec<CardInstanceId> = state
        .cards
        .deck
        .iter()
        .chain(state.cards.collection.iter())
        .cloned()
        .collect();

    for inst_id in all_cards {
        let Some(instance) = state.cards.instances.get(&inst_id) else {
            continue;
        };

        let Some(card_def) = registry.get_card(&instance.card_id) else {
            continue;
        };

        let Some(ref journey_behavior) = card_def.journey_behavior else {
            continue;
        };

        if journey_behavior.decay_chance <= 0.0 {
            continue;
        }

        // Roll for decay
        if rng.next_f64() < journey_behavior.decay_chance {
            let new_condition = instance.condition.saturating_sub(10);

            if new_condition == 0 {
                // Card is destroyed
                decayed_cards.push(card_def.name.clone());
                events.push(Event::card_removed(inst_id, "decay"));
            } else {
                // Card condition reduced - we'd need a CardConditionChanged event
                // For now, we track this in the decayed_cards list
            }
        }
    }

    (events, decayed_cards)
}

/// Tick contract timers and expire any that run out
pub fn tick_contract_timers(
    state: &GameState,
    registry: &dyn ContentRegistry,
) -> (Vec<Event>, Vec<SmolStr>) {
    let mut events = Vec::new();
    let mut expired_contracts = Vec::new();

    for contract_id in &state.cards.active_contracts {
        let Some(instance) = state.cards.instances.get(contract_id) else {
            continue;
        };

        let Some(cycles_remaining) = instance.cycles_remaining else {
            continue;
        };

        if cycles_remaining <= 1 {
            // Contract expires
            let card_def = registry.get_card(&instance.card_id);
            let name = card_def
                .map(|d| d.name.clone())
                .unwrap_or_else(|| SmolStr::new("Unknown contract"));
            expired_contracts.push(name);

            events.push(Event::card_removed(contract_id.clone(), "contract expired"));

            // Apply penalty
            if let Some(card_def) = card_def {
                if let Some(ref terms) = card_def.contract_terms {
                    if let Some(ref penalty) = terms.penalty {
                        for (resource_id, &amount) in penalty {
                            let current = state.resource(resource_id);
                            let new_val = (current - amount).max(0);
                            events.push(Event::resource_changed(
                                resource_id.clone(),
                                current,
                                new_val,
                                "contract penalty",
                            ));
                        }
                    }
                }
            }

            // Track stats
            events.push(Event::StatChanged {
                key: SmolStr::new("contracts_failed"),
                new_value: state.stat("contracts_failed") + 1,
            });
        }
        // Note: Decrementing cycles_remaining would require a new event type
        // For now, we assume this is handled by the caller or a CardUpdated event
    }

    (events, expired_contracts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journey_state_progression() {
        let mut journey = JourneyState::new(
            engine_core::LocationId::new("test_port"),
            3,
        );
        assert_eq!(journey.events_remaining, 3);
        assert!(!journey.is_complete());

        journey.consume_event();
        assert_eq!(journey.events_remaining, 2);

        journey.consume_event();
        journey.consume_event();
        assert!(journey.is_complete());

        // Should not go negative
        journey.consume_event();
        assert_eq!(journey.events_remaining, 0);
    }
}
