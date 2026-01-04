//! Rhai bindings for entity-entity collision detection.
//!
//! These bindings provide script access to check if entities are overlapping,
//! useful for combat, pickups, and other interactions.
//!
//! Entity hitboxes are defined by components:
//! - `hitbox_width`: Width of the hitbox in pixels (default: 16)
//! - `hitbox_height`: Height of the hitbox in pixels (default: 16)
//! - `hitbox_offset_x`: X offset from entity position (default: 0)
//! - `hitbox_offset_y`: Y offset from entity position (default: 0)
//!
//! # Example
//!
//! ```rhai
//! let me = current_entity();
//!
//! // Check what entities I'm overlapping with
//! for other in entities_overlapping(me) {
//!     if entity_has_tag(other, "damageable") {
//!         deal_damage(other, 1);
//!     }
//! }
//!
//! // Check if two specific entities are colliding
//! if check_collision("actor:player", "enemy:octorok_1") {
//!     // Handle collision
//! }
//!
//! // Find all entities in a rectangular area
//! let nearby = entities_in_rect(100.0, 100.0, 50.0, 50.0);
//! ```

use crate::world_bindings::WorldSnapshot;
use rhai::{Dynamic, Engine, ImmutableString};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Snapshot of entity collision data.
#[derive(Clone)]
pub struct CollisionSnapshot {
    /// Entity hitboxes keyed by qualified ID.
    pub hitboxes: FxHashMap<String, EntityHitbox>,
}

/// Hitbox data for a single entity.
#[derive(Clone, Debug)]
pub struct EntityHitbox {
    /// Entity position X.
    pub x: f32,
    /// Entity position Y.
    pub y: f32,
    /// Hitbox width.
    pub width: f32,
    /// Hitbox height.
    pub height: f32,
    /// Offset from position X.
    pub offset_x: f32,
    /// Offset from position Y.
    pub offset_y: f32,
}

impl EntityHitbox {
    /// Get the actual bounding rectangle of the hitbox.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (
            self.x + self.offset_x,
            self.y + self.offset_y,
            self.width,
            self.height,
        )
    }

    /// Check if this hitbox overlaps with another.
    pub fn overlaps(&self, other: &EntityHitbox) -> bool {
        let (ax, ay, aw, ah) = self.rect();
        let (bx, by, bw, bh) = other.rect();

        // AABB overlap test
        ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
    }

    /// Check if this hitbox contains a point.
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        let (x, y, w, h) = self.rect();
        px >= x && px < x + w && py >= y && py < y + h
    }

    /// Check if this hitbox overlaps with a rectangle.
    pub fn overlaps_rect(&self, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
        let (x, y, w, h) = self.rect();
        x < rx + rw && x + w > rx && y < ry + rh && y + h > ry
    }
}

impl CollisionSnapshot {
    /// Create an empty collision snapshot.
    pub fn new() -> Self {
        Self {
            hitboxes: FxHashMap::default(),
        }
    }

    /// Create a collision snapshot from world state.
    ///
    /// Extracts position and hitbox components from entities.
    pub fn from_world_snapshot(world: &WorldSnapshot) -> Self {
        let mut hitboxes = FxHashMap::default();

        for (entity_id, entity) in &world.entities {
            // Get position
            let x = entity
                .components
                .get("position_x")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(0.0) as f32;
            let y = entity
                .components
                .get("position_y")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(0.0) as f32;

            // Get hitbox dimensions (default to 16x16)
            let width = entity
                .components
                .get("hitbox_width")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(16.0) as f32;
            let height = entity
                .components
                .get("hitbox_height")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(16.0) as f32;

            // Get hitbox offset (default to 0)
            let offset_x = entity
                .components
                .get("hitbox_offset_x")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(0.0) as f32;
            let offset_y = entity
                .components
                .get("hitbox_offset_y")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(0.0) as f32;

            hitboxes.insert(
                entity_id.clone(),
                EntityHitbox {
                    x,
                    y,
                    width,
                    height,
                    offset_x,
                    offset_y,
                },
            );
        }

        Self { hitboxes }
    }

    /// Get hitbox for an entity.
    pub fn get(&self, entity_id: &str) -> Option<&EntityHitbox> {
        self.hitboxes.get(entity_id)
    }

    /// Get all entities overlapping with the given entity.
    pub fn overlapping_with(&self, entity_id: &str) -> Vec<&str> {
        let Some(hitbox) = self.hitboxes.get(entity_id) else {
            return Vec::new();
        };

        self.hitboxes
            .iter()
            .filter(|(id, other)| *id != entity_id && hitbox.overlaps(other))
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Get all entities within a rectangle.
    pub fn in_rect(&self, x: f32, y: f32, width: f32, height: f32) -> Vec<&str> {
        self.hitboxes
            .iter()
            .filter(|(_, hb)| hb.overlaps_rect(x, y, width, height))
            .map(|(id, _)| id.as_str())
            .collect()
    }
}

impl Default for CollisionSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Register collision bindings with a Rhai engine.
pub fn register_collision_bindings(engine: &mut Engine, snapshot: Arc<CollisionSnapshot>) {
    // Get entities overlapping with the given entity
    let snap = snapshot.clone();
    engine.register_fn(
        "entities_overlapping",
        move |entity: ImmutableString| -> rhai::Array {
            snap.overlapping_with(entity.as_str())
                .into_iter()
                .map(|id| Dynamic::from(id.to_string()))
                .collect()
        },
    );

    // Check if two specific entities are colliding
    let snap = snapshot.clone();
    engine.register_fn(
        "check_collision",
        move |entity_a: ImmutableString, entity_b: ImmutableString| -> bool {
            let Some(hb_a) = snap.get(entity_a.as_str()) else {
                return false;
            };
            let Some(hb_b) = snap.get(entity_b.as_str()) else {
                return false;
            };
            hb_a.overlaps(hb_b)
        },
    );

    // Get all entities in a rectangular area
    let snap = snapshot.clone();
    engine.register_fn(
        "entities_in_rect",
        move |x: f64, y: f64, width: f64, height: f64| -> rhai::Array {
            snap.in_rect(x as f32, y as f32, width as f32, height as f32)
                .into_iter()
                .map(|id| Dynamic::from(id.to_string()))
                .collect()
        },
    );

    // Get hitbox data for an entity
    let snap = snapshot.clone();
    engine.register_fn("get_hitbox", move |entity: ImmutableString| -> Dynamic {
        snap.get(entity.as_str())
            .map(|hb| {
                let (x, y, w, h) = hb.rect();
                let mut map = rhai::Map::new();
                map.insert("x".into(), Dynamic::from(x as f64));
                map.insert("y".into(), Dynamic::from(y as f64));
                map.insert("width".into(), Dynamic::from(w as f64));
                map.insert("height".into(), Dynamic::from(h as f64));
                Dynamic::from(map)
            })
            .unwrap_or(Dynamic::UNIT)
    });

    // Check if entity contains a point
    let snap = snapshot.clone();
    engine.register_fn(
        "entity_contains_point",
        move |entity: ImmutableString, x: f64, y: f64| -> bool {
            snap.get(entity.as_str())
                .map(|hb| hb.contains_point(x as f32, y as f32))
                .unwrap_or(false)
        },
    );

    // Get distance between entity centers
    let snap = snapshot.clone();
    engine.register_fn(
        "entity_distance",
        move |entity_a: ImmutableString, entity_b: ImmutableString| -> f64 {
            let Some(hb_a) = snap.get(entity_a.as_str()) else {
                return f64::MAX;
            };
            let Some(hb_b) = snap.get(entity_b.as_str()) else {
                return f64::MAX;
            };

            // Use center of hitboxes
            let (ax, ay, aw, ah) = hb_a.rect();
            let (bx, by, bw, bh) = hb_b.rect();

            let center_ax = ax + aw / 2.0;
            let center_ay = ay + ah / 2.0;
            let center_bx = bx + bw / 2.0;
            let center_by = by + bh / 2.0;

            let dx = center_bx - center_ax;
            let dy = center_by - center_ay;

            ((dx * dx + dy * dy) as f64).sqrt()
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hitbox(x: f32, y: f32, w: f32, h: f32) -> EntityHitbox {
        EntityHitbox {
            x,
            y,
            width: w,
            height: h,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    #[test]
    fn hitbox_overlap() {
        let a = create_test_hitbox(0.0, 0.0, 16.0, 16.0);
        let b = create_test_hitbox(8.0, 8.0, 16.0, 16.0);
        let c = create_test_hitbox(100.0, 100.0, 16.0, 16.0);

        assert!(a.overlaps(&b)); // Overlapping
        assert!(!a.overlaps(&c)); // Not overlapping
    }

    #[test]
    fn hitbox_contains_point() {
        let hb = create_test_hitbox(10.0, 10.0, 20.0, 20.0);

        assert!(hb.contains_point(15.0, 15.0)); // Inside
        assert!(hb.contains_point(10.0, 10.0)); // Edge (top-left)
        assert!(!hb.contains_point(30.0, 30.0)); // Outside (edge not included)
        assert!(!hb.contains_point(5.0, 5.0)); // Outside
    }

    #[test]
    fn collision_snapshot_overlapping() {
        let mut hitboxes = FxHashMap::default();
        hitboxes.insert("a".to_string(), create_test_hitbox(0.0, 0.0, 16.0, 16.0));
        hitboxes.insert("b".to_string(), create_test_hitbox(8.0, 8.0, 16.0, 16.0));
        hitboxes.insert("c".to_string(), create_test_hitbox(100.0, 100.0, 16.0, 16.0));

        let snapshot = CollisionSnapshot { hitboxes };

        let overlapping = snapshot.overlapping_with("a");
        assert_eq!(overlapping.len(), 1);
        assert!(overlapping.contains(&"b"));

        let overlapping = snapshot.overlapping_with("c");
        assert!(overlapping.is_empty());
    }

    #[test]
    fn rhai_bindings() {
        let mut hitboxes = FxHashMap::default();
        hitboxes.insert(
            "actor:player".to_string(),
            create_test_hitbox(0.0, 0.0, 16.0, 16.0),
        );
        hitboxes.insert(
            "enemy:octorok".to_string(),
            create_test_hitbox(8.0, 8.0, 16.0, 16.0),
        );

        let snapshot = Arc::new(CollisionSnapshot { hitboxes });
        let mut engine = Engine::new();
        register_collision_bindings(&mut engine, snapshot);

        // Test collision check
        let collides: bool = engine
            .eval(r#"check_collision("actor:player", "enemy:octorok")"#)
            .unwrap();
        assert!(collides);

        // Test entities_overlapping
        let overlapping: rhai::Array = engine
            .eval(r#"entities_overlapping("actor:player")"#)
            .unwrap();
        assert_eq!(overlapping.len(), 1);
    }
}
