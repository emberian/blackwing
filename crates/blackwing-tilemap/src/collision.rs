//! Collision detection and movement resolution.

use crate::{CollisionType, Room, TileSet};

/// Result of a collision query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionResult {
    /// Tile X position.
    pub tile_x: usize,
    /// Tile Y position.
    pub tile_y: usize,
    /// Collision type of the tile.
    pub collision: CollisionType,
}

/// Query collision for a single point.
pub fn point_collision(
    room: &Room,
    tileset: &TileSet,
    x: f32,
    y: f32,
) -> CollisionType {
    if x < 0.0 || y < 0.0 {
        return CollisionType::Solid;
    }

    let tile_x = (x / tileset.tile_width as f32) as usize;
    let tile_y = (y / tileset.tile_height as f32) as usize;

    room.tile_at(tile_x, tile_y)
        .map(|tile_id| tileset.collision_for(tile_id))
        .unwrap_or(CollisionType::Solid)
}

/// Query collision for a rectangle (entity hitbox).
/// Returns all tiles the rectangle overlaps that have blocking collision.
pub fn rect_collision(
    room: &Room,
    tileset: &TileSet,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Vec<CollisionResult> {
    let mut results = Vec::new();

    if width <= 0.0 || height <= 0.0 {
        return results;
    }

    let tw = tileset.tile_width as f32;
    let th = tileset.tile_height as f32;

    // Calculate tile range
    let start_x = (x / tw).floor() as i32;
    let start_y = (y / th).floor() as i32;
    let end_x = ((x + width - 0.001) / tw).floor() as i32;
    let end_y = ((y + height - 0.001) / th).floor() as i32;

    for ty in start_y..=end_y {
        for tx in start_x..=end_x {
            if tx < 0 || ty < 0 {
                results.push(CollisionResult {
                    tile_x: 0,
                    tile_y: 0,
                    collision: CollisionType::Solid,
                });
                continue;
            }

            let tx = tx as usize;
            let ty = ty as usize;

            if let Some(tile_id) = room.tile_at(tx, ty) {
                let collision = tileset.collision_for(tile_id);
                if collision.is_blocking() {
                    results.push(CollisionResult {
                        tile_x: tx,
                        tile_y: ty,
                        collision,
                    });
                }
            } else {
                // Out of bounds = solid
                results.push(CollisionResult {
                    tile_x: tx,
                    tile_y: ty,
                    collision: CollisionType::Solid,
                });
            }
        }
    }

    results
}

/// Check if a rectangle can be placed at a position without collision.
pub fn can_place(
    room: &Room,
    tileset: &TileSet,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> bool {
    rect_collision(room, tileset, x, y, width, height).is_empty()
}

/// Resolve movement with collision, returning the actual delta that can be applied.
/// Uses separate X/Y resolution to allow sliding along walls.
pub fn resolve_movement(
    room: &Room,
    tileset: &TileSet,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    dx: f32,
    dy: f32,
) -> (f32, f32) {
    let mut result_dx = dx;
    let mut result_dy = dy;

    // Try full movement first
    if can_place(room, tileset, x + dx, y + dy, width, height) {
        return (dx, dy);
    }

    // Try X movement only
    if dx != 0.0 && can_place(room, tileset, x + dx, y, width, height) {
        // X works, try Y separately
        if dy != 0.0 && !can_place(room, tileset, x + dx, y + dy, width, height) {
            result_dy = 0.0;
        }
    } else {
        result_dx = 0.0;
        // Try Y movement only
        if dy != 0.0 && !can_place(room, tileset, x, y + dy, width, height) {
            result_dy = 0.0;
        }
    }

    (result_dx, result_dy)
}

/// More precise movement resolution that finds the exact collision point.
/// Useful for pixel-perfect movement.
pub fn resolve_movement_precise(
    room: &Room,
    tileset: &TileSet,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    dx: f32,
    dy: f32,
    steps: u32,
) -> (f32, f32) {
    if steps == 0 || (dx == 0.0 && dy == 0.0) {
        return (0.0, 0.0);
    }

    let step_dx = dx / steps as f32;
    let step_dy = dy / steps as f32;

    let mut final_dx = 0.0;
    let mut final_dy = 0.0;

    for _ in 0..steps {
        // Try both axes
        if can_place(room, tileset, x + final_dx + step_dx, y + final_dy + step_dy, width, height) {
            final_dx += step_dx;
            final_dy += step_dy;
            continue;
        }

        // Try X only
        let x_ok = can_place(room, tileset, x + final_dx + step_dx, y + final_dy, width, height);
        // Try Y only
        let y_ok = can_place(room, tileset, x + final_dx, y + final_dy + step_dy, width, height);

        if x_ok {
            final_dx += step_dx;
        }
        if y_ok {
            final_dy += step_dy;
        }

        // If neither worked, we're stuck
        if !x_ok && !y_ok {
            break;
        }
    }

    (final_dx, final_dy)
}

/// Check if an entity at a position is standing on a specific collision type.
pub fn standing_on(
    room: &Room,
    tileset: &TileSet,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    check: CollisionType,
) -> bool {
    let tw = tileset.tile_width as f32;
    let th = tileset.tile_height as f32;

    // Check tiles under the entity
    let start_x = (x / tw).floor() as i32;
    let end_x = ((x + width - 0.001) / tw).floor() as i32;
    let foot_y = ((y + height) / th).floor() as i32;

    for tx in start_x..=end_x {
        if tx < 0 || foot_y < 0 {
            continue;
        }
        if let Some(tile_id) = room.tile_at(tx as usize, foot_y as usize) {
            if tileset.collision_for(tile_id) == check {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TileDef;

    fn make_test_room_and_tileset() -> (Room, TileSet) {
        let tileset = TileSet {
            id: "test".into(),
            name: "Test".into(),
            texture_path: "test.png".into(),
            tile_width: 16,
            tile_height: 16,
            tiles_per_row: 16,
            tiles: vec![
                TileDef::walkable(0, "floor", 0),
                TileDef::solid(1, "wall", 1),
            ],
        };

        let mut room = Room::with_size("test", "test", 8, 8);
        // Fill with floor
        room.fill_rect(0, 0, 8, 8, 0);
        // Add wall border
        room.fill_border(1);

        (room, tileset)
    }

    #[test]
    fn point_collision_basic() {
        let (room, tileset) = make_test_room_and_tileset();

        // Center is walkable
        assert_eq!(
            point_collision(&room, &tileset, 64.0, 64.0),
            CollisionType::Walkable
        );

        // Border is solid
        assert_eq!(
            point_collision(&room, &tileset, 8.0, 8.0),
            CollisionType::Solid
        );

        // Negative is solid
        assert_eq!(
            point_collision(&room, &tileset, -1.0, 50.0),
            CollisionType::Solid
        );
    }

    #[test]
    fn rect_collision_basic() {
        let (room, tileset) = make_test_room_and_tileset();

        // Center, no collision (tile 2,2 is walkable)
        let results = rect_collision(&room, &tileset, 32.0, 32.0, 16.0, 16.0);
        assert!(results.is_empty());

        // Overlapping wall border at tile (0, 1) - wall is at x=0
        let results = rect_collision(&room, &tileset, 0.0, 16.0, 16.0, 16.0);
        assert!(!results.is_empty());
    }

    #[test]
    fn movement_resolution() {
        let (room, tileset) = make_test_room_and_tileset();

        // Move in open area - should work
        let (dx, dy) = resolve_movement(&room, &tileset, 64.0, 64.0, 12.0, 12.0, 4.0, 0.0);
        assert_eq!(dx, 4.0);
        assert_eq!(dy, 0.0);

        // Move into wall - should be blocked
        let (dx, _dy) = resolve_movement(&room, &tileset, 20.0, 64.0, 12.0, 12.0, -10.0, 0.0);
        assert_eq!(dx, 0.0);
    }

    #[test]
    fn can_place_check() {
        let (room, tileset) = make_test_room_and_tileset();

        // Open area
        assert!(can_place(&room, &tileset, 32.0, 32.0, 16.0, 16.0));

        // Overlapping wall
        assert!(!can_place(&room, &tileset, 0.0, 0.0, 32.0, 32.0));
    }
}
