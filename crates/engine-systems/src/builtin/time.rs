//! Time system for game time management.
//!
//! Handles:
//! - Advancing game time (ticks)
//! - Wait commands
//! - Time-based events (in tick)

use crate::command::Command;
use crate::context::{SystemContext, WorldView};
use crate::effect::Effect;
use crate::error::SystemError;
use crate::system::System;

/// System for managing game time.
pub struct TimeSystem {
    // Could have config like max_wait_ticks
}

impl TimeSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TimeSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for TimeSystem {
    fn id(&self) -> &str {
        "time"
    }

    fn handles_commands(&self) -> &[&str] {
        &["wait", "advance_time"]
    }

    fn handle_command(
        &self,
        _world: &WorldView<'_>,
        command: &Command,
        _ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        match command.kind.as_str() {
            "wait" => {
                let ticks = command.get_int("ticks").unwrap_or(1) as u64;
                Ok(vec![Effect::AdvanceTime { ticks }])
            }

            "advance_time" => {
                let ticks = command
                    .get_int("ticks")
                    .ok_or_else(|| SystemError::missing_arg("ticks"))? as u64;
                Ok(vec![Effect::AdvanceTime { ticks }])
            }

            _ => Err(SystemError::UnknownCommand(command.kind.clone())),
        }
    }

    fn tick(
        &self,
        _world: &WorldView<'_>,
        _delta_ticks: u64,
        _ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        // Could check for scheduled events, timers, etc.
        Ok(vec![])
    }

    fn priority(&self) -> i32 {
        // Time system runs with low priority (other systems first)
        -100
    }
}
