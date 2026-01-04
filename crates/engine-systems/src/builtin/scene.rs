//! Scene system for narrative content.
//!
//! Handles:
//! - Entering and exiting scenes
//! - Making choices in passages
//! - Scene-local state

use engine_world::ScopeKind;

use crate::command::Command;
use crate::context::{SystemContext, WorldView};
use crate::effect::Effect;
use crate::error::SystemError;
use crate::system::System;

/// System for handling narrative scenes with passages and choices.
pub struct SceneSystem {
    // Configuration could go here
}

impl SceneSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SceneSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for SceneSystem {
    fn id(&self) -> &str {
        "scene"
    }

    fn scope_filter(&self) -> &[ScopeKind] {
        // Only active when in a Scene scope
        &[ScopeKind::Scene]
    }

    fn handles_commands(&self) -> &[&str] {
        &["make_choice", "advance_scene", "exit_scene"]
    }

    fn handle_command(
        &self,
        world: &WorldView<'_>,
        command: &Command,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        // Verify we're in a scene scope
        let scopes = world
            .scopes(&ctx.actor)
            .ok_or_else(|| SystemError::invalid_state("Actor has no scope stack"))?;

        let current = scopes
            .current()
            .ok_or_else(|| SystemError::invalid_state("No active scope"))?;

        if current.kind != ScopeKind::Scene {
            return Err(SystemError::NotInScope);
        }

        match command.kind.as_str() {
            "make_choice" => {
                let _scene_id = command
                    .get_str("scene_id")
                    .ok_or_else(|| SystemError::missing_arg("scene_id"))?;
                let _passage = command
                    .get_int("passage")
                    .ok_or_else(|| SystemError::missing_arg("passage"))?;
                let _choice = command
                    .get_int("choice")
                    .ok_or_else(|| SystemError::missing_arg("choice"))?;

                // In a full implementation, we would:
                // 1. Look up the scene from content registry
                // 2. Validate the passage/choice indices
                // 3. Execute choice effects
                // 4. Advance to next passage or exit scene

                // For now, return a placeholder chronicle entry
                Ok(vec![Effect::chronicle(
                    "Choice Made",
                    "Player made a choice in the scene.",
                )])
            }

            "advance_scene" => {
                // Advance to next passage automatically
                Ok(vec![])
            }

            "exit_scene" => {
                // Pop the scene scope
                Ok(vec![Effect::pop_scope(ctx.actor.clone())])
            }

            _ => Err(SystemError::UnknownCommand(command.kind.clone())),
        }
    }
}
