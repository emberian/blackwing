//! Inventory system for item management.
//!
//! Handles:
//! - Taking and dropping items
//! - Using and equipping items
//! - Transferring items between entities
//! - Container management

use engine_primitives::{EntityId, Value};
use engine_world::ScopeKind;

use crate::command::Command;
use crate::context::{SystemContext, WorldView};
use crate::effect::Effect;
use crate::error::SystemError;
use crate::system::System;

/// Component key for storing inventory (list of EntityIds)
const INVENTORY_COMPONENT: &str = "inventory";
/// Component key for equipped items
const EQUIPPED_COMPONENT: &str = "equipped";
/// Component key for item location (which entity contains the item)
const CONTAINER_COMPONENT: &str = "container";
/// Component key for actor's current location
const POSITION_COMPONENT: &str = "position";

/// System for managing inventories, items, and containers.
///
/// Inventory is tracked via entity components:
/// - `inventory`: Array of EntityIds the entity is carrying
/// - `equipped`: Map of slot -> EntityId for equipped items
/// - `container`: EntityId of what contains this item (location or actor)
pub struct InventorySystem;

impl InventorySystem {
    pub fn new() -> Self {
        Self
    }

    /// Get an entity's inventory as a Vec of EntityIds
    fn get_inventory(
        world: &WorldView<'_>,
        entity: &EntityId,
    ) -> Vec<EntityId> {
        let Some(key) = world.entities().key_of(entity) else {
            return vec![];
        };
        let Some(inv) = world.entities().get_component(key, INVENTORY_COMPONENT) else {
            return vec![];
        };
        let Some(arr) = inv.as_array() else {
            return vec![];
        };
        arr.iter()
            .filter_map(|v| v.as_entity_ref().cloned())
            .collect()
    }

    /// Check if an item exists and is at the actor's location
    fn item_is_at_actor_location(
        world: &WorldView<'_>,
        actor: &EntityId,
        item: &EntityId,
    ) -> bool {
        // Get actor's location
        let Some(actor_key) = world.entities().key_of(actor) else {
            return false;
        };
        let Some(actor_pos) = world.entities().get_component(actor_key, POSITION_COMPONENT) else {
            return false;
        };

        // Get item's container
        let Some(item_key) = world.entities().key_of(item) else {
            return false;
        };
        let Some(item_container) = world.entities().get_component(item_key, CONTAINER_COMPONENT) else {
            return false;
        };

        // Compare: item's container should match actor's position
        actor_pos == item_container
    }

    /// Check if an actor has an item in their inventory
    fn actor_has_item(
        world: &WorldView<'_>,
        actor: &EntityId,
        item: &EntityId,
    ) -> bool {
        Self::get_inventory(world, actor)
            .iter()
            .any(|i| i == item)
    }
}

impl Default for InventorySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for InventorySystem {
    fn id(&self) -> &str {
        "inventory"
    }

    fn scope_filter(&self) -> &[ScopeKind] {
        // Inventory commands work in any scope
        &[]
    }

    fn handles_commands(&self) -> &[&str] {
        &["take", "drop", "give", "use_item", "equip", "unequip", "examine"]
    }

    fn handle_command(
        &self,
        world: &WorldView<'_>,
        command: &Command,
        ctx: &mut SystemContext<'_>,
    ) -> Result<Vec<Effect>, SystemError> {
        match command.kind.as_str() {
            "take" => {
                let item = command
                    .get_entity("item")
                    .ok_or_else(|| SystemError::missing_arg("item"))?
                    .clone();

                // Verify item exists
                if world.entities().key_of(&item).is_none() {
                    return Err(SystemError::invalid_state(format!(
                        "Item {} does not exist",
                        item.as_qualified()
                    )));
                }

                // Verify item is at actor's location
                if !Self::item_is_at_actor_location(world, &ctx.actor, &item) {
                    return Err(SystemError::invalid_state(
                        "Item is not at your location"
                    ));
                }

                // Get current inventory
                let mut inventory = Self::get_inventory(world, &ctx.actor);
                inventory.push(item.clone());

                let inventory_value = Value::Array(
                    inventory.into_iter().map(Value::EntityRef).collect()
                );

                Ok(vec![
                    // Update actor's inventory
                    Effect::set_component(
                        ctx.actor.clone(),
                        INVENTORY_COMPONENT,
                        inventory_value,
                    ),
                    // Update item's container to be the actor
                    Effect::set_component(
                        item.clone(),
                        CONTAINER_COMPONENT,
                        Value::EntityRef(ctx.actor.clone()),
                    ),
                    Effect::chronicle(
                        "Item Taken",
                        format!("Picked up {}", item.id),
                    ),
                ])
            }

            "drop" => {
                let item = command
                    .get_entity("item")
                    .ok_or_else(|| SystemError::missing_arg("item"))?
                    .clone();

                // Verify actor has the item
                if !Self::actor_has_item(world, &ctx.actor, &item) {
                    return Err(SystemError::invalid_state(format!(
                        "You don't have {}",
                        item.as_qualified()
                    )));
                }

                // Get actor's current location
                let actor_key = world.entities().key_of(&ctx.actor)
                    .ok_or_else(|| SystemError::invalid_state("Actor not found"))?;
                let actor_pos = world
                    .entities()
                    .get_component(actor_key, POSITION_COMPONENT)
                    .cloned()
                    .unwrap_or(Value::Null);

                // Remove from inventory
                let mut inventory = Self::get_inventory(world, &ctx.actor);
                inventory.retain(|i| i != &item);

                let inventory_value = Value::Array(
                    inventory.into_iter().map(Value::EntityRef).collect()
                );

                Ok(vec![
                    // Update actor's inventory
                    Effect::set_component(
                        ctx.actor.clone(),
                        INVENTORY_COMPONENT,
                        inventory_value,
                    ),
                    // Update item's container to actor's location
                    Effect::set_component(
                        item.clone(),
                        CONTAINER_COMPONENT,
                        actor_pos,
                    ),
                    Effect::chronicle(
                        "Item Dropped",
                        format!("Dropped {}", item.id),
                    ),
                ])
            }

            "give" => {
                let item = command
                    .get_entity("item")
                    .ok_or_else(|| SystemError::missing_arg("item"))?
                    .clone();
                let target = command
                    .get_entity("target")
                    .ok_or_else(|| SystemError::missing_arg("target"))?
                    .clone();

                // Verify actor has the item
                if !Self::actor_has_item(world, &ctx.actor, &item) {
                    return Err(SystemError::invalid_state(format!(
                        "You don't have {}",
                        item.as_qualified()
                    )));
                }

                // Remove from actor's inventory
                let mut actor_inventory = Self::get_inventory(world, &ctx.actor);
                actor_inventory.retain(|i| i != &item);

                // Add to target's inventory
                let mut target_inventory = Self::get_inventory(world, &target);
                target_inventory.push(item.clone());

                let actor_inv_value = Value::Array(
                    actor_inventory.into_iter().map(Value::EntityRef).collect()
                );
                let target_inv_value = Value::Array(
                    target_inventory.into_iter().map(Value::EntityRef).collect()
                );

                Ok(vec![
                    Effect::set_component(
                        ctx.actor.clone(),
                        INVENTORY_COMPONENT,
                        actor_inv_value,
                    ),
                    Effect::set_component(
                        target.clone(),
                        INVENTORY_COMPONENT,
                        target_inv_value,
                    ),
                    Effect::set_component(
                        item.clone(),
                        CONTAINER_COMPONENT,
                        Value::EntityRef(target.clone()),
                    ),
                    Effect::chronicle(
                        "Item Given",
                        format!("Gave {} to {}", item.id, target.id),
                    ),
                ])
            }

            "use_item" => {
                let item = command
                    .get_entity("item")
                    .ok_or_else(|| SystemError::missing_arg("item"))?
                    .clone();

                // Verify actor has the item
                if !Self::actor_has_item(world, &ctx.actor, &item) {
                    return Err(SystemError::invalid_state(format!(
                        "You don't have {}",
                        item.as_qualified()
                    )));
                }

                // The actual "use" effect would be determined by the item's on_use script
                // For now, just log the action
                Ok(vec![
                    Effect::chronicle(
                        "Item Used",
                        format!("Used {}", item.id),
                    ),
                    // Run the item's use script (if any)
                    // In a full implementation, we'd look up the item template
                    // and execute its on_use script
                ])
            }

            "equip" => {
                let item = command
                    .get_entity("item")
                    .ok_or_else(|| SystemError::missing_arg("item"))?
                    .clone();
                let slot = command
                    .get_str("slot")
                    .unwrap_or("main_hand");

                // Verify actor has the item
                if !Self::actor_has_item(world, &ctx.actor, &item) {
                    return Err(SystemError::invalid_state(format!(
                        "You don't have {}",
                        item.as_qualified()
                    )));
                }

                // Get current equipped map
                let actor_key = world.entities().key_of(&ctx.actor)
                    .ok_or_else(|| SystemError::invalid_state("Actor not found"))?;
                let mut equipped = world
                    .entities()
                    .get_component(actor_key, EQUIPPED_COMPONENT)
                    .and_then(|v| v.as_map())
                    .cloned()
                    .unwrap_or_default();

                // Equip the item
                equipped.insert(slot.into(), Value::EntityRef(item.clone()));

                Ok(vec![
                    Effect::set_component(
                        ctx.actor.clone(),
                        EQUIPPED_COMPONENT,
                        Value::Map(equipped),
                    ),
                    Effect::chronicle(
                        "Item Equipped",
                        format!("Equipped {} in {}", item.id, slot),
                    ),
                ])
            }

            "unequip" => {
                let slot = command
                    .get_str("slot")
                    .ok_or_else(|| SystemError::missing_arg("slot"))?;

                // Get current equipped map
                let actor_key = world.entities().key_of(&ctx.actor)
                    .ok_or_else(|| SystemError::invalid_state("Actor not found"))?;
                let mut equipped = world
                    .entities()
                    .get_component(actor_key, EQUIPPED_COMPONENT)
                    .and_then(|v| v.as_map())
                    .cloned()
                    .unwrap_or_default();

                // Check if slot has an item
                if !equipped.contains_key(slot) {
                    return Err(SystemError::invalid_state(format!(
                        "Nothing equipped in slot {}",
                        slot
                    )));
                }

                equipped.remove(slot);

                Ok(vec![
                    Effect::set_component(
                        ctx.actor.clone(),
                        EQUIPPED_COMPONENT,
                        Value::Map(equipped),
                    ),
                    Effect::chronicle(
                        "Item Unequipped",
                        format!("Unequipped item from {}", slot),
                    ),
                ])
            }

            "examine" => {
                let item = command
                    .get_entity("item")
                    .ok_or_else(|| SystemError::missing_arg("item"))?
                    .clone();

                // Just log that we examined it
                // In a full implementation, we'd return the item's description
                Ok(vec![
                    Effect::chronicle(
                        "Examined Item",
                        format!("Examined {}", item.id),
                    ),
                ])
            }

            _ => Err(SystemError::UnknownCommand(command.kind.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_world::WorldState;

    #[test]
    fn take_item() {
        let system = InventorySystem::new();
        let mut state = WorldState::new();

        // Create actor at a location
        let actor = EntityId::new("actor", "player");
        let actor_key = state.entities.spawn(actor.clone());
        let location = EntityId::new("location", "market");
        state.entities.set_component(
            actor_key,
            POSITION_COMPONENT,
            Value::EntityRef(location.clone()),
        );

        // Create item at the same location
        let item = EntityId::new("item", "sword");
        let item_key = state.entities.spawn(item.clone());
        state.entities.set_component(
            item_key,
            CONTAINER_COMPONENT,
            Value::EntityRef(location),
        );

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("take")
            .with_arg("item", Value::EntityRef(item.clone()));

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        // Should have: set actor inventory, set item container, chronicle
        assert_eq!(effects.len(), 3);

        match &effects[0] {
            Effect::SetComponent { component, value, .. } => {
                assert_eq!(component.as_str(), INVENTORY_COMPONENT);
                let arr = value.as_array().unwrap();
                assert_eq!(arr.len(), 1);
            }
            _ => panic!("Expected SetComponent effect"),
        }
    }

    #[test]
    fn drop_item() {
        let system = InventorySystem::new();
        let mut state = WorldState::new();

        // Create actor with item in inventory
        let actor = EntityId::new("actor", "player");
        let actor_key = state.entities.spawn(actor.clone());
        let location = EntityId::new("location", "market");
        state.entities.set_component(
            actor_key,
            POSITION_COMPONENT,
            Value::EntityRef(location.clone()),
        );

        let item = EntityId::new("item", "sword");
        state.entities.spawn(item.clone());
        state.entities.set_component(
            actor_key,
            INVENTORY_COMPONENT,
            Value::Array(vec![Value::EntityRef(item.clone())]),
        );

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("drop")
            .with_arg("item", Value::EntityRef(item.clone()));

        let effects = system.handle_command(&view, &cmd, &mut ctx).unwrap();

        // Should have: set actor inventory (empty), set item container, chronicle
        assert_eq!(effects.len(), 3);
    }

    #[test]
    fn cannot_take_item_not_at_location() {
        let system = InventorySystem::new();
        let mut state = WorldState::new();

        // Create actor at one location
        let actor = EntityId::new("actor", "player");
        let actor_key = state.entities.spawn(actor.clone());
        let location1 = EntityId::new("location", "market");
        state.entities.set_component(
            actor_key,
            POSITION_COMPONENT,
            Value::EntityRef(location1),
        );

        // Create item at different location
        let item = EntityId::new("item", "sword");
        let item_key = state.entities.spawn(item.clone());
        let location2 = EntityId::new("location", "tavern");
        state.entities.set_component(
            item_key,
            CONTAINER_COMPONENT,
            Value::EntityRef(location2),
        );

        let view = WorldView::new(&state);
        let mut rng = engine_primitives::Rng::new(12345);
        let mut ctx = SystemContext::new(actor.clone(), &mut rng);

        let cmd = Command::new("take")
            .with_arg("item", Value::EntityRef(item.clone()));

        let result = system.handle_command(&view, &cmd, &mut ctx);
        assert!(result.is_err());
    }
}
