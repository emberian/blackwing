use serde::{Deserialize, Serialize};

/// Game configuration for tuning gameplay parameters.
/// These values control the economic and mechanical balance of the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Base fuel consumed per jump
    pub base_fuel_per_jump: i64,

    /// Cost in credits to repair 1 hull point
    pub base_repair_cost: i64,

    /// Cost in credits to buy 1 supply unit
    pub base_supply_cost: i64,

    /// Cost in credits to buy 1 fuel unit
    pub base_fuel_cost: i64,

    /// Range of events that can occur during a journey
    pub journey_event_count: EventCountRange,

    /// Supplies consumed per cycle during journey
    pub journey_supply_cost: i64,

    /// Hull wear per cycle during journey
    pub journey_hull_wear: i64,

    /// Default time limit for contracts without explicit limit
    pub contract_time_limit: u32,

    /// Integrity damage when starving (no supplies)
    pub starvation_integrity_damage: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EventCountRange {
    pub min: u32,
    pub max: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            base_fuel_per_jump: 10,
            base_repair_cost: 6,
            base_supply_cost: 2,
            base_fuel_cost: 3,
            journey_event_count: EventCountRange { min: 1, max: 3 },
            journey_supply_cost: 2,
            journey_hull_wear: 2,
            contract_time_limit: 10,
            starvation_integrity_damage: 15,
        }
    }
}

impl GameConfig {
    /// Calculate fuel cost for a jump, accounting for efficiency modifiers
    pub fn fuel_cost_with_efficiency(&self, efficiency_multiplier: f64) -> i64 {
        let cost = (self.base_fuel_per_jump as f64 / efficiency_multiplier).ceil() as i64;
        cost.max(1)
    }

    /// Calculate the number of events for a journey using RNG
    pub fn roll_journey_event_count(&self, rng_value: f64) -> u32 {
        let range = self.journey_event_count.max - self.journey_event_count.min + 1;
        self.journey_event_count.min + (rng_value * range as f64) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = GameConfig::default();
        assert_eq!(config.base_fuel_per_jump, 10);
        assert_eq!(config.base_repair_cost, 6);
        assert_eq!(config.journey_event_count.min, 1);
        assert_eq!(config.journey_event_count.max, 3);
    }

    #[test]
    fn fuel_efficiency_calculation() {
        let config = GameConfig::default();
        // No efficiency bonus
        assert_eq!(config.fuel_cost_with_efficiency(1.0), 10);
        // 50% more efficient
        assert_eq!(config.fuel_cost_with_efficiency(1.5), 7);
        // Double efficiency
        assert_eq!(config.fuel_cost_with_efficiency(2.0), 5);
        // Min fuel is 1
        assert_eq!(config.fuel_cost_with_efficiency(100.0), 1);
    }

    #[test]
    fn journey_event_count_roll() {
        let config = GameConfig::default();
        // Min roll
        assert_eq!(config.roll_journey_event_count(0.0), 1);
        // Max roll (just under 1.0)
        assert_eq!(config.roll_journey_event_count(0.99), 3);
        // Mid roll
        assert_eq!(config.roll_journey_event_count(0.5), 2);
    }
}
