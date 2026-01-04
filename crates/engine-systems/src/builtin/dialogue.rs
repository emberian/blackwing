//! Dialogue system for NPC conversations.
//!
//! Handles:
//! - Starting and ending dialogues
//! - Navigating dialogue nodes
//! - Choice selection with conditions
//! - Running node scripts

use engine_primitives::Value;
use engine_world::ScopeKind;

use crate::command::Command;
use crate::context::{SystemContext, WorldView};
use crate::effect::Effect;
use crate::error::SystemError;
use crate::system::System;

/// System for handling NPC conversations with branching dialogue trees.
///
/// Dialogues are structured as nodes with text and choices. Each node can have:
/// - Text to display to the player
/// - Choices that lead to other nodes (with optional conditions)
/// - Scripts that run on entering the node
///
/// The dialogue state is tracked via scope locals:
/// - `node`: Current node ID within the dialogue
/// - `speaker`: Entity ID of the NPC speaking
pub struct DialogueSystem;

impl DialogueSystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DialogueSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for DialogueSystem {
    fn id(&self) -> &str {
        "dialogue"
    }

    fn scope_filter(&self) -> &[ScopeKind] {
        // Active when in a Conversation scope
        &[ScopeKind::Conversation]
    }

    fn handles_commands(&self) -> &[&str] {
        &[
            "start_dialogue",
            "dialogue_choice",
            "advance_dialogue",
            "end_dialogue",
        ]
    }

    fn handle_command(
        &self,
        world: &WorldView<'_>,
        command: &Command,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        match command.kind.as_str() {
            "start_dialogue" => {
                let dialogue_id = command
                    .get_str("dialogue")
                    .ok_or_else(|| SystemError::missing_arg("dialogue"))?;

                let speaker = command.get_str("speaker");

                let mut effects = vec![
                    // Push conversation scope
                    Effect::push_scope(ctx.actor.clone(), ScopeKind::Conversation, dialogue_id),
                    // Set initial node
                    Effect::SetScopeLocal {
                        actor: ctx.actor.clone(),
                        key: "node".into(),
                        value: Value::String("start".into()),
                    },
                ];

                // Track speaker if provided
                if let Some(spk) = speaker {
                    effects.push(Effect::SetScopeLocal {
                        actor: ctx.actor.clone(),
                        key: "speaker".into(),
                        value: Value::String(spk.into()),
                    });
                }

                effects.push(Effect::chronicle(
                    "Dialogue Started",
                    format!("Started dialogue: {}", dialogue_id),
                ));

                Ok(effects)
            }

            "dialogue_choice" => {
                // Verify we're in a conversation scope
                let scopes = world
                    .scopes(&ctx.actor)
                    .ok_or_else(|| SystemError::invalid_state("Actor has no scope stack"))?;

                let current = scopes
                    .current()
                    .ok_or_else(|| SystemError::invalid_state("No active scope"))?;

                if current.kind != ScopeKind::Conversation {
                    return Err(SystemError::NotInScope);
                }

                let choice_idx = command
                    .get_int("choice")
                    .ok_or_else(|| SystemError::missing_arg("choice"))?;

                let target_node = command
                    .get_str("target")
                    .ok_or_else(|| SystemError::missing_arg("target"))?;

                // Check if target is "end" (special case to exit dialogue)
                if target_node == "end" {
                    return Ok(vec![
                        Effect::chronicle(
                            "Dialogue Ended",
                            format!("Ended dialogue via choice {}", choice_idx),
                        ),
                        Effect::pop_scope(ctx.actor.clone()),
                    ]);
                }

                // Update current node
                Ok(vec![
                    Effect::SetScopeLocal {
                        actor: ctx.actor.clone(),
                        key: "node".into(),
                        value: Value::String(target_node.into()),
                    },
                    Effect::chronicle(
                        "Dialogue Advanced",
                        format!("Selected choice {} -> {}", choice_idx, target_node),
                    ),
                ])
            }

            "advance_dialogue" => {
                // For dialogues with no choices (just text), advance to next node
                let scopes = world
                    .scopes(&ctx.actor)
                    .ok_or_else(|| SystemError::invalid_state("Actor has no scope stack"))?;

                let current = scopes
                    .current()
                    .ok_or_else(|| SystemError::invalid_state("No active scope"))?;

                if current.kind != ScopeKind::Conversation {
                    return Err(SystemError::NotInScope);
                }

                let target_node = command
                    .get_str("target")
                    .ok_or_else(|| SystemError::missing_arg("target"))?;

                if target_node == "end" {
                    return Ok(vec![
                        Effect::chronicle("Dialogue Ended", "Dialogue concluded."),
                        Effect::pop_scope(ctx.actor.clone()),
                    ]);
                }

                Ok(vec![Effect::SetScopeLocal {
                    actor: ctx.actor.clone(),
                    key: "node".into(),
                    value: Value::String(target_node.into()),
                }])
            }

            "end_dialogue" => {
                // Verify we're in a conversation scope
                let scopes = world
                    .scopes(&ctx.actor)
                    .ok_or_else(|| SystemError::invalid_state("Actor has no scope stack"))?;

                let current = scopes
                    .current()
                    .ok_or_else(|| SystemError::invalid_state("No active scope"))?;

                if current.kind != ScopeKind::Conversation {
                    return Err(SystemError::NotInScope);
                }

                Ok(vec![
                    Effect::chronicle("Dialogue Ended", "Player ended the conversation."),
                    Effect::pop_scope(ctx.actor.clone()),
                ])
            }

            _ => Err(SystemError::UnknownCommand(command.kind.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_primitives::EntityId;
    use engine_world::WorldState;

    #[test]
    fn start_dialogue() {
        let system = DialogueSystem::new();
        let state = WorldState::new();
        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(EntityId::new("actor", "player"), &mut rng);

        let cmd = Command::new("start_dialogue")
            .with_arg("dialogue", Value::String("bob_greeting".into()))
            .with_arg("speaker", Value::String("npc:bob".into()));

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        // Should have: push_scope, set node, set speaker, chronicle
        assert_eq!(effects.len(), 4);

        // First effect should be pushing conversation scope
        match &effects[0] {
            Effect::PushScope { kind, id, .. } => {
                assert_eq!(*kind, ScopeKind::Conversation);
                assert_eq!(id.as_str(), "bob_greeting");
            }
            _ => panic!("Expected PushScope effect"),
        }
    }

    #[test]
    fn dialogue_choice() {
        let system = DialogueSystem::new();
        let mut state = WorldState::new();

        // Set up actor with conversation scope
        let actor = EntityId::new("actor", "player");
        state.entities.spawn(actor.clone());
        let mut scope_stack = engine_world::ScopeStack::new();
        scope_stack.push(engine_world::Scope::new(
            ScopeKind::Conversation,
            "bob_greeting",
        ));
        state.scopes.insert(actor.clone(), scope_stack);

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor, &mut rng);

        let cmd = Command::new("dialogue_choice")
            .with_arg("choice", Value::Int(0))
            .with_arg("target", Value::String("shop".into()));

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        // Should have: set node, chronicle
        assert_eq!(effects.len(), 2);
    }

    #[test]
    fn end_dialogue() {
        let system = DialogueSystem::new();
        let mut state = WorldState::new();

        // Set up actor with conversation scope
        let actor = EntityId::new("actor", "player");
        state.entities.spawn(actor.clone());
        let mut scope_stack = engine_world::ScopeStack::new();
        scope_stack.push(engine_world::Scope::new(
            ScopeKind::Conversation,
            "bob_greeting",
        ));
        state.scopes.insert(actor.clone(), scope_stack);

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("end_dialogue");

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        // Should have: chronicle, pop_scope
        assert_eq!(effects.len(), 2);
        assert!(matches!(&effects[1], Effect::PopScope { .. }));
    }
}
