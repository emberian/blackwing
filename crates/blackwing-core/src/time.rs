//! Game time representation.
//!
//! Supports both discrete (turn-based) and real-time game modes,
//! as well as hybrid modes that can switch between them.

use serde::{Deserialize, Serialize};

/// Represents game time, supporting both discrete and real-time modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameTime {
    /// Turn-based time measured in discrete ticks
    Discrete { tick: u64 },
    /// Real-time measured in milliseconds with a fixed tick rate
    RealTime { elapsed_ms: u64, tick_rate_ms: u64 },
}

impl GameTime {
    /// Create a new discrete (turn-based) time starting at tick 0
    pub fn discrete() -> Self {
        GameTime::Discrete { tick: 0 }
    }

    /// Create a new real-time clock with the given tick rate
    pub fn real_time(tick_rate_ms: u64) -> Self {
        GameTime::RealTime {
            elapsed_ms: 0,
            tick_rate_ms,
        }
    }

    /// Get the current tick number
    pub fn tick(&self) -> u64 {
        match self {
            GameTime::Discrete { tick } => *tick,
            GameTime::RealTime {
                elapsed_ms,
                tick_rate_ms,
            } => elapsed_ms / tick_rate_ms,
        }
    }

    /// Get elapsed milliseconds (0 for discrete time)
    pub fn elapsed_ms(&self) -> u64 {
        match self {
            GameTime::Discrete { .. } => 0,
            GameTime::RealTime { elapsed_ms, .. } => *elapsed_ms,
        }
    }

    /// Advance time by one tick (for discrete) or by delta_ms (for real-time)
    pub fn advance(&mut self, delta_ms: u64) {
        match self {
            GameTime::Discrete { tick } => *tick += 1,
            GameTime::RealTime { elapsed_ms, .. } => *elapsed_ms += delta_ms,
        }
    }

    /// Advance by exactly one tick
    pub fn advance_tick(&mut self) {
        match self {
            GameTime::Discrete { tick } => *tick += 1,
            GameTime::RealTime {
                elapsed_ms,
                tick_rate_ms,
            } => *elapsed_ms += *tick_rate_ms,
        }
    }

    /// Check if this is discrete (turn-based) time
    pub fn is_discrete(&self) -> bool {
        matches!(self, GameTime::Discrete { .. })
    }

    /// Check if this is real-time
    pub fn is_real_time(&self) -> bool {
        matches!(self, GameTime::RealTime { .. })
    }

    /// Get the tick rate in milliseconds (returns None for discrete time)
    pub fn tick_rate_ms(&self) -> Option<u64> {
        match self {
            GameTime::Discrete { .. } => None,
            GameTime::RealTime { tick_rate_ms, .. } => Some(*tick_rate_ms),
        }
    }
}

impl Default for GameTime {
    fn default() -> Self {
        GameTime::discrete()
    }
}

/// Game mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// Turn-based: time only advances via explicit commands
    TurnBased,
    /// Real-time: time advances automatically at tick_rate_ms
    RealTime { tick_rate_ms: u64 },
    /// Hybrid: can switch between turn-based and real-time
    Hybrid { auto_tick: bool, tick_rate_ms: u64 },
}

impl GameMode {
    pub fn is_turn_based(&self) -> bool {
        matches!(self, GameMode::TurnBased)
    }

    pub fn is_real_time(&self) -> bool {
        matches!(self, GameMode::RealTime { .. })
    }

    pub fn should_auto_tick(&self) -> bool {
        match self {
            GameMode::TurnBased => false,
            GameMode::RealTime { .. } => true,
            GameMode::Hybrid { auto_tick, .. } => *auto_tick,
        }
    }

    pub fn tick_rate_ms(&self) -> Option<u64> {
        match self {
            GameMode::TurnBased => None,
            GameMode::RealTime { tick_rate_ms } => Some(*tick_rate_ms),
            GameMode::Hybrid { tick_rate_ms, .. } => Some(*tick_rate_ms),
        }
    }
}

impl Default for GameMode {
    fn default() -> Self {
        GameMode::TurnBased
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discrete_time_advances() {
        let mut time = GameTime::discrete();
        assert_eq!(time.tick(), 0);

        time.advance_tick();
        assert_eq!(time.tick(), 1);

        time.advance_tick();
        time.advance_tick();
        assert_eq!(time.tick(), 3);
    }

    #[test]
    fn real_time_advances() {
        let mut time = GameTime::real_time(100); // 100ms per tick
        assert_eq!(time.tick(), 0);
        assert_eq!(time.elapsed_ms(), 0);

        time.advance(50);
        assert_eq!(time.tick(), 0); // Still tick 0
        assert_eq!(time.elapsed_ms(), 50);

        time.advance(60);
        assert_eq!(time.tick(), 1); // Now tick 1
        assert_eq!(time.elapsed_ms(), 110);
    }

    #[test]
    fn game_mode_auto_tick() {
        assert!(!GameMode::TurnBased.should_auto_tick());
        assert!(GameMode::RealTime { tick_rate_ms: 100 }.should_auto_tick());
        assert!(!GameMode::Hybrid {
            auto_tick: false,
            tick_rate_ms: 100
        }
        .should_auto_tick());
        assert!(GameMode::Hybrid {
            auto_tick: true,
            tick_rate_ms: 100
        }
        .should_auto_tick());
    }
}
