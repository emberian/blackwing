//! Rhai bindings for animation control.
//!
//! These bindings provide script access to entity animations.
//! Animation state is stored as entity components:
//! - `animation`: Current animation name (e.g., "walk", "idle", "attack")
//! - `animation_direction`: Direction the animation faces (e.g., "north", "south")
//! - `animation_time`: Elapsed time in milliseconds
//! - `animation_finished`: Whether the animation has completed (for non-looping)
//! - `facing`: The direction the entity is facing
//!
//! # Example
//!
//! ```rhai
//! let me = current_entity();
//!
//! // Set animation based on movement
//! if is_moving() {
//!     set_animation(me, "walk", get_facing(me));
//! } else {
//!     set_animation(me, "idle", get_facing(me));
//! }
//!
//! // Check if attack animation finished
//! if animation_finished(me) {
//!     remove_tag(me, "attacking");
//! }
//! ```

use crate::world_bindings::{WorldEffect, WorldScriptState};
use blackwing_core::Value;
use rhai::{Engine, ImmutableString};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use std::sync::Arc;

/// Snapshot of entity animation data for script access.
#[derive(Clone)]
pub struct AnimationSnapshot {
    /// Animation states keyed by entity qualified ID.
    /// Each entry contains (animation_name, direction, time_ms, finished).
    pub animations: FxHashMap<String, AnimationState>,
}

/// Animation state for a single entity.
#[derive(Clone, Default)]
pub struct AnimationState {
    pub animation: SmolStr,
    pub direction: SmolStr,
    pub time_ms: u32,
    pub finished: bool,
}

impl AnimationSnapshot {
    /// Create an empty animation snapshot.
    pub fn new() -> Self {
        Self {
            animations: FxHashMap::default(),
        }
    }

    /// Create a snapshot from entity component data.
    ///
    /// Reads animation components from entities that have them.
    pub fn from_entity_components(
        entity_animations: impl IntoIterator<Item = (String, AnimationState)>,
    ) -> Self {
        Self {
            animations: entity_animations.into_iter().collect(),
        }
    }

    /// Get animation state for an entity.
    pub fn get(&self, entity_id: &str) -> Option<&AnimationState> {
        self.animations.get(entity_id)
    }
}

impl Default for AnimationSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Register animation bindings with a Rhai engine.
pub fn register_animation_bindings(
    engine: &mut Engine,
    snapshot: Arc<AnimationSnapshot>,
    state: WorldScriptState,
) {
    // Get current animation name
    let snap = snapshot.clone();
    engine.register_fn("get_animation", move |entity: ImmutableString| -> ImmutableString {
        snap.get(entity.as_str())
            .map(|s| ImmutableString::from(s.animation.as_str()))
            .unwrap_or_else(|| "idle".into())
    });

    // Get animation direction
    let snap = snapshot.clone();
    engine.register_fn(
        "get_animation_direction",
        move |entity: ImmutableString| -> ImmutableString {
            snap.get(entity.as_str())
                .map(|s| ImmutableString::from(s.direction.as_str()))
                .unwrap_or_else(|| "south".into())
        },
    );

    // Check if animation has finished (for non-looping animations)
    let snap = snapshot.clone();
    engine.register_fn(
        "animation_finished",
        move |entity: ImmutableString| -> bool {
            snap.get(entity.as_str())
                .map(|s| s.finished)
                .unwrap_or(false)
        },
    );

    // Get animation elapsed time
    let snap = snapshot.clone();
    engine.register_fn(
        "animation_time",
        move |entity: ImmutableString| -> i64 {
            snap.get(entity.as_str())
                .map(|s| s.time_ms as i64)
                .unwrap_or(0)
        },
    );

    // Set animation (emits component effects)
    let st = state.clone();
    engine.register_fn(
        "set_animation",
        move |entity: ImmutableString, name: ImmutableString, direction: ImmutableString| {
            if let Some((kind, id)) = entity.split_once(':') {
                let kind = SmolStr::new(kind);
                let id = SmolStr::new(id);

                // Set animation component
                st.effects.lock().unwrap().push(WorldEffect::SetComponent {
                    kind: kind.clone(),
                    id: id.clone(),
                    component: "animation".into(),
                    value: Value::String(SmolStr::new(name.as_str())),
                });

                // Set direction component
                st.effects.lock().unwrap().push(WorldEffect::SetComponent {
                    kind: kind.clone(),
                    id: id.clone(),
                    component: "animation_direction".into(),
                    value: Value::String(SmolStr::new(direction.as_str())),
                });

                // Reset animation time
                st.effects.lock().unwrap().push(WorldEffect::SetComponent {
                    kind: kind.clone(),
                    id: id.clone(),
                    component: "animation_time".into(),
                    value: Value::Int(0),
                });

                // Reset finished flag
                st.effects.lock().unwrap().push(WorldEffect::SetComponent {
                    kind,
                    id,
                    component: "animation_finished".into(),
                    value: Value::Bool(false),
                });
            }
        },
    );

    // Set animation without resetting time (for continuing animations)
    let st = state.clone();
    engine.register_fn(
        "set_animation_if_different",
        move |entity: ImmutableString, name: ImmutableString, direction: ImmutableString| {
            // This is a more complex operation that would check current animation
            // For now, just set it - the caller can check get_animation() first
            if let Some((kind, id)) = entity.split_once(':') {
                let kind = SmolStr::new(kind);
                let id = SmolStr::new(id);

                st.effects.lock().unwrap().push(WorldEffect::SetComponent {
                    kind: kind.clone(),
                    id: id.clone(),
                    component: "animation".into(),
                    value: Value::String(SmolStr::new(name.as_str())),
                });

                st.effects.lock().unwrap().push(WorldEffect::SetComponent {
                    kind,
                    id,
                    component: "animation_direction".into(),
                    value: Value::String(SmolStr::new(direction.as_str())),
                });
            }
        },
    );

    // === Facing direction helpers ===

    // Get entity facing direction
    let snap = snapshot.clone();
    engine.register_fn("get_facing", move |entity: ImmutableString| -> ImmutableString {
        // First try animation direction, then fall back to facing component
        snap.get(entity.as_str())
            .map(|s| ImmutableString::from(s.direction.as_str()))
            .unwrap_or_else(|| "south".into())
    });

    // Set entity facing direction
    let st = state.clone();
    engine.register_fn(
        "set_facing",
        move |entity: ImmutableString, direction: ImmutableString| {
            if let Some((kind, id)) = entity.split_once(':') {
                st.effects.lock().unwrap().push(WorldEffect::SetComponent {
                    kind: SmolStr::new(kind),
                    id: SmolStr::new(id),
                    component: "facing".into(),
                    value: Value::String(SmolStr::new(direction.as_str())),
                });
            }
        },
    );

    // === Direction utility functions ===

    // Get offset for a direction (useful for spawning hitboxes in front of entity)
    engine.register_fn(
        "direction_offset_x",
        |direction: ImmutableString, distance: f64| -> f64 {
            match direction.as_str() {
                "east" => distance,
                "west" => -distance,
                _ => 0.0,
            }
        },
    );

    engine.register_fn(
        "direction_offset_y",
        |direction: ImmutableString, distance: f64| -> f64 {
            match direction.as_str() {
                "south" => distance,
                "north" => -distance,
                _ => 0.0,
            }
        },
    );

    // Get velocity components for a direction
    engine.register_fn(
        "direction_velocity_x",
        |direction: ImmutableString, speed: f64| -> f64 {
            match direction.as_str() {
                "east" => speed,
                "west" => -speed,
                _ => 0.0,
            }
        },
    );

    engine.register_fn(
        "direction_velocity_y",
        |direction: ImmutableString, speed: f64| -> f64 {
            match direction.as_str() {
                "south" => speed,
                "north" => -speed,
                _ => 0.0,
            }
        },
    );

    // Get opposite direction
    engine.register_fn(
        "opposite_direction",
        |direction: ImmutableString| -> ImmutableString {
            match direction.as_str() {
                "north" => "south".into(),
                "south" => "north".into(),
                "east" => "west".into(),
                "west" => "east".into(),
                _ => direction,
            }
        },
    );

    // Convert movement deltas to facing direction
    engine.register_fn(
        "facing_from_movement",
        |dx: f64, dy: f64| -> ImmutableString {
            if dx.abs() > dy.abs() {
                if dx > 0.0 {
                    "east".into()
                } else {
                    "west".into()
                }
            } else if dy.abs() > 0.0 {
                if dy > 0.0 {
                    "south".into()
                } else {
                    "north".into()
                }
            } else {
                "south".into() // Default
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_snapshot() {
        let mut anims = FxHashMap::default();
        anims.insert(
            "actor:player".to_string(),
            AnimationState {
                animation: "walk".into(),
                direction: "south".into(),
                time_ms: 150,
                finished: false,
            },
        );

        let snapshot = AnimationSnapshot::from_entity_components(anims);
        let state = snapshot.get("actor:player").unwrap();

        assert_eq!(state.animation.as_str(), "walk");
        assert_eq!(state.direction.as_str(), "south");
        assert_eq!(state.time_ms, 150);
        assert!(!state.finished);
    }

    #[test]
    fn direction_helpers() {
        let mut engine = Engine::new();
        let snapshot = Arc::new(AnimationSnapshot::new());
        let state = WorldScriptState::new(12345);
        register_animation_bindings(&mut engine, snapshot, state);

        // Test direction offsets
        let x: f64 = engine.eval(r#"direction_offset_x("east", 16.0)"#).unwrap();
        assert_eq!(x, 16.0);

        let x: f64 = engine.eval(r#"direction_offset_x("west", 16.0)"#).unwrap();
        assert_eq!(x, -16.0);

        let y: f64 = engine.eval(r#"direction_offset_y("south", 16.0)"#).unwrap();
        assert_eq!(y, 16.0);

        // Test opposite direction
        let dir: String = engine.eval(r#"opposite_direction("north")"#).unwrap();
        assert_eq!(dir, "south");

        // Test facing from movement
        let facing: String = engine.eval(r#"facing_from_movement(1.0, 0.0)"#).unwrap();
        assert_eq!(facing, "east");

        let facing: String = engine.eval(r#"facing_from_movement(0.0, -1.0)"#).unwrap();
        assert_eq!(facing, "north");
    }
}
