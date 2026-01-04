//! Script execution for entity ticks.
//!
//! This module provides infrastructure for executing Rhai scripts during the game loop,
//! including entity on_tick scripts.

use blackwing_core::{Effect, EntityId, FlagId, ResourceId, Value, WorldState};
use engine_script::{
    rhai::{Engine, Scope, AST},
    register_input_bindings, register_tilemap_bindings, register_world_bindings, InputSnapshot,
    TilemapSnapshot, WorldEffect, WorldScriptState, WorldSnapshot,
};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use std::sync::Arc;

/// Holds compiled scripts and execution state.
pub struct ScriptRunner {
    /// Rhai engine for script execution.
    engine: Engine,
    /// Compiled on_tick scripts keyed by entity template ID.
    tick_scripts: FxHashMap<SmolStr, AST>,
    /// Script execution state (for collecting effects).
    script_state: WorldScriptState,
    /// RNG seed for deterministic script execution.
    rng_seed: u64,
}

impl ScriptRunner {
    /// Create a new script runner.
    pub fn new(rng_seed: u64) -> Self {
        let mut engine = Engine::new();

        // Set safety limits
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(100_000);
        engine.set_max_string_size(10_000);

        Self {
            engine,
            tick_scripts: FxHashMap::default(),
            script_state: WorldScriptState::new(rng_seed),
            rng_seed,
        }
    }

    /// Register an on_tick script for an entity template.
    pub fn register_tick_script(
        &mut self,
        template_id: impl Into<SmolStr>,
        script_source: &str,
    ) -> Result<(), String> {
        let template_id = template_id.into();
        let ast = self
            .engine
            .compile(script_source)
            .map_err(|e| format!("Failed to compile script for {}: {}", template_id, e))?;
        self.tick_scripts.insert(template_id, ast);
        Ok(())
    }

    /// Execute on_tick scripts for all entities that have them.
    ///
    /// Returns a list of effects to apply to the world state.
    pub fn run_entity_ticks(
        &mut self,
        world_state: &WorldState,
        input: &InputSnapshot,
        tilemap: Option<&TilemapSnapshot>,
        delta_ms: u32,
        entity_templates: &FxHashMap<EntityId, SmolStr>, // entity -> template_id
    ) -> Vec<Effect> {
        // Reset script state for this frame
        self.script_state = WorldScriptState::new(self.rng_seed);
        self.script_state.set_delta_ms(delta_ms);

        // Create snapshots
        let world_snapshot = Arc::new(WorldSnapshot::from_world(world_state));
        let input_snapshot = Arc::new(input.clone());

        let mut all_effects = Vec::new();

        // Iterate over entities and run their tick scripts
        for (_key, entity) in world_state.entities.iter() {
            // Look up template ID for this entity
            let template_id = match entity_templates.get(&entity.id) {
                Some(tid) => tid,
                None => continue,
            };

            // Check if this template has a tick script
            let ast = match self.tick_scripts.get(template_id) {
                Some(ast) => ast,
                None => continue,
            };

            // Set current entity context
            self.script_state
                .set_entity_context(Some(entity.id.as_qualified()));

            // Create a fresh engine scope with bindings
            let mut scope = Scope::new();

            // Register bindings (we need to do this for each entity because the context changes)
            // Note: In a production system, we'd optimize this to avoid re-registering
            let mut tick_engine = Engine::new();
            tick_engine.set_max_expr_depths(64, 64);
            tick_engine.set_max_operations(100_000);

            register_world_bindings(
                &mut tick_engine,
                world_snapshot.clone(),
                self.script_state.clone(),
            );
            register_input_bindings(&mut tick_engine, input_snapshot.clone());

            if let Some(tilemap) = tilemap {
                register_tilemap_bindings(&mut tick_engine, Arc::new(tilemap.clone()));
            }

            // Execute the script
            // The script should define an on_tick(delta_ms) function
            match tick_engine.call_fn::<()>(&mut scope, ast, "on_tick", (delta_ms as i64,)) {
                Ok(()) => {}
                Err(e) => {
                    // Log error but continue with other entities
                    eprintln!(
                        "Error in on_tick for {}: {}",
                        entity.id.as_qualified(),
                        e
                    );
                }
            }
        }

        // Collect effects from script execution
        let world_effects = self.script_state.clone().into_effects();

        // Convert WorldEffect to Effect
        for we in world_effects {
            if let Some(effect) = world_effect_to_effect(we) {
                all_effects.push(effect);
            }
        }

        all_effects
    }

    /// Run a global tick script (not tied to any entity).
    pub fn run_global_tick(
        &mut self,
        script_source: &str,
        world_state: &WorldState,
        input: &InputSnapshot,
        tilemap: Option<&TilemapSnapshot>,
        delta_ms: u32,
    ) -> Result<Vec<Effect>, String> {
        // Reset script state
        self.script_state = WorldScriptState::new(self.rng_seed);
        self.script_state.set_delta_ms(delta_ms);
        self.script_state.set_entity_context(None);

        // Create snapshots
        let world_snapshot = Arc::new(WorldSnapshot::from_world(world_state));
        let input_snapshot = Arc::new(input.clone());

        // Create engine with bindings
        let mut tick_engine = Engine::new();
        tick_engine.set_max_expr_depths(64, 64);
        tick_engine.set_max_operations(100_000);

        register_world_bindings(
            &mut tick_engine,
            world_snapshot,
            self.script_state.clone(),
        );
        register_input_bindings(&mut tick_engine, input_snapshot);

        if let Some(tilemap) = tilemap {
            register_tilemap_bindings(&mut tick_engine, Arc::new(tilemap.clone()));
        }

        // Compile and run
        let ast = tick_engine
            .compile(script_source)
            .map_err(|e| format!("Failed to compile global tick script: {}", e))?;

        let mut scope = Scope::new();
        tick_engine
            .call_fn::<()>(&mut scope, &ast, "on_game_tick", (delta_ms as i64,))
            .map_err(|e| format!("Error in on_game_tick: {}", e))?;

        // Collect and convert effects
        let world_effects = self.script_state.clone().into_effects();
        let effects = world_effects
            .into_iter()
            .filter_map(world_effect_to_effect)
            .collect();

        Ok(effects)
    }
}

/// Convert a WorldEffect to a blackwing-core Effect.
fn world_effect_to_effect(we: WorldEffect) -> Option<Effect> {
    match we {
        WorldEffect::SpawnEntity { kind, id } => {
            Some(Effect::spawn(EntityId::new(kind.as_str(), id.as_str())))
        }
        WorldEffect::DespawnEntity { kind, id } => {
            Some(Effect::despawn(EntityId::new(kind.as_str(), id.as_str())))
        }
        WorldEffect::SetComponent {
            kind,
            id,
            component,
            value,
        } => Some(Effect::set_component(
            EntityId::new(kind.as_str(), id.as_str()),
            component,
            value,
        )),
        WorldEffect::RemoveComponent {
            kind,
            id,
            component,
        } => Some(Effect::RemoveComponent {
            entity: EntityId::new(kind.as_str(), id.as_str()),
            component,
        }),
        WorldEffect::AddTag { kind, id, tag } => Some(Effect::AddTag {
            entity: EntityId::new(kind.as_str(), id.as_str()),
            tag,
        }),
        WorldEffect::RemoveTag { kind, id, tag } => Some(Effect::RemoveTag {
            entity: EntityId::new(kind.as_str(), id.as_str()),
            tag,
        }),
        WorldEffect::SetResource { resource, value } => Some(Effect::SetResource {
            resource: ResourceId::new(resource.as_str()),
            value,
        }),
        WorldEffect::ModifyResource { resource, delta } => {
            Some(Effect::modify_resource(ResourceId::new(resource.as_str()), delta))
        }
        WorldEffect::SetFlag { flag, value } => Some(Effect::set_flag(FlagId::new(flag.as_str()), value)),
        WorldEffect::ClearFlag { flag } => Some(Effect::ClearFlag {
            flag: FlagId::new(flag.as_str()),
        }),
        WorldEffect::PushScope { .. } => {
            // Push scope requires an actor - for now we skip
            None
        }
        WorldEffect::PopScope => None,
        WorldEffect::Chronicle { title, description } => {
            Some(Effect::chronicle(title, description))
        }
        WorldEffect::SetPosition { kind, id, x, y } => Some(Effect::set_position(
            EntityId::new(kind.as_str(), id.as_str()),
            x,
            y,
        )),
        WorldEffect::MoveBy { kind, id, dx, dy } => Some(Effect::move_by(
            EntityId::new(kind.as_str(), id.as_str()),
            dx,
            dy,
        )),
        WorldEffect::ChangeRoom {
            room_id,
            spawn_x,
            spawn_y,
        } => Some(Effect::change_room(room_id, spawn_x, spawn_y)),
        WorldEffect::SpawnAtPosition { kind, id, x, y } => Some(Effect::spawn_at_position(
            EntityId::new(kind.as_str(), id.as_str()),
            x,
            y,
        )),
        WorldEffect::DealDamage {
            target_kind,
            target_id,
            amount,
            source_kind,
            source_id,
        } => {
            let source = match (source_kind, source_id) {
                (Some(sk), Some(si)) => Some(EntityId::new(sk.as_str(), si.as_str())),
                _ => None,
            };
            Some(Effect::deal_damage(
                EntityId::new(target_kind.as_str(), target_id.as_str()),
                amount,
                source,
            ))
        }
        WorldEffect::Heal {
            target_kind,
            target_id,
            amount,
        } => Some(Effect::heal(
            EntityId::new(target_kind.as_str(), target_id.as_str()),
            amount,
        )),
        WorldEffect::SetInvincible {
            kind,
            id,
            duration_ms,
        } => Some(Effect::set_invincible(
            EntityId::new(kind.as_str(), id.as_str()),
            duration_ms,
        )),
        WorldEffect::SpawnHitbox {
            id,
            x,
            y,
            width,
            height,
            damage,
            lifetime_ms,
            owner,
        } => {
            // SpawnHitbox is a compound effect - we emit a batch of effects
            let entity_id = EntityId::new("hitbox", id.as_str());
            let mut effects = vec![
                Effect::spawn(entity_id.clone()),
                Effect::set_component(
                    entity_id.clone(),
                    "position_x",
                    Value::Float(x.into()),
                ),
                Effect::set_component(
                    entity_id.clone(),
                    "position_y",
                    Value::Float(y.into()),
                ),
                Effect::set_component(
                    entity_id.clone(),
                    "hitbox_width",
                    Value::Float(width.into()),
                ),
                Effect::set_component(
                    entity_id.clone(),
                    "hitbox_height",
                    Value::Float(height.into()),
                ),
                Effect::set_component(entity_id.clone(), "damage", Value::Int(damage)),
                Effect::set_component(entity_id.clone(), "lifetime", Value::Int(lifetime_ms)),
                Effect::AddTag {
                    entity: entity_id.clone(),
                    tag: "hitbox".into(),
                },
                Effect::AddTag {
                    entity: entity_id.clone(),
                    tag: "damage_zone".into(),
                },
            ];

            if let Some(owner_id) = owner {
                effects.push(Effect::set_component(
                    entity_id,
                    "owner",
                    Value::String(owner_id),
                ));
            }

            Some(Effect::batch(effects))
        }
        WorldEffect::SpawnProjectile {
            id,
            x,
            y,
            direction,
            speed,
            damage,
            lifetime_ms,
            owner,
        } => {
            // SpawnProjectile creates an entity with projectile components and tags
            let entity_id = EntityId::new("projectile", id.as_str());
            let mut effects = vec![
                Effect::spawn(entity_id.clone()),
                Effect::set_component(
                    entity_id.clone(),
                    "position_x",
                    Value::Float(x.into()),
                ),
                Effect::set_component(
                    entity_id.clone(),
                    "position_y",
                    Value::Float(y.into()),
                ),
                Effect::set_component(
                    entity_id.clone(),
                    "direction",
                    Value::String(direction),
                ),
                Effect::set_component(
                    entity_id.clone(),
                    "speed",
                    Value::Float(speed.into()),
                ),
                Effect::set_component(entity_id.clone(), "damage", Value::Int(damage)),
                Effect::set_component(entity_id.clone(), "lifetime", Value::Int(lifetime_ms)),
                // Default hitbox for projectiles
                Effect::set_component(
                    entity_id.clone(),
                    "hitbox_width",
                    Value::Float(8.0),
                ),
                Effect::set_component(
                    entity_id.clone(),
                    "hitbox_height",
                    Value::Float(8.0),
                ),
                Effect::AddTag {
                    entity: entity_id.clone(),
                    tag: "projectile".into(),
                },
                Effect::AddTag {
                    entity: entity_id.clone(),
                    tag: "damage_zone".into(),
                },
            ];

            if let Some(owner_id) = owner {
                effects.push(Effect::set_component(
                    entity_id,
                    "owner",
                    Value::String(owner_id),
                ));
            }

            Some(Effect::batch(effects))
        }
    }
}
