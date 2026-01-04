//! Rhai bindings for tilemap queries.
//!
//! These bindings provide script access to:
//! - Tile information at positions
//! - Collision/walkability checks
//! - Current room information
//!
//! # Example
//!
//! ```rhai
//! // Check if a position is walkable
//! if is_walkable(128.0, 88.0) {
//!     move_to(128.0, 88.0);
//! }
//!
//! // Get current room
//! let room = current_room();
//!
//! // Get tile at position
//! let tile = tile_at(64.0, 32.0);
//! if tile.collision == "solid" {
//!     // blocked!
//! }
//! ```

use rhai::{Dynamic, Engine, ImmutableString};
use smol_str::SmolStr;
use std::sync::Arc;

/// Snapshot of tilemap state for read-only access in scripts.
#[derive(Clone)]
pub struct TilemapSnapshot {
    /// Current room ID.
    pub current_room_id: SmolStr,
    /// Current room width in pixels.
    pub room_width: f32,
    /// Current room height in pixels.
    pub room_height: f32,
    /// Tile size in pixels.
    pub tile_size: u16,
    /// Tile data for current room (row-major).
    pub tiles: Vec<TileSnapshot>,
    /// Room width in tiles.
    pub tiles_width: usize,
    /// Room height in tiles.
    pub tiles_height: usize,
}

/// Snapshot of a single tile.
#[derive(Clone)]
pub struct TileSnapshot {
    /// Tile ID.
    pub id: u16,
    /// Collision type as string.
    pub collision: SmolStr,
    /// Whether this tile blocks movement.
    pub is_blocking: bool,
    /// Tile name.
    pub name: SmolStr,
    /// Tile tags.
    pub tags: Vec<SmolStr>,
}

impl TilemapSnapshot {
    /// Create a snapshot from room and tileset data.
    pub fn from_room(
        room_id: &str,
        tiles: &[u16],
        tiles_width: usize,
        tiles_height: usize,
        tile_size: u16,
        tile_info: impl Fn(u16) -> TileSnapshot,
    ) -> Self {
        Self {
            current_room_id: SmolStr::new(room_id),
            room_width: tiles_width as f32 * tile_size as f32,
            room_height: tiles_height as f32 * tile_size as f32,
            tile_size,
            tiles: tiles.iter().map(|&id| tile_info(id)).collect(),
            tiles_width,
            tiles_height,
        }
    }

    /// Get the tile at a pixel position.
    pub fn tile_at_pixel(&self, x: f32, y: f32) -> Option<&TileSnapshot> {
        if x < 0.0 || y < 0.0 || x >= self.room_width || y >= self.room_height {
            return None;
        }
        let tx = (x / self.tile_size as f32) as usize;
        let ty = (y / self.tile_size as f32) as usize;
        self.tile_at(tx, ty)
    }

    /// Get the tile at a tile position.
    pub fn tile_at(&self, tx: usize, ty: usize) -> Option<&TileSnapshot> {
        if tx >= self.tiles_width || ty >= self.tiles_height {
            return None;
        }
        self.tiles.get(ty * self.tiles_width + tx)
    }

    /// Check if a position is walkable (no blocking tiles in hitbox).
    pub fn is_walkable(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        if x < 0.0 || y < 0.0 {
            return false;
        }
        if x + width > self.room_width || y + height > self.room_height {
            return false;
        }

        let ts = self.tile_size as f32;
        let start_tx = (x / ts) as usize;
        let start_ty = (y / ts) as usize;
        let end_tx = ((x + width - 0.001) / ts) as usize;
        let end_ty = ((y + height - 0.001) / ts) as usize;

        for ty in start_ty..=end_ty {
            for tx in start_tx..=end_tx {
                if let Some(tile) = self.tile_at(tx, ty) {
                    if tile.is_blocking {
                        return false;
                    }
                } else {
                    return false; // Out of bounds = not walkable
                }
            }
        }

        true
    }
}

/// Register tilemap bindings with a Rhai engine.
///
/// These bindings provide read-only access to tilemap state for scripts.
pub fn register_tilemap_bindings(engine: &mut Engine, snapshot: Arc<TilemapSnapshot>) {
    // Get current room ID
    let snap = snapshot.clone();
    engine.register_fn("current_room", move || -> ImmutableString {
        snap.current_room_id.as_str().into()
    });

    // Get room dimensions
    let snap = snapshot.clone();
    engine.register_fn("room_width", move || -> f64 {
        snap.room_width as f64
    });

    let snap = snapshot.clone();
    engine.register_fn("room_height", move || -> f64 {
        snap.room_height as f64
    });

    let snap = snapshot.clone();
    engine.register_fn("tile_size", move || -> i64 {
        snap.tile_size as i64
    });

    // Get tile at pixel position
    let snap = snapshot.clone();
    engine.register_fn("tile_at", move |x: f64, y: f64| -> Dynamic {
        snap.tile_at_pixel(x as f32, y as f32)
            .map(|tile| {
                let mut map = rhai::Map::new();
                map.insert("id".into(), Dynamic::from(tile.id as i64));
                map.insert("name".into(), Dynamic::from(tile.name.to_string()));
                map.insert("collision".into(), Dynamic::from(tile.collision.to_string()));
                map.insert("is_blocking".into(), Dynamic::from(tile.is_blocking));
                map.insert(
                    "tags".into(),
                    Dynamic::from(
                        tile.tags
                            .iter()
                            .map(|t| Dynamic::from(t.to_string()))
                            .collect::<Vec<_>>(),
                    ),
                );
                Dynamic::from(map)
            })
            .unwrap_or(Dynamic::UNIT)
    });

    // Get tile at tile coordinates
    let snap = snapshot.clone();
    engine.register_fn("tile_at_grid", move |tx: i64, ty: i64| -> Dynamic {
        if tx < 0 || ty < 0 {
            return Dynamic::UNIT;
        }
        snap.tile_at(tx as usize, ty as usize)
            .map(|tile| {
                let mut map = rhai::Map::new();
                map.insert("id".into(), Dynamic::from(tile.id as i64));
                map.insert("name".into(), Dynamic::from(tile.name.to_string()));
                map.insert("collision".into(), Dynamic::from(tile.collision.to_string()));
                map.insert("is_blocking".into(), Dynamic::from(tile.is_blocking));
                Dynamic::from(map)
            })
            .unwrap_or(Dynamic::UNIT)
    });

    // Check if a position is walkable (point check)
    let snap = snapshot.clone();
    engine.register_fn("is_walkable", move |x: f64, y: f64| -> bool {
        snap.tile_at_pixel(x as f32, y as f32)
            .map(|tile| !tile.is_blocking)
            .unwrap_or(false)
    });

    // Check if a rectangle is walkable (hitbox check)
    let snap = snapshot.clone();
    engine.register_fn(
        "is_walkable_rect",
        move |x: f64, y: f64, width: f64, height: f64| -> bool {
            snap.is_walkable(x as f32, y as f32, width as f32, height as f32)
        },
    );

    // Get collision type at position
    let snap = snapshot.clone();
    engine.register_fn("collision_at", move |x: f64, y: f64| -> ImmutableString {
        snap.tile_at_pixel(x as f32, y as f32)
            .map(|tile| tile.collision.as_str().into())
            .unwrap_or_else(|| "solid".into())
    });

    // Check if tile has a specific tag
    let snap = snapshot.clone();
    engine.register_fn(
        "tile_has_tag",
        move |x: f64, y: f64, tag: ImmutableString| -> bool {
            snap.tile_at_pixel(x as f32, y as f32)
                .map(|tile| tile.tags.iter().any(|t| t == tag.as_str()))
                .unwrap_or(false)
        },
    );

    // Convert pixel to tile coordinates
    let snap = snapshot.clone();
    engine.register_fn("pixel_to_tile_x", move |x: f64| -> i64 {
        (x / snap.tile_size as f64) as i64
    });

    let snap = snapshot.clone();
    engine.register_fn("pixel_to_tile_y", move |y: f64| -> i64 {
        (y / snap.tile_size as f64) as i64
    });

    // Convert tile to pixel coordinates (center of tile)
    let snap = snapshot.clone();
    engine.register_fn("tile_to_pixel_x", move |tx: i64| -> f64 {
        (tx as f64 + 0.5) * snap.tile_size as f64
    });

    let snap = snapshot.clone();
    engine.register_fn("tile_to_pixel_y", move |ty: i64| -> f64 {
        (ty as f64 + 0.5) * snap.tile_size as f64
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_snapshot() -> TilemapSnapshot {
        // Create a simple 4x4 room with a border of walls
        let tiles = vec![
            // Row 0: all walls
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            // Row 1: wall, floor, floor, wall
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            TileSnapshot {
                id: 0,
                collision: "walkable".into(),
                is_blocking: false,
                name: "floor".into(),
                tags: vec!["ground".into()],
            },
            TileSnapshot {
                id: 0,
                collision: "walkable".into(),
                is_blocking: false,
                name: "floor".into(),
                tags: vec!["ground".into()],
            },
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            // Row 2: wall, floor, floor, wall
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            TileSnapshot {
                id: 0,
                collision: "walkable".into(),
                is_blocking: false,
                name: "floor".into(),
                tags: vec!["ground".into()],
            },
            TileSnapshot {
                id: 0,
                collision: "walkable".into(),
                is_blocking: false,
                name: "floor".into(),
                tags: vec!["ground".into()],
            },
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            // Row 3: all walls
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
            TileSnapshot {
                id: 1,
                collision: "solid".into(),
                is_blocking: true,
                name: "wall".into(),
                tags: vec![],
            },
        ];

        TilemapSnapshot {
            current_room_id: "test_room".into(),
            room_width: 64.0,
            room_height: 64.0,
            tile_size: 16,
            tiles,
            tiles_width: 4,
            tiles_height: 4,
        }
    }

    #[test]
    fn test_tile_at() {
        let snapshot = create_test_snapshot();

        // Wall at (0, 0)
        let tile = snapshot.tile_at(0, 0).unwrap();
        assert!(tile.is_blocking);
        assert_eq!(tile.name, "wall");

        // Floor at (1, 1)
        let tile = snapshot.tile_at(1, 1).unwrap();
        assert!(!tile.is_blocking);
        assert_eq!(tile.name, "floor");
    }

    #[test]
    fn test_tile_at_pixel() {
        let snapshot = create_test_snapshot();

        // Pixel (24, 24) is in tile (1, 1) which is floor
        let tile = snapshot.tile_at_pixel(24.0, 24.0).unwrap();
        assert!(!tile.is_blocking);

        // Pixel (8, 8) is in tile (0, 0) which is wall
        let tile = snapshot.tile_at_pixel(8.0, 8.0).unwrap();
        assert!(tile.is_blocking);
    }

    #[test]
    fn test_is_walkable() {
        let snapshot = create_test_snapshot();

        // Center of room should be walkable
        assert!(snapshot.is_walkable(20.0, 20.0, 8.0, 8.0));

        // Touching the wall should not be walkable
        assert!(!snapshot.is_walkable(8.0, 8.0, 8.0, 8.0));

        // Out of bounds
        assert!(!snapshot.is_walkable(-1.0, 20.0, 8.0, 8.0));
        assert!(!snapshot.is_walkable(60.0, 20.0, 8.0, 8.0));
    }

    #[test]
    fn test_rhai_bindings() {
        let snapshot = Arc::new(create_test_snapshot());
        let mut engine = Engine::new();
        register_tilemap_bindings(&mut engine, snapshot);

        // Test current_room
        let room: String = engine.eval("current_room()").unwrap();
        assert_eq!(room, "test_room");

        // Test room_width
        let width: f64 = engine.eval("room_width()").unwrap();
        assert_eq!(width, 64.0);

        // Test is_walkable
        let walkable: bool = engine.eval("is_walkable(24.0, 24.0)").unwrap();
        assert!(walkable);

        let not_walkable: bool = engine.eval("is_walkable(8.0, 8.0)").unwrap();
        assert!(!not_walkable);

        // Test collision_at
        let collision: String = engine.eval("collision_at(24.0, 24.0)").unwrap();
        assert_eq!(collision, "walkable");
    }
}
