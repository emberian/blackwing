//! Movement system for location/room navigation.
//!
//! Handles:
//! - Moving between locations via exits
//! - Looking at current location
//! - Entering and exiting locations

use engine_primitives::{EntityId, Value};
use engine_world::ScopeKind;

use crate::command::Command;
use crate::context::{SystemContext, WorldView};
use crate::effect::Effect;
use crate::error::SystemError;
use crate::system::System;

/// Component key for actor's current location
const POSITION_COMPONENT: &str = "position";
/// Component key for location's exits (map of direction -> target location)
const EXITS_COMPONENT: &str = "exits";
/// Component key for location's description
const DESCRIPTION_COMPONENT: &str = "description";
/// Component key for location's name
const NAME_COMPONENT: &str = "name";

/// System for handling movement between locations.
///
/// Locations are entities with special components:
/// - `exits`: Map of direction -> EntityId of connected location
/// - `name`: Display name of the location
/// - `description`: Text description of the location
///
/// Actors track their position via:
/// - `position`: EntityId of their current location
///
/// The movement system also manages Location scopes.
pub struct MovementSystem;

impl MovementSystem {
    pub fn new() -> Self {
        Self
    }

    /// Get an actor's current location
    fn get_actor_location(
        world: &WorldView<'_>,
        actor: &EntityId,
    ) -> Option<EntityId> {
        let key = world.entities().key_of(actor)?;
        world
            .entities()
            .get_component(key, POSITION_COMPONENT)?
            .as_entity_ref()
            .cloned()
    }

    /// Get exits from a location
    fn get_location_exits(
        world: &WorldView<'_>,
        location: &EntityId,
    ) -> indexmap::IndexMap<smol_str::SmolStr, EntityId> {
        let Some(key) = world.entities().key_of(location) else {
            return indexmap::IndexMap::new();
        };
        let Some(exits) = world.entities().get_component(key, EXITS_COMPONENT) else {
            return indexmap::IndexMap::new();
        };
        let Some(map) = exits.as_map() else {
            return indexmap::IndexMap::new();
        };

        let mut result = indexmap::IndexMap::new();
        for (dir, val) in map {
            if let Some(entity) = val.as_entity_ref() {
                result.insert(smol_str::SmolStr::new(dir), entity.clone());
            }
        }
        result
    }

    /// Get a location's description
    fn get_location_description(
        world: &WorldView<'_>,
        location: &EntityId,
    ) -> (String, String) {
        let Some(key) = world.entities().key_of(location) else {
            return ("Unknown".to_string(), "You are nowhere.".to_string());
        };

        let name = world
            .entities()
            .get_component(key, NAME_COMPONENT)
            .and_then(|v| v.as_str())
            .unwrap_or(&location.id)
            .to_string();

        let description = world
            .entities()
            .get_component(key, DESCRIPTION_COMPONENT)
            .and_then(|v| v.as_str())
            .unwrap_or("You see nothing special.")
            .to_string();

        (name, description)
    }
}

impl Default for MovementSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for MovementSystem {
    fn id(&self) -> &str {
        "movement"
    }

    fn scope_filter(&self) -> &[ScopeKind] {
        // Movement works in any scope (though "go" might be restricted in combat, etc.)
        &[]
    }

    fn handles_commands(&self) -> &[&str] {
        &["go", "move", "travel", "look", "enter", "exit"]
    }

    fn handle_command(
        &self,
        world: &WorldView<'_>,
        command: &Command,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        match command.kind.as_str() {
            "go" | "move" => {
                let direction = command
                    .get_str("direction")
                    .ok_or_else(|| SystemError::missing_arg("direction"))?;

                // Get actor's current location
                let current_location = Self::get_actor_location(world, &ctx.actor)
                    .ok_or_else(|| SystemError::invalid_state("Actor has no position"))?;

                // Get exits from current location
                let exits = Self::get_location_exits(world, &current_location);

                // Find exit in the specified direction
                let target = exits.get(direction).cloned().ok_or_else(|| {
                    SystemError::invalid_state(format!("No exit to the {}", direction))
                })?;

                // Get target location name for chronicle
                let (target_name, _) = Self::get_location_description(world, &target);

                Ok(vec![
                    // Update actor position
                    Effect::set_component(
                        ctx.actor.clone(),
                        POSITION_COMPONENT,
                        Value::EntityRef(target.clone()),
                    ),
                    // Pop old location scope
                    Effect::pop_scope(ctx.actor.clone()),
                    // Push new location scope
                    Effect::push_scope(
                        ctx.actor.clone(),
                        ScopeKind::Location,
                        target.as_qualified(),
                    ),
                    Effect::chronicle(
                        "Moved",
                        format!("Went {} to {}", direction, target_name),
                    ),
                ])
            }

            "travel" => {
                // Direct travel to a specific location (for fast travel, teleport, etc.)
                let destination = command
                    .get_entity("destination")
                    .ok_or_else(|| SystemError::missing_arg("destination"))?
                    .clone();

                // Verify destination exists
                if world.entities().key_of(&destination).is_none() {
                    return Err(SystemError::invalid_state(format!(
                        "Location {} does not exist",
                        destination.as_qualified()
                    )));
                }

                let (dest_name, _) = Self::get_location_description(world, &destination);

                // Check if actor has a current location (might be first spawn)
                let has_location = Self::get_actor_location(world, &ctx.actor).is_some();

                let mut effects = vec![
                    // Update actor position
                    Effect::set_component(
                        ctx.actor.clone(),
                        POSITION_COMPONENT,
                        Value::EntityRef(destination.clone()),
                    ),
                ];

                // Pop old scope only if we had one
                if has_location {
                    effects.push(Effect::pop_scope(ctx.actor.clone()));
                }

                // Push new location scope
                effects.push(Effect::push_scope(
                    ctx.actor.clone(),
                    ScopeKind::Location,
                    destination.as_qualified(),
                ));

                effects.push(Effect::chronicle(
                    "Traveled",
                    format!("Traveled to {}", dest_name),
                ));

                Ok(effects)
            }

            "look" => {
                // Look at surroundings or a specific target
                let target = command.get_entity("target");

                if let Some(target) = target {
                    // Look at specific entity
                    let Some(key) = world.entities().key_of(target) else {
                        return Err(SystemError::invalid_state(format!(
                            "{} not found",
                            target.as_qualified()
                        )));
                    };

                    let description = world
                        .entities()
                        .get_component(key, DESCRIPTION_COMPONENT)
                        .and_then(|v| v.as_str())
                        .unwrap_or("You see nothing special.");

                    Ok(vec![Effect::chronicle(
                        format!("Looked at {}", target.id),
                        description,
                    )])
                } else {
                    // Look at current location
                    let current_location = Self::get_actor_location(world, &ctx.actor)
                        .ok_or_else(|| SystemError::invalid_state("Actor has no position"))?;

                    let (name, description) =
                        Self::get_location_description(world, &current_location);
                    let exits = Self::get_location_exits(world, &current_location);

                    let exits_text = if exits.is_empty() {
                        "There are no obvious exits.".to_string()
                    } else {
                        let exit_list: Vec<&str> = exits.keys().map(|s| s.as_str()).collect();
                        format!("Exits: {}", exit_list.join(", "))
                    };

                    Ok(vec![Effect::chronicle(
                        name,
                        format!("{}\n\n{}", description, exits_text),
                    )])
                }
            }

            "enter" => {
                // Enter a specific location (e.g., entering a building)
                let location = command
                    .get_entity("location")
                    .ok_or_else(|| SystemError::missing_arg("location"))?
                    .clone();

                // This is essentially the same as travel
                let (name, _) = Self::get_location_description(world, &location);

                let has_location = Self::get_actor_location(world, &ctx.actor).is_some();

                let mut effects = vec![
                    Effect::set_component(
                        ctx.actor.clone(),
                        POSITION_COMPONENT,
                        Value::EntityRef(location.clone()),
                    ),
                ];

                if has_location {
                    effects.push(Effect::pop_scope(ctx.actor.clone()));
                }

                effects.push(Effect::push_scope(
                    ctx.actor.clone(),
                    ScopeKind::Location,
                    location.as_qualified(),
                ));

                effects.push(Effect::chronicle("Entered", format!("Entered {}", name)));

                Ok(effects)
            }

            "exit" => {
                // Exit current location to parent/outside
                // This would typically go to a default exit or require "exit to X"
                let destination = command.get_entity("destination");

                if let Some(dest) = destination {
                    // Exit to specific destination
                    let (name, _) = Self::get_location_description(world, dest);

                    Ok(vec![
                        Effect::set_component(
                            ctx.actor.clone(),
                            POSITION_COMPONENT,
                            Value::EntityRef(dest.clone()),
                        ),
                        Effect::pop_scope(ctx.actor.clone()),
                        Effect::push_scope(
                            ctx.actor.clone(),
                            ScopeKind::Location,
                            dest.as_qualified(),
                        ),
                        Effect::chronicle("Exited", format!("Exited to {}", name)),
                    ])
                } else {
                    // Look for an "out" or "exit" direction
                    let current_location = Self::get_actor_location(world, &ctx.actor)
                        .ok_or_else(|| SystemError::invalid_state("Actor has no position"))?;

                    let exits = Self::get_location_exits(world, &current_location);

                    // Try common exit directions
                    let exit_dest = exits
                        .get("out")
                        .or_else(|| exits.get("exit"))
                        .or_else(|| exits.get("outside"))
                        .cloned()
                        .ok_or_else(|| SystemError::invalid_state("No obvious exit"))?;

                    let (name, _) = Self::get_location_description(world, &exit_dest);

                    Ok(vec![
                        Effect::set_component(
                            ctx.actor.clone(),
                            POSITION_COMPONENT,
                            Value::EntityRef(exit_dest.clone()),
                        ),
                        Effect::pop_scope(ctx.actor.clone()),
                        Effect::push_scope(
                            ctx.actor.clone(),
                            ScopeKind::Location,
                            exit_dest.as_qualified(),
                        ),
                        Effect::chronicle("Exited", format!("Exited to {}", name)),
                    ])
                }
            }

            _ => Err(SystemError::UnknownCommand(command.kind.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_world::WorldState;

    fn setup_test_world() -> (WorldState, EntityId, EntityId, EntityId) {
        let mut state = WorldState::new();

        // Create two locations
        let market = EntityId::new("location", "market");
        let market_key = state.entities.spawn(market.clone());
        state.entities.set_component(
            market_key,
            NAME_COMPONENT,
            Value::String("Market Square".into()),
        );
        state.entities.set_component(
            market_key,
            DESCRIPTION_COMPONENT,
            Value::String("A bustling marketplace.".into()),
        );

        let tavern = EntityId::new("location", "tavern");
        let tavern_key = state.entities.spawn(tavern.clone());
        state.entities.set_component(
            tavern_key,
            NAME_COMPONENT,
            Value::String("The Golden Dragon".into()),
        );
        state.entities.set_component(
            tavern_key,
            DESCRIPTION_COMPONENT,
            Value::String("A cozy tavern with a warm fire.".into()),
        );

        // Set up exits
        let mut market_exits = std::collections::HashMap::new();
        market_exits.insert("north".to_string(), Value::EntityRef(tavern.clone()));
        state.entities.set_component(
            market_key,
            EXITS_COMPONENT,
            Value::Map(market_exits),
        );

        let mut tavern_exits = std::collections::HashMap::new();
        tavern_exits.insert("south".to_string(), Value::EntityRef(market.clone()));
        state.entities.set_component(
            tavern_key,
            EXITS_COMPONENT,
            Value::Map(tavern_exits),
        );

        // Create actor at market
        let actor = EntityId::new("actor", "player");
        let actor_key = state.entities.spawn(actor.clone());
        state.entities.set_component(
            actor_key,
            POSITION_COMPONENT,
            Value::EntityRef(market.clone()),
        );
        state.scopes.insert(
            actor.clone(),
            engine_world::ScopeStack::new(),
        );
        state.scopes.get_mut(&actor).unwrap().push(
            engine_world::Scope::new(ScopeKind::Location, "location:market"),
        );

        (state, actor, market, tavern)
    }

    #[test]
    fn go_direction() {
        let system = MovementSystem::new();
        let (state, actor, _market, tavern) = setup_test_world();

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("go")
            .with_arg("direction", Value::String("north".into()));

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        // Should have: set position, pop scope, push scope, chronicle
        assert_eq!(effects.len(), 4);

        match &effects[0] {
            Effect::SetComponent { component, value, .. } => {
                assert_eq!(component.as_str(), POSITION_COMPONENT);
                assert_eq!(value.as_entity_ref(), Some(&tavern));
            }
            _ => panic!("Expected SetComponent effect"),
        }
    }

    #[test]
    fn go_invalid_direction() {
        let system = MovementSystem::new();
        let (state, actor, _market, _tavern) = setup_test_world();

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("go")
            .with_arg("direction", Value::String("east".into()));

        let result = system.handle_command(&view, &cmd, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn look_at_location() {
        let system = MovementSystem::new();
        let (state, actor, _market, _tavern) = setup_test_world();

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("look");

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        assert_eq!(effects.len(), 1);
        match &effects[0] {
            Effect::Chronicle { title, description } => {
                assert_eq!(title.as_str(), "Market Square");
                assert!(description.contains("bustling marketplace"));
                assert!(description.contains("north"));
            }
            _ => panic!("Expected Chronicle effect"),
        }
    }

    #[test]
    fn travel_directly() {
        let system = MovementSystem::new();
        let (state, actor, _market, tavern) = setup_test_world();

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("travel")
            .with_arg("destination", Value::EntityRef(tavern.clone()));

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        // Should have: set position, pop scope, push scope, chronicle
        assert_eq!(effects.len(), 4);
    }
}
