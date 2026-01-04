//! Quest system for tracking objectives and progress.
//!
//! Handles:
//! - Starting and completing quests
//! - Tracking quest stages and objectives
//! - Auto-advancing quests when objectives are met

use engine_primitives::Value;
use engine_world::ScopeKind;
use smol_str::SmolStr;

use crate::command::Command;
use crate::context::{SystemContext, WorldView};
use crate::effect::Effect;
use crate::error::SystemError;
use crate::system::System;

/// Component key for storing active quests on an actor
const ACTIVE_QUESTS_COMPONENT: &str = "active_quests";
/// Component key for storing completed quests on an actor
const COMPLETED_QUESTS_COMPONENT: &str = "completed_quests";
/// Component key for quest stage progress
const QUEST_STAGE_PREFIX: &str = "quest_stage_";

/// System for tracking quests with stages and objectives.
///
/// Quests are tracked via actor components:
/// - `active_quests`: Array of quest IDs currently in progress
/// - `completed_quests`: Array of quest IDs that are done
/// - `quest_stage_{quest_id}`: Current stage ID for each active quest
///
/// Quest stages can have objectives that auto-complete:
/// - `visit`: Player visits a location
/// - `interact`: Player interacts with an entity
/// - `collect`: Player collects items
/// - `kill`: Player defeats enemies
pub struct QuestSystem;

impl QuestSystem {
    pub fn new() -> Self {
        Self
    }

    /// Get the current stage for a quest from an actor's components
    fn get_quest_stage<'a>(
        world: &'a WorldView<'_>,
        actor: &engine_primitives::EntityId,
        quest_id: &str,
    ) -> Option<SmolStr> {
        let key = world.entities().key_of(actor)?;
        let component_key = format!("{}{}", QUEST_STAGE_PREFIX, quest_id);
        world
            .entities()
            .get_component(key, &component_key)
            .and_then(|v| v.as_str())
            .map(SmolStr::new)
    }

    /// Check if an actor has a quest active
    fn has_active_quest(
        world: &WorldView<'_>,
        actor: &engine_primitives::EntityId,
        quest_id: &str,
    ) -> bool {
        let Some(key) = world.entities().key_of(actor) else {
            return false;
        };
        let Some(quests) = world.entities().get_component(key, ACTIVE_QUESTS_COMPONENT) else {
            return false;
        };
        let Some(arr) = quests.as_array() else {
            return false;
        };
        arr.iter().any(|v| v.as_str() == Some(quest_id))
    }
}

impl Default for QuestSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for QuestSystem {
    fn id(&self) -> &str {
        "quest"
    }

    fn scope_filter(&self) -> &[ScopeKind] {
        // Quest commands work in any scope
        &[]
    }

    fn handles_commands(&self) -> &[&str] {
        &[
            "start_quest",
            "advance_quest",
            "complete_quest",
            "abandon_quest",
            "check_quest_objective",
        ]
    }

    fn handle_command(
        &self,
        world: &WorldView<'_>,
        command: &Command,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        match command.kind.as_str() {
            "start_quest" => {
                let quest_id = command
                    .get_str("quest")
                    .ok_or_else(|| SystemError::missing_arg("quest"))?;

                let initial_stage = command.get_str("stage").unwrap_or("start");

                // Check if quest is already active
                if Self::has_active_quest(world, &ctx.actor, quest_id) {
                    return Err(SystemError::invalid_state(format!(
                        "Quest {} is already active",
                        quest_id
                    )));
                }

                // Get current active quests
                let current_quests: Vec<Value> = world
                    .entities()
                    .key_of(&ctx.actor)
                    .and_then(|key| world.entities().get_component(key, ACTIVE_QUESTS_COMPONENT))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.to_vec())
                    .unwrap_or_default();

                // Add new quest to list
                let mut new_quests = current_quests;
                new_quests.push(Value::String(quest_id.into()));

                let stage_key = format!("{}{}", QUEST_STAGE_PREFIX, quest_id);

                Ok(vec![
                    // Update active quests list
                    Effect::set_component(
                        ctx.actor.clone(),
                        ACTIVE_QUESTS_COMPONENT,
                        Value::Array(new_quests),
                    ),
                    // Set initial stage
                    Effect::set_component(
                        ctx.actor.clone(),
                        stage_key,
                        Value::String(initial_stage.into()),
                    ),
                    Effect::chronicle(
                        "Quest Started",
                        format!("Started quest: {}", quest_id),
                    ),
                ])
            }

            "advance_quest" => {
                let quest_id = command
                    .get_str("quest")
                    .ok_or_else(|| SystemError::missing_arg("quest"))?;

                let new_stage = command
                    .get_str("stage")
                    .ok_or_else(|| SystemError::missing_arg("stage"))?;

                // Verify quest is active
                if !Self::has_active_quest(world, &ctx.actor, quest_id) {
                    return Err(SystemError::invalid_state(format!(
                        "Quest {} is not active",
                        quest_id
                    )));
                }

                let stage_key = format!("{}{}", QUEST_STAGE_PREFIX, quest_id);

                Ok(vec![
                    Effect::set_component(
                        ctx.actor.clone(),
                        stage_key,
                        Value::String(new_stage.into()),
                    ),
                    Effect::chronicle(
                        "Quest Advanced",
                        format!("Quest {} advanced to stage: {}", quest_id, new_stage),
                    ),
                ])
            }

            "complete_quest" => {
                let quest_id = command
                    .get_str("quest")
                    .ok_or_else(|| SystemError::missing_arg("quest"))?;

                // Verify quest is active
                if !Self::has_active_quest(world, &ctx.actor, quest_id) {
                    return Err(SystemError::invalid_state(format!(
                        "Quest {} is not active",
                        quest_id
                    )));
                }

                // Remove from active quests
                let current_quests: Vec<Value> = world
                    .entities()
                    .key_of(&ctx.actor)
                    .and_then(|key| world.entities().get_component(key, ACTIVE_QUESTS_COMPONENT))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.to_vec())
                    .unwrap_or_default();

                let new_active: Vec<Value> = current_quests
                    .into_iter()
                    .filter(|v: &Value| v.as_str() != Some(quest_id))
                    .collect();

                // Add to completed quests
                let current_completed: Vec<Value> = world
                    .entities()
                    .key_of(&ctx.actor)
                    .and_then(|key| world.entities().get_component(key, COMPLETED_QUESTS_COMPONENT))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.to_vec())
                    .unwrap_or_default();

                let mut new_completed = current_completed;
                new_completed.push(Value::String(quest_id.into()));

                let stage_key = format!("{}{}", QUEST_STAGE_PREFIX, quest_id);

                Ok(vec![
                    Effect::set_component(
                        ctx.actor.clone(),
                        ACTIVE_QUESTS_COMPONENT,
                        Value::Array(new_active),
                    ),
                    Effect::set_component(
                        ctx.actor.clone(),
                        COMPLETED_QUESTS_COMPONENT,
                        Value::Array(new_completed),
                    ),
                    // Remove stage tracking
                    Effect::RemoveComponent {
                        entity: ctx.actor.clone(),
                        component: stage_key.into(),
                    },
                    Effect::chronicle(
                        "Quest Completed",
                        format!("Completed quest: {}", quest_id),
                    ),
                ])
            }

            "abandon_quest" => {
                let quest_id = command
                    .get_str("quest")
                    .ok_or_else(|| SystemError::missing_arg("quest"))?;

                // Verify quest is active
                if !Self::has_active_quest(world, &ctx.actor, quest_id) {
                    return Err(SystemError::invalid_state(format!(
                        "Quest {} is not active",
                        quest_id
                    )));
                }

                // Remove from active quests
                let current_quests: Vec<Value> = world
                    .entities()
                    .key_of(&ctx.actor)
                    .and_then(|key| world.entities().get_component(key, ACTIVE_QUESTS_COMPONENT))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.to_vec())
                    .unwrap_or_default();

                let new_active: Vec<Value> = current_quests
                    .into_iter()
                    .filter(|v: &Value| v.as_str() != Some(quest_id))
                    .collect();

                let stage_key = format!("{}{}", QUEST_STAGE_PREFIX, quest_id);

                Ok(vec![
                    Effect::set_component(
                        ctx.actor.clone(),
                        ACTIVE_QUESTS_COMPONENT,
                        Value::Array(new_active),
                    ),
                    Effect::RemoveComponent {
                        entity: ctx.actor.clone(),
                        component: stage_key.into(),
                    },
                    Effect::chronicle(
                        "Quest Abandoned",
                        format!("Abandoned quest: {}", quest_id),
                    ),
                ])
            }

            "check_quest_objective" => {
                // This is used to check if an objective has been met
                // The actual check would be done by the runtime/content layer
                // This just returns the current state
                let quest_id = command
                    .get_str("quest")
                    .ok_or_else(|| SystemError::missing_arg("quest"))?;

                if !Self::has_active_quest(world, &ctx.actor, quest_id) {
                    return Ok(vec![]);
                }

                let _current_stage = Self::get_quest_stage(world, &ctx.actor, quest_id);

                // In a full implementation, we would check the objective conditions
                // and return effects to advance if met
                Ok(vec![])
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
        // In a full implementation, we would iterate over all actors
        // and check quest objectives for auto-advancement
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_primitives::EntityId;
    use engine_world::WorldState;

    #[test]
    fn start_quest() {
        let system = QuestSystem::new();
        let mut state = WorldState::new();

        let actor = EntityId::new("actor", "player");
        state.entities.spawn(actor.clone());

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("start_quest")
            .with_arg("quest", Value::String("rescue_miner".into()));

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        // Should have: set active_quests, set stage, chronicle
        assert_eq!(effects.len(), 3);

        match &effects[0] {
            Effect::SetComponent { component, value, .. } => {
                assert_eq!(component.as_str(), ACTIVE_QUESTS_COMPONENT);
                let arr = value.as_array().unwrap();
                assert_eq!(arr.len(), 1);
                assert_eq!(arr[0].as_str(), Some("rescue_miner"));
            }
            _ => panic!("Expected SetComponent effect"),
        }
    }

    #[test]
    fn complete_quest() {
        let system = QuestSystem::new();
        let mut state = WorldState::new();

        let actor = EntityId::new("actor", "player");
        let key = state.entities.spawn(actor.clone());

        // Set up active quest
        state.entities.set_component(
            key,
            ACTIVE_QUESTS_COMPONENT,
            Value::Array(vec![Value::String("rescue_miner".into())]),
        );
        state.entities.set_component(
            key,
            "quest_stage_rescue_miner",
            Value::String("return".into()),
        );

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("complete_quest")
            .with_arg("quest", Value::String("rescue_miner".into()));

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        // Should have: update active_quests, update completed_quests, remove stage, chronicle
        assert_eq!(effects.len(), 4);
    }

    #[test]
    fn cannot_start_duplicate_quest() {
        let system = QuestSystem::new();
        let mut state = WorldState::new();

        let actor = EntityId::new("actor", "player");
        let key = state.entities.spawn(actor.clone());

        // Already have this quest active
        state.entities.set_component(
            key,
            ACTIVE_QUESTS_COMPONENT,
            Value::Array(vec![Value::String("rescue_miner".into())]),
        );

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("start_quest")
            .with_arg("quest", Value::String("rescue_miner".into()));

        let result = system.handle_command(&view, &cmd, &mut ctx);
        assert!(result.is_err());
    }
}
