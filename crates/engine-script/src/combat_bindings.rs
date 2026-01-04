//! Rhai bindings for combat mechanics.
//!
//! These bindings provide script access to health, damage, and invincibility systems.
//!
//! Combat state is stored as entity components:
//! - `health`: Current health points
//! - `max_health`: Maximum health points
//! - `invincible`: Whether entity is currently invincible (bool)
//! - `invincible_timer`: Remaining invincibility time in milliseconds
//!
//! # Example
//!
//! ```rhai
//! let me = current_entity();
//! let target = "enemy:octorok_1";
//!
//! // Check health
//! let hp = get_health(target);
//! if hp > 0 && !is_invincible(target) {
//!     deal_damage(target, 1);
//! }
//!
//! // Heal player
//! if get_health(me) < get_max_health(me) {
//!     heal(me, 1);
//! }
//!
//! // Grant temporary invincibility after taking damage
//! set_invincible(me, 1000);  // 1 second
//! ```

use crate::world_bindings::{WorldEffect, WorldScriptState, WorldSnapshot};
use rhai::{Engine, ImmutableString};
use smol_str::SmolStr;
use std::sync::Arc;

/// Register combat bindings with a Rhai engine.
pub fn register_combat_bindings(
    engine: &mut Engine,
    snapshot: Arc<WorldSnapshot>,
    state: WorldScriptState,
) {
    // === Health Read Functions ===

    // Get current health
    let snap = snapshot.clone();
    engine.register_fn("get_health", move |entity: ImmutableString| -> i64 {
        snap.entities
            .get(entity.as_str())
            .and_then(|e| e.components.get("health"))
            .and_then(|d| d.as_int().ok())
            .unwrap_or(0)
    });

    // Get max health
    let snap = snapshot.clone();
    engine.register_fn("get_max_health", move |entity: ImmutableString| -> i64 {
        snap.entities
            .get(entity.as_str())
            .and_then(|e| e.components.get("max_health"))
            .and_then(|d| d.as_int().ok())
            .unwrap_or(3) // Default max health
    });

    // Check if entity is dead (health <= 0)
    let snap = snapshot.clone();
    engine.register_fn("is_dead", move |entity: ImmutableString| -> bool {
        snap.entities
            .get(entity.as_str())
            .and_then(|e| e.components.get("health"))
            .and_then(|d| d.as_int().ok())
            .map(|hp| hp <= 0)
            .unwrap_or(false)
    });

    // Check if entity is alive (health > 0)
    let snap = snapshot.clone();
    engine.register_fn("is_alive", move |entity: ImmutableString| -> bool {
        snap.entities
            .get(entity.as_str())
            .and_then(|e| e.components.get("health"))
            .and_then(|d| d.as_int().ok())
            .map(|hp| hp > 0)
            .unwrap_or(true) // Default to alive if no health component
    });

    // === Invincibility Read Functions ===

    // Check if entity is invincible
    let snap = snapshot.clone();
    engine.register_fn("is_invincible", move |entity: ImmutableString| -> bool {
        snap.entities
            .get(entity.as_str())
            .and_then(|e| e.components.get("invincible"))
            .and_then(|d| d.as_bool().ok())
            .unwrap_or(false)
    });

    // Get remaining invincibility time
    let snap = snapshot.clone();
    engine.register_fn(
        "invincibility_timer",
        move |entity: ImmutableString| -> i64 {
            snap.entities
                .get(entity.as_str())
                .and_then(|e| e.components.get("invincible_timer"))
                .and_then(|d| d.as_int().ok())
                .unwrap_or(0)
        },
    );

    // === Combat Write Functions ===

    // Deal damage to entity (with optional source for knockback direction)
    let st = state.clone();
    engine.register_fn(
        "deal_damage",
        move |target: ImmutableString, amount: i64| {
            if let Some((kind, id)) = target.split_once(':') {
                st.effects.lock().unwrap().push(WorldEffect::DealDamage {
                    target_kind: SmolStr::new(kind),
                    target_id: SmolStr::new(id),
                    amount,
                    source_kind: None,
                    source_id: None,
                });
            }
        },
    );

    // Deal damage with source (for knockback calculation)
    let st = state.clone();
    engine.register_fn(
        "deal_damage_from",
        move |target: ImmutableString, amount: i64, source: ImmutableString| {
            if let Some((target_kind, target_id)) = target.split_once(':') {
                let (source_kind, source_id) = if let Some((sk, si)) = source.split_once(':') {
                    (Some(SmolStr::new(sk)), Some(SmolStr::new(si)))
                } else {
                    (None, None)
                };

                st.effects.lock().unwrap().push(WorldEffect::DealDamage {
                    target_kind: SmolStr::new(target_kind),
                    target_id: SmolStr::new(target_id),
                    amount,
                    source_kind,
                    source_id,
                });
            }
        },
    );

    // Heal entity
    let st = state.clone();
    engine.register_fn("heal", move |target: ImmutableString, amount: i64| {
        if let Some((kind, id)) = target.split_once(':') {
            st.effects.lock().unwrap().push(WorldEffect::Heal {
                target_kind: SmolStr::new(kind),
                target_id: SmolStr::new(id),
                amount,
            });
        }
    });

    // Set invincibility for duration
    let st = state.clone();
    engine.register_fn(
        "set_invincible",
        move |entity: ImmutableString, duration_ms: i64| {
            if let Some((kind, id)) = entity.split_once(':') {
                st.effects.lock().unwrap().push(WorldEffect::SetInvincible {
                    kind: SmolStr::new(kind),
                    id: SmolStr::new(id),
                    duration_ms: duration_ms.max(0) as u32,
                });
            }
        },
    );

    // === Combat Utility Functions ===

    // Check if entity can take damage (has health, not invincible, not dead)
    let snap = snapshot.clone();
    engine.register_fn("can_take_damage", move |entity: ImmutableString| -> bool {
        let Some(e) = snap.entities.get(entity.as_str()) else {
            return false;
        };

        // Check if invincible
        let invincible = e
            .components
            .get("invincible")
            .and_then(|d| d.as_bool().ok())
            .unwrap_or(false);
        if invincible {
            return false;
        }

        // Check if has health > 0
        let health = e
            .components
            .get("health")
            .and_then(|d| d.as_int().ok())
            .unwrap_or(0);

        health > 0
    });

    // Get health percentage (0.0 to 1.0)
    let snap = snapshot.clone();
    engine.register_fn(
        "health_percentage",
        move |entity: ImmutableString| -> f64 {
            let Some(e) = snap.entities.get(entity.as_str()) else {
                return 0.0;
            };

            let health = e
                .components
                .get("health")
                .and_then(|d| d.as_int().ok())
                .unwrap_or(0);

            let max_health = e
                .components
                .get("max_health")
                .and_then(|d| d.as_int().ok())
                .unwrap_or(3);

            if max_health <= 0 {
                return 0.0;
            }

            (health as f64 / max_health as f64).clamp(0.0, 1.0)
        },
    );

    // Check if entity has "damageable" tag (convention for entities that can take damage)
    let snap = snapshot.clone();
    engine.register_fn("is_damageable", move |entity: ImmutableString| -> bool {
        snap.entities
            .get(entity.as_str())
            .map(|e| e.tags.iter().any(|t| t == "damageable"))
            .unwrap_or(false)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_bindings::EntitySnapshot;
    use rhai::Dynamic;
    use rustc_hash::FxHashMap;

    fn create_test_snapshot() -> Arc<WorldSnapshot> {
        let mut entities = FxHashMap::default();

        // Player with 3 health
        let mut player_components = FxHashMap::default();
        player_components.insert("health".into(), Dynamic::from(3_i64));
        player_components.insert("max_health".into(), Dynamic::from(3_i64));
        player_components.insert("invincible".into(), Dynamic::from(false));

        entities.insert(
            "actor:player".to_string(),
            EntitySnapshot {
                kind: "actor".into(),
                id: "player".into(),
                components: player_components,
                tags: vec!["damageable".into()],
            },
        );

        // Damaged enemy
        let mut enemy_components = FxHashMap::default();
        enemy_components.insert("health".into(), Dynamic::from(1_i64));
        enemy_components.insert("max_health".into(), Dynamic::from(2_i64));

        entities.insert(
            "enemy:octorok".to_string(),
            EntitySnapshot {
                kind: "enemy".into(),
                id: "octorok".into(),
                components: enemy_components,
                tags: vec!["damageable".into(), "hostile".into()],
            },
        );

        // Invincible entity
        let mut invincible_components = FxHashMap::default();
        invincible_components.insert("health".into(), Dynamic::from(1_i64));
        invincible_components.insert("invincible".into(), Dynamic::from(true));
        invincible_components.insert("invincible_timer".into(), Dynamic::from(500_i64));

        entities.insert(
            "enemy:boss".to_string(),
            EntitySnapshot {
                kind: "enemy".into(),
                id: "boss".into(),
                components: invincible_components,
                tags: vec!["damageable".into()],
            },
        );

        Arc::new(WorldSnapshot {
            entities,
            resources: FxHashMap::default(),
            flags: FxHashMap::default(),
        })
    }

    #[test]
    fn health_functions() {
        let snapshot = create_test_snapshot();
        let state = WorldScriptState::new(12345);
        let mut engine = Engine::new();
        register_combat_bindings(&mut engine, snapshot, state);

        let hp: i64 = engine.eval(r#"get_health("actor:player")"#).unwrap();
        assert_eq!(hp, 3);

        let max_hp: i64 = engine.eval(r#"get_max_health("actor:player")"#).unwrap();
        assert_eq!(max_hp, 3);

        let enemy_hp: i64 = engine.eval(r#"get_health("enemy:octorok")"#).unwrap();
        assert_eq!(enemy_hp, 1);

        let pct: f64 = engine.eval(r#"health_percentage("enemy:octorok")"#).unwrap();
        assert!((pct - 0.5).abs() < 0.001);
    }

    #[test]
    fn invincibility_functions() {
        let snapshot = create_test_snapshot();
        let state = WorldScriptState::new(12345);
        let mut engine = Engine::new();
        register_combat_bindings(&mut engine, snapshot, state);

        let player_inv: bool = engine.eval(r#"is_invincible("actor:player")"#).unwrap();
        assert!(!player_inv);

        let boss_inv: bool = engine.eval(r#"is_invincible("enemy:boss")"#).unwrap();
        assert!(boss_inv);

        let timer: i64 = engine.eval(r#"invincibility_timer("enemy:boss")"#).unwrap();
        assert_eq!(timer, 500);
    }

    #[test]
    fn can_take_damage_function() {
        let snapshot = create_test_snapshot();
        let state = WorldScriptState::new(12345);
        let mut engine = Engine::new();
        register_combat_bindings(&mut engine, snapshot, state);

        // Player can take damage
        let player_can: bool = engine.eval(r#"can_take_damage("actor:player")"#).unwrap();
        assert!(player_can);

        // Boss is invincible, cannot take damage
        let boss_can: bool = engine.eval(r#"can_take_damage("enemy:boss")"#).unwrap();
        assert!(!boss_can);
    }

    #[test]
    fn deal_damage_effect() {
        let snapshot = create_test_snapshot();
        let state = WorldScriptState::new(12345);
        let mut engine = Engine::new();
        register_combat_bindings(&mut engine, snapshot, state.clone());

        engine.run(r#"deal_damage("enemy:octorok", 1)"#).unwrap();

        let effects = state.into_effects();
        assert_eq!(effects.len(), 1);
        assert!(matches!(
            &effects[0],
            WorldEffect::DealDamage {
                target_kind,
                target_id,
                amount,
                ..
            } if target_kind == "enemy" && target_id == "octorok" && *amount == 1
        ));
    }

    #[test]
    fn heal_effect() {
        let snapshot = create_test_snapshot();
        let state = WorldScriptState::new(12345);
        let mut engine = Engine::new();
        register_combat_bindings(&mut engine, snapshot, state.clone());

        engine.run(r#"heal("actor:player", 1)"#).unwrap();

        let effects = state.into_effects();
        assert_eq!(effects.len(), 1);
        assert!(matches!(
            &effects[0],
            WorldEffect::Heal {
                target_kind,
                target_id,
                amount,
            } if target_kind == "actor" && target_id == "player" && *amount == 1
        ));
    }
}
